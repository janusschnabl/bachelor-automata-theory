pub mod errors;
pub mod dot;
pub mod automaton;
pub use crate::errors::{Error, Result};
pub use crate::automaton::Automaton;
use crate::automaton::automaton_isomorphic;
use regex_syntax::ast::{parse::Parser, Ast};
use std::collections::HashSet;
use std::fmt;

//TODO: i want to move the thompson construction logic to a seperate file and keep only the core logic here.
#[derive(Debug, Clone, Default)]
pub struct EpsilonNfa {
    pub states: Vec<State>,
    pub start: usize,
    pub accept: usize,
    pub alphabet: HashSet<u8>,
}
#[derive(Debug, Clone)]
pub struct State {
    pub transitions: Vec<(Symbol, usize)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Symbol {
    Epsilon,
    Byte(u8),
}


impl EpsilonNfa {
    pub fn from_regex(regex: &str, alphabet: Option<HashSet<u8>>) -> Result<Self> {
        if !regex.is_ascii() {
            return Err(Error::UnsupportedFeature("only ASCII regex supported"));
        }

        let ast = Parser::new().parse(regex)?;

        let mut nfa = EpsilonNfa::new();
        let (start, accept) = nfa.build_from_ast(&ast)?;
        nfa.start = start;
        nfa.accept = accept;
        
        let used_symbols = nfa.extract_used_symbols();
        nfa.alphabet = match alphabet {
            Some(custom_alphabet) => {
                if !used_symbols.is_subset(&custom_alphabet) {
                    return Err(Error::InvalidInput(
                        "provided alphabet does not include all symbols in the automata".to_string(),
                    ));
                }
                custom_alphabet
            }
            None => used_symbols,
        };

        Ok(nfa)
    }

    fn extract_used_symbols(&self) -> HashSet<u8> {
        let mut symbols = HashSet::new();
        for state in &self.states {
            for (symbol, _) in &state.transitions {
                if let Symbol::Byte(b) = symbol {
                    symbols.insert(*b);
                }
            }
        }
        symbols
    }

    //TODO: JAnus har også implementeret det her et andet sted, så det skal lige forenes.
    pub fn epsilon_closure(&self, state: usize) -> HashSet<usize> {
        let mut closure = HashSet::new();
        let mut stack = vec![state];

        while let Some(current) = stack.pop() {
            if closure.insert(current) {
                for (symbol, next) in &self.states[current].transitions {
                    if matches!(symbol, Symbol::Epsilon) {
                        stack.push(*next);
                    }
                }
            }
        }

        closure
    }

    fn build_from_ast(&mut self, ast: &Ast) -> Result<(usize, usize)> {
        match ast {
            Ast::Literal(lit) => {
                let b = lit.c as u32;
                if b > 255 {
                    return Err(Error::UnsupportedFeature("non ASCII literal"));
                }
                self.build_literal(&[b as u8])
            }

            Ast::Alternation(alt) => self.build_alternation(&alt.asts),

            Ast::Concat(concat) => self.build_concat(&concat.asts),

            Ast::Repetition(rep) => self.build_repetition(rep),

            Ast::Group(group) => {self.build_from_ast(&group.ast)},

            Ast::Empty(_) => self.build_empty(),

            _ => Err(Error::UnsupportedFeature("unsupported AST node")),
        }
    }
    fn add_state(&mut self) -> usize {
        let id = self.states.len();
        self.states.push(State {
            transitions: vec![],
        });
        id
    }
    fn add_transition(&mut self, from: usize, symbol: Symbol, to: usize) {
        self.states[from].transitions.push((symbol, to));
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

        // build first
        let (start, mut accept) = self.build_from_ast(&subs[0])?;

        // chain the rest
        for sub in &subs[1..] {
            let (next_start, next_accept) = self.build_from_ast(sub)?;

            // connect previous accept to next start
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

            _ => Err(Error::UnsupportedFeature("only * and + supported")),
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

        // for single/first byte
        let start = self.add_state();
        let mut accept = self.add_state();
        self.add_transition(start, Symbol::Byte(bytes[0]), accept);

        // for remaining bytes
        for &b in &bytes[1..] {
            let next_start = self.add_state();
            let next_accept = self.add_state();
            self.add_transition(next_start, Symbol::Byte(b), next_accept);
            self.add_transition(accept, Symbol::Epsilon, next_start);
            accept = next_accept;
        }

        Ok((start, accept))
    }
    pub fn new() -> Self {
        Self::default()
    }
}



impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Symbol::Epsilon => write!(f, "ε"),
            Symbol::Byte(b) => {
                if b.is_ascii_graphic() {
                    write!(f, "{}", *b as char)
                } else {
                    write!(f, "0x{:02X}", b)
                }
            }
        }
    }
}


impl fmt::Display for EpsilonNfa {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, state) in self.states.iter().enumerate() {
            writeln!(f, "State {}:", i)?;

            if state.transitions.is_empty() {
                writeln!(f, "  (no outgoing transitions)")?;
            }

            for (symbol, target) in &state.transitions {
                writeln!(f, "  --{}--> {}", symbol, target)?;
            }

            writeln!(f)?; // blank line between states
        }

        Ok(())
    }
}


impl Automaton for EpsilonNfa {
    type Label = Symbol;

    fn state_count(&self) -> usize {
        self.states.len()
    }

    fn start_state(&self) -> usize {
        self.start
    }

    fn accept_states(&self) -> HashSet<usize> {
        let mut set = HashSet::new();
        set.insert(self.accept);
        set
    }

    fn transitions_from(&self, state: usize) -> Vec<(Self::Label, usize)> {
        self.states[state].transitions.clone()
    }

    fn alphabet(&self) -> &HashSet<u8> {
        &self.alphabet
    }

    fn encode_label(label: &Symbol) -> String {
        match label {
            Symbol::Epsilon => "ε".to_string(),
            Symbol::Byte(b) => {
                if b.is_ascii_graphic() {
                    format!("{}", *b as char)
                } else {
                    format!("0x{:02X}", b)
                }
            }
        }
    }

    fn decode_label(label: &str) -> Result<Symbol> {
        if label == "ε" {
            Ok(Symbol::Epsilon)
        } else if label.len() == 1 {
            Ok(Symbol::Byte(label.as_bytes()[0]))
        } else {
            Err(Error::InvalidInput(format!("invalid label: {label}")))
        }
    }

    fn next_states(&self, state: usize, byte: u8) -> HashSet<usize> {
        // From this state and all epsilon-reachable states, find byte transitions and epsilon-close results
        let mut next = HashSet::new();
        let closure = self.epsilon_closure(state);  // Epsilon-close the source first
        for s in closure {
            for (symbol, target) in &self.states[s].transitions {
                if let Symbol::Byte(b) = symbol {
                    if *b == byte {
                        next.extend(self.epsilon_closure(*target));  // Epsilon-close the target
                    }
                }
            }
        }
        next
    }

    fn accepts(&self, word: &str) -> bool {
        let initial_states = self.epsilon_closure(self.start);
        self.accepts_from_states(&initial_states, word)
    }
}

