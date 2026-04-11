use crate::automaton::State;
use crate::epsilon_nfa::Symbol;
use crate::{Automaton, Result};
use std::collections::HashSet;

#[derive(Debug, Clone, Default)]
pub struct Dfa {
    pub states: Vec<State<u8>>,
    pub start: usize,
    pub accept: Vec<usize>,
    pub alphabet: HashSet<u8>,
}

impl Automaton for Dfa {
    type Label = u8;

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

    fn encode_label(label: &Self::Label) -> String {
        crate::nfa::Nfa::encode_label(label)
    }

    fn decode_label(label: &str) -> Result<Self::Label> {
        crate::nfa::Nfa::decode_label(label)
    }

    fn next_states(&self, state: usize, byte: Self::Label) -> HashSet<usize> {
        let mut next = HashSet::new();
        for (b, target) in &self.states[state].transitions {
            if *b == byte {
                next.insert(*target);
                return next;
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