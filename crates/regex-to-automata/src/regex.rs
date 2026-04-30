use crate::epsilon_nfa::{EpsilonNfa, Symbol};
use crate::automaton::Automaton;
use crate::Result;
use regex_syntax::ast::{parse::Parser, Ast};
use std::collections::HashSet;

impl EpsilonNfa {
    pub fn from_regex(regex: &str, alphabet: Option<HashSet<u8>>) -> Result<Self> {
        if !regex.is_ascii() {
            return Err(crate::Error::UnsupportedFeature("only ASCII regex supported"));
        }

        if regex.contains('\\') {
            return Err(crate::Error::UnsupportedFeature("backslash not supported"));
        }

        let ast = Parser::new().parse(regex)?;

        let mut nfa = EpsilonNfa::new();
        let (start, accept) = nfa.build_from_ast(&ast)?;
        nfa.start = start;
        nfa.accept = accept;

        let used_symbols = nfa.extract_used_symbols();
        EpsilonNfa::ensure_dot_friendly_labels(&used_symbols)?;
        
        nfa.alphabet = match alphabet {
            Some(custom_alphabet) => {
                if !used_symbols.is_subset(&custom_alphabet) {
                    return Err(crate::Error::InvalidInput(
                        "provided alphabet does not include all symbols in the automata".to_string(),
                    ));
                }
                custom_alphabet
            }
            None => used_symbols,
        };

        Ok(nfa)
    }

    fn build_from_ast(&mut self, ast: &Ast) -> Result<(usize, usize)> {
        match ast {
            Ast::Literal(lit) => {
                let b = lit.c as u32;
                if b > 255 {
                    return Err(crate::Error::UnsupportedFeature("non ASCII literal"));
                }
                self.build_literal(&[b as u8])
            }

            Ast::Alternation(alt) => self.build_alternation(&alt.asts),

            Ast::Concat(concat) => self.build_concat(&concat.asts),

            Ast::Repetition(rep) => self.build_repetition(rep),

            Ast::Group(group) => self.build_from_ast(&group.ast),

            Ast::Empty(_) => self.build_empty(),

            _ => Err(crate::Error::UnsupportedFeature("unsupported AST node")),
        }
    }

    fn build_alternation(&mut self, subs: &[Ast]) -> Result<(usize, usize)> {
        let mut iter = subs.iter();

        let mut current = self.build_from_ast(iter.next().unwrap())?;

        for sub in iter {
            let right = self.build_from_ast(sub)?;
            current = self.build_binary_alternation(current, right);
        }

        Ok(current)
    }

    fn build_binary_alternation(
        &mut self,
        left: (usize, usize),
        right: (usize, usize),
    ) -> (usize, usize) {
        let start = self.add_state();
        let accept = self.add_state();

        let (l_start, l_accept) = left;
        let (r_start, r_accept) = right;

        self.add_transition(start, Symbol::Epsilon, l_start);
        self.add_transition(start, Symbol::Epsilon, r_start);

        self.add_transition(l_accept, Symbol::Epsilon, accept);
        self.add_transition(r_accept, Symbol::Epsilon, accept);

        (start, accept)
    }

    fn build_concat(&mut self, subs: &[Ast]) -> Result<(usize, usize)> {
        assert!(!subs.is_empty());

        let (start, mut accept) = self.build_from_ast(&subs[0])?;

        for sub in &subs[1..] {
            let (next_start, next_accept) = self.build_from_ast(sub)?;

            self.add_transition(accept, Symbol::Epsilon, next_start);

            accept = next_accept;
        }

        Ok((start, accept))
    }

    fn build_repetition(
        &mut self,
        rep: &regex_syntax::ast::Repetition,
    ) -> Result<(usize, usize)> {
        match rep.op.kind {
            regex_syntax::ast::RepetitionKind::ZeroOrMore => {
                let (sub_start, sub_accept) = self.build_from_ast(&rep.ast)?;

                let start = self.add_state();
                let accept = self.add_state();

                self.add_transition(start, Symbol::Epsilon, sub_start);
                self.add_transition(start, Symbol::Epsilon, accept);

                self.add_transition(sub_accept, Symbol::Epsilon, sub_start);
                self.add_transition(sub_accept, Symbol::Epsilon, accept);

                Ok((start, accept))
            }

            regex_syntax::ast::RepetitionKind::OneOrMore => {
                let (sub_start, sub_accept) = self.build_from_ast(&rep.ast)?;

                let start = self.add_state();
                let accept = self.add_state();

                self.add_transition(start, Symbol::Epsilon, sub_start);
                self.add_transition(sub_accept, Symbol::Epsilon, sub_start);
                self.add_transition(sub_accept, Symbol::Epsilon, accept);

                Ok((start, accept))
            }

            _ => Err(crate::Error::UnsupportedFeature("only * and + supported")),
        }
    }

    fn build_empty(&mut self) -> Result<(usize, usize)> {
        let start = self.add_state();
        let accept = self.add_state();
        self.add_transition(start, Symbol::Epsilon, accept);
        Ok((start, accept))
    }

    fn build_literal(&mut self, bytes: &[u8]) -> Result<(usize, usize)> {
        assert!(!bytes.is_empty());

        let start = self.add_state();
        let mut accept = self.add_state();
        self.add_transition(start, Symbol::Byte(bytes[0]), accept);

        for &b in &bytes[1..] {
            let next_start = self.add_state();
            let next_accept = self.add_state();
            self.add_transition(next_start, Symbol::Byte(b), next_accept);
            self.add_transition(accept, Symbol::Epsilon, next_start);
            accept = next_accept;
        }

        Ok((start, accept))
    }
}

/// Generates a random valid regex using supported operators
///
/// # Arguments
/// * `rng` - Random number generator
/// * `depth` - Recursion depth for regex construction
/// * `num_literals` - Number of literals to use from the alphabet (a, b, c, ... up to z)
///
/// # Errors
/// Returns error if `num_literals` is 0 or greater than 26
pub fn generate_random_regex<R: rand::Rng>(
    rng: &mut R,
    depth: usize,
    num_literals: usize,
) -> crate::Result<String> {
    if num_literals == 0 || num_literals > 26 {
        return Err(crate::Error::InvalidInput(
            "num_literals must be between 1 and 26".to_string(),
        ));
    }

    let literals: Vec<char> = ('a'..('a' as u8 + num_literals as u8) as char)
        .collect();

    Ok(generate_random_regex_recursively(rng, depth, &literals))
}

fn generate_random_regex_recursively<R: rand::Rng>(
    rng: &mut R,
    depth: usize,
    literals: &Vec<char>,
) -> String {
    if depth == 0 {
        if rng.random_range(0..100) < 5 {
            return String::new();
        }
        return literals[rng.random_range(0..literals.len())].to_string();
    }

    match rng.random_range(0..5) {
        0 => {
            if rng.random_range(0..100) < 30 {
                String::new()
            } else {
                literals[rng.random_range(0..literals.len())].to_string()
            }
        },
        1 => format!(
            "{}{}",
            generate_random_regex_recursively(rng, depth - 1, literals),
            generate_random_regex_recursively(rng, depth - 1, literals)
        ),
        2 => format!(
            "{}|{}",
            generate_random_regex_recursively(rng, depth - 1, literals),
            generate_random_regex_recursively(rng, depth - 1, literals)
        ),
        3 => format!("({})*", generate_random_regex_recursively(rng, depth - 1, literals)),
        _ => format!("({})+", generate_random_regex_recursively(rng, depth - 1, literals)),
    }
}
