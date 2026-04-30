use crate::automaton::State;
use crate::epsilon_nfa::Symbol;
use crate::{Automaton, Result};
use crate::errors::Error;
use std::collections::HashSet;

#[derive(Debug, Clone, Default)]
pub struct Nfa {
    pub states: Vec<State<u8>>,
    pub start: usize,
    pub accept: Vec<usize>,
    pub alphabet: HashSet<u8>,
}

impl Nfa {
    pub fn pruned(&self) -> Nfa {
        let reachable = self.reachable_states();

        let mut idx_mapping = vec![None; self.states.len()];
        let mut new_states = Vec::new();

        for old_idx in 0..self.states.len() {
            if reachable.contains(&old_idx) {
                let new_idx = new_states.len();
                idx_mapping[old_idx] = Some(new_idx);
                new_states.push(State::new());
            }
        }

        for old_from in 0..self.states.len() {
            let Some(new_from) = idx_mapping[old_from] else {
                continue;
            };

            for (sym, old_to) in &self.states[old_from].transitions {
                let Some(new_to) = idx_mapping[*old_to] else {
                    panic!("Transition to unreachable state should have been filtered out by reachable_states()")
                };

                new_states[new_from]
                    .transitions
                    .push((*sym, new_to));
            }
        }

        let new_accept = self
            .accept
            .iter()
            .filter_map(|old_idx| idx_mapping[*old_idx])
            .collect();

        let new_start = idx_mapping[self.start]
            .expect("start state should always be reachable");

        Nfa {
            states: new_states,
            start: new_start,
            accept: new_accept,
            alphabet: self.alphabet.clone(),
        }
    }

    fn reachable_states(&self) -> HashSet<usize> {
        let mut reachable = HashSet::new();
        let mut stack = vec![self.start];

        while let Some(state) = stack.pop() {
            if reachable.insert(state) {
                for (_, target) in &self.states[state].transitions {
                    stack.push(*target);
                }
            }
        }

        reachable
    }
}

impl Automaton for Nfa {
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

    fn get_states(&self) -> &Vec<State<Self::Label>> {
        &self.states
    }

    fn get_states_mut(&mut self) -> &mut Vec<State<Self::Label>> {
        &mut self.states
    }

    fn encode_label(label: &Self::Label) -> String {
        let symbol = Symbol::Byte(*label);
        crate::epsilon_nfa::EpsilonNfa::encode_label(&symbol)
    }

    fn decode_label(label: &str) -> Result<Self::Label> {
        let symbol = crate::epsilon_nfa::EpsilonNfa::decode_label(label)?;
        if let Symbol::Byte(b) = symbol {
            Ok(b)
        } else {
            Err(Error::InvalidInput(format!("invalid label for NFA: {label}")))
        }
    }

    fn next_states(&self, state: usize, byte: u8) -> HashSet<usize> {
        self.states[state]
            .transitions
            .iter()
            .filter_map(|(sym, target)| 
               if *sym == byte { Some(*target) } 
               else { None })
            .collect()
    }

    fn set_start(&mut self, state: usize) {
        self.start = state;
    }

    fn set_accept_states(&mut self, states: HashSet<usize>) -> Result<()> {
        self.accept = states.into_iter().collect();
        Ok(())
    }
}
