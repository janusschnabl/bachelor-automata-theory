use crate::automaton::State;
use crate::{Automaton, Result};
use std::collections::HashSet;

#[derive(Debug, Clone, Default)]
pub struct Dfa {
    pub states: Vec<State<u8>>,
    pub start: usize,
    pub accept: Vec<usize>,
    pub alphabet: HashSet<u8>,
}

impl Dfa {
    pub(crate) fn validate(&self) -> Result<()> {
        if !self.is_complete_and_deterministic() {
            return Err(crate::Error::InvalidAutomaton(
                "DFA is incomplete or non-deterministic".to_string(),
            ));
        }
        Ok(())
    }

    pub(crate) fn extract_used_symbols(&mut self) {
        let mut alphabet = HashSet::new();
        for state in &self.states {
            for (symbol, _target) in &state.transitions {
                alphabet.insert(*symbol);
            }
        }
        self.alphabet = alphabet;
    }

    fn is_complete_and_deterministic(&self) -> bool {
        for state in &self.states {
            let mut seen_symbols = HashSet::new();
            for (symbol, _target) in &state.transitions {
                if !self.alphabet.contains(symbol) {
                    return false;
                }
                if seen_symbols.contains(symbol) {
                    return false;
                }
                seen_symbols.insert(*symbol);
            }
            if seen_symbols.len() != self.alphabet.len() {
                return false;
            }
        }
        true
    }
}

impl Automaton for Dfa {
    type Label = u8;

    fn from_dot(dot: &str) -> Result<Self> {
        let mut dfa = crate::automaton::from_dot_default::<Dfa>(dot)?;
        dfa.extract_used_symbols();
        dfa.validate()?;
        Ok(dfa)
    }

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