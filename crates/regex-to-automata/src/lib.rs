pub mod errors;
pub use crate::errors::{Error, Result};

use regex_syntax::ast::{parse::Parser, Ast};
use std::fmt;

#[derive(Debug, Clone, Default)]
pub struct EpsilonNfa {
    pub states: Vec<State>,
    pub start: usize,
    pub accept: usize,
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
    pub fn from_regex(regex: &str) -> Result<Self> {
        if !regex.is_ascii() {
            return Err(Error::UnsupportedFeature("only ASCII regex supported"));
        }

        let ast = Parser::new().parse(regex)?;

        let mut nfa = EpsilonNfa::new();
        let (start, accept) = nfa.build_from_ast(&ast)?;
        nfa.start = start;
        nfa.accept = accept;

        Ok(nfa)
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
        let start = self.add_state();
        let accept = self.add_state();

        for sub in subs {
            let (sub_start, sub_accept) = self.build_from_ast(sub)?;

            // start -> branch
            self.add_transition(start, Symbol::Epsilon, sub_start);

            // branch -> accept
            self.add_transition(sub_accept, Symbol::Epsilon, accept);
        }

        Ok((start, accept))
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

impl EpsilonNfa {
    pub fn to_dot(&self) -> String {
        let mut s = String::new();

        s.push_str("digraph NFA {\n");
        s.push_str("  rankdir=LR;\n");

        // Nodes
        for (i, _) in self.states.iter().enumerate() {
            let mut attrs = Vec::new();

            if i == self.start {
                attrs.push("isInitial=true");
            }
            if i == self.accept {
                attrs.push("isAccepting=true");
            }

            if attrs.is_empty() {
                s.push_str(&format!("  {};\n", i));
            } else {
                s.push_str(&format!("  {} [{}];\n", i, attrs.join(", ")));
            }
        }

        s.push('\n');

        // Edges
        for (from, state) in self.states.iter().enumerate() {
            for (symbol, to) in &state.transitions {
                s.push_str(&format!("  {} -> {} [label=\"{}\"];\n", from, to, symbol));
            }
        }

        s.push_str("}\n");

        s
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

