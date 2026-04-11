use crate::automaton::State;
use crate::epsilon_nfa::Symbol;
use crate::{Automaton, Nfa, Result};
use std::collections::{BTreeSet, HashSet, BTreeMap};

#[derive(Debug, Clone, Default)]
pub struct Dfa {
    pub states: Vec<State<Symbol>>,
    pub start: usize,
    pub accept: Vec<usize>,
    pub alphabet: HashSet<u8>,
}

impl Automaton for Dfa {
    type Label = Symbol;

    fn start_state(&self) -> usize {
        self.start
    }

    fn accept_states(&self) -> HashSet<usize> {
        self.accept.iter().copied().collect()
    }

    fn alphabet(&self) -> &HashSet<u8> {
        &self.alphabet
    }

    fn get_states(&self) -> &Vec<crate::automaton::State<Self::Label>> {
        &self.states
    }

    fn get_states_mut(&mut self) -> &mut Vec<crate::automaton::State<Self::Label>> {
        &mut self.states
    }

    fn encode_label(label: &Symbol) -> String {
        // Genbrug EpsilonNfa's implementation
        crate::epsilon_nfa::EpsilonNfa::encode_label(label)
    }

    fn decode_label(label: &str) -> Result<Symbol> {
        crate::epsilon_nfa::EpsilonNfa::decode_label(label)
    }

    fn next_states(&self, state: usize, byte: u8) -> HashSet<usize> {
        let mut next = HashSet::new();
        for (symbol, target) in &self.states[state].transitions {
            if let Symbol::Byte(b) = symbol {
                if *b == byte {
                    next.insert(*target);
                    return next;
                }
            }
        }
        next
    }

    fn set_start(&mut self, state: usize) {
        self.start = state;
    }

    fn set_accept_states(&mut self, states: HashSet<usize>) -> Result<()> {
        self.accept = states.into_iter().collect();
        Ok(())
    }
}