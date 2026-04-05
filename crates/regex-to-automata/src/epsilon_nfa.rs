use crate::{Automaton, Error, Result};
use std::collections::HashSet;
use std::fmt;

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
    pub fn new() -> Self {
        Self::default()
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

    pub(crate) fn add_state(&mut self) -> usize {
        let id = self.states.len();
        self.states.push(State {
            transitions: vec![],
        });
        id
    }

    pub(crate) fn add_transition(&mut self, from: usize, symbol: Symbol, to: usize) {
        self.states[from].transitions.push((symbol, to));
    }

    pub(crate) fn extract_used_symbols(&self) -> HashSet<u8> {
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
        let closure = self.epsilon_closure(state); // Epsilon-close the source first
        for s in closure {
            for (symbol, target) in &self.states[s].transitions {
                if let Symbol::Byte(b) = symbol {
                    if *b == byte {
                        next.extend(self.epsilon_closure(*target)); // Epsilon-close the target
                    }
                }
            }
        }
        next
    }

    fn accepts(&self, word: &str) -> bool {
        let initial_states = self.epsilon_closure(self.start);
        crate::automaton::accepts_from_states(self, &initial_states, word)
    }
}
