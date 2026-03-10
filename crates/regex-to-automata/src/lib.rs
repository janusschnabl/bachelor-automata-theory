pub mod errors;
pub use crate::errors::{Error, Result};

use regex_syntax::{
    Parser, ParserBuilder,
    hir::{Class, Hir, HirKind},
};
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
    pub fn from_regex(regex: &str) -> Result<Self>  {
        let ast = ParserBuilder::new()
            .unicode(false)
            .utf8(false)
            .build()
            .parse(regex)?;

        let mut nfa = EpsilonNfa::new();
        let (start, accept) = nfa.build_from_hir(&ast)?;
        nfa.start = start;
        nfa.accept = accept;

        Ok(nfa)
    }

    fn build_from_hir(&mut self, hir: &Hir) -> Result<(usize, usize)> {
        match hir.kind() {
            //Classic algorithm:
            HirKind::Literal(lit) => self.build_literal(&lit.0),
            HirKind::Alternation(subs) => self.build_alternation(subs),
            HirKind::Concat(subs) => self.build_concat(subs),
            HirKind::Repetition(rep) => self.build_repetition(rep),
            HirKind::Empty => self.build_empty(),

            //library artifacts:
            HirKind::Class(class) => self.build_class(class),
            HirKind::Capture(cap) => self.build_from_hir(&cap.sub),

            _ => Err(Error::UnsupportedFeature("unsupported HIR node")),
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
    fn build_alternation(&mut self, subs: &[Hir]) -> Result<(usize, usize)> {
        let start = self.add_state();
        let accept = self.add_state();

        for sub in subs {
            let (sub_start, sub_accept) = self.build_from_hir(sub)?;

            // start -> branch
            self.add_transition(start, Symbol::Epsilon, sub_start);

            // branch -> accept
            self.add_transition(sub_accept, Symbol::Epsilon, accept);
        }

        Ok((start, accept))
    }
    fn build_concat(&mut self, subs: &[Hir]) -> Result<(usize, usize)> {
        assert!(!subs.is_empty());

        // build first
        let (mut start, mut accept) = self.build_from_hir(&subs[0])?;

        // chain the rest
        for sub in &subs[1..] {
            let (next_start, next_accept) = self.build_from_hir(sub)?;

            // connect previous accept to next start
            self.add_transition(accept, Symbol::Epsilon, next_start);

            accept = next_accept;
        }

        Ok((start, accept))
    }
    fn build_repetition(&mut self, rep: &regex_syntax::hir::Repetition) -> Result<(usize, usize)> {
        match (rep.min, rep.max) {
            // *
            (0, None) => {
                let (sub_start, sub_accept) = self.build_from_hir(&rep.sub)?;

                let start = self.add_state();
                let accept = self.add_state();

                // ε -> sub
                self.add_transition(start, Symbol::Epsilon, sub_start);
                // ε -> accept
                self.add_transition(start, Symbol::Epsilon, accept);

                // loop
                self.add_transition(sub_accept, Symbol::Epsilon, sub_start);
                // exit
                self.add_transition(sub_accept, Symbol::Epsilon, accept);

                Ok((start, accept))
            }

            // +
            (1, None) => {
                let (sub_start, sub_accept) = self.build_from_hir(&rep.sub)?;

                let start = self.add_state();
                let accept = self.add_state();

                // must go through sub once
                self.add_transition(start, Symbol::Epsilon, sub_start);

                // loop
                self.add_transition(sub_accept, Symbol::Epsilon, sub_start);
                // exit
                self.add_transition(sub_accept, Symbol::Epsilon, accept);

                Ok((start, accept))
            }

            _ => Err(Error::UnsupportedFeature("only * and + supported for repetition")),
        }
    }
    fn build_class(&mut self, class: &Class) -> Result<(usize, usize)> {
        let start = self.add_state();
        let accept = self.add_state();

        match class {
            Class::Bytes(bytes) => {
                for range in bytes.iter() {
                    for b in range.start()..=range.end() {
                        let (s, a) = self.build_literal(&[b as u8])?;
                        self.add_transition(start, Symbol::Epsilon, s); //not sure if this branch is ever reached.
                        self.add_transition(a, Symbol::Epsilon, accept);
                    }
                }
            }
            Class::Unicode(unicode) => {
                for range in unicode.iter() {
                    let start_u = range.start() as u32;
                    let end_u = range.end() as u32;

                    // Only allow ASCII / byte-range characters
                    if end_u > 255 {
                        Err(Error::UnsupportedFeature("non-ASCII character class not supported"))?;
                    }

                    for b in start_u..=end_u {
                        let (s, a) = self.build_literal(&[b as u8])?;
                        self.add_transition(start, Symbol::Epsilon, s);
                        self.add_transition(a, Symbol::Epsilon, accept);
                    }
                }
            }
        }

        Ok((start, accept))
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




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
