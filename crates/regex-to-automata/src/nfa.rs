use crate::automaton::State;
use crate::epsilon_nfa::Symbol;
use crate::{Automaton, EpsilonNfa, Result};
use crate::errors::Error;
use std::collections::HashSet;

#[derive(Debug, Clone, Default)]
pub struct Nfa {
    pub states: Vec<State<u8>>,
    pub start: usize,
    pub accept: Vec<usize>,
    pub alphabet: HashSet<u8>,
}

impl EpsilonNfa {
    pub fn to_nfa(&self) -> Nfa {
        let epsilon_closures = self.compute_epsilon_closures();
        let new_transitions_from_states: Vec<HashSet<(u8, usize)>> = self.states.iter().enumerate()
            .map(|(s,_)| self.new_transitions(s, &epsilon_closures)).collect();
        
        let accepting_states: HashSet<usize> = (0..epsilon_closures.len())
            .filter(|idx| epsilon_closures[*idx].contains(&self.accept))
            .collect();


        let nfa = self.build_nfa(new_transitions_from_states, accepting_states);
        nfa.pruned()
    }

    fn compute_epsilon_closures(&self) -> Vec<HashSet<usize>> {
        (0..self.states.len())
            .map(|s| self.epsilon_closure(s))
            .collect()
    }

    fn new_transitions(&self, state:usize, epsilon_closures: &Vec<HashSet<usize>>) -> HashSet<(u8, usize)> {
        let mut transitions: HashSet<(u8, usize)> = HashSet::new();

        for byte in self.outgoing_symbols_from_states(&epsilon_closures[state]) {
            for next_state in self.next_states(state, byte) {
                transitions.insert((byte, next_state));
            }
        }

        transitions
    }

    fn outgoing_symbols_from_states(&self, states: &HashSet<usize>) -> HashSet<u8> {
        let mut transitions: HashSet<u8> = HashSet::new();
    
        for &state in states {
            for (symbol, target) in &self.states[state].transitions {
                if let Symbol::Byte(b) = symbol {
                    transitions.insert(*b);
                }
            }
        }

        transitions       
    }

    fn build_nfa(&self, transitions_per_state: Vec<HashSet<(u8, usize)>>, accepting_states: HashSet<usize>) -> Nfa {
        let states = transitions_per_state.iter().map(|transitions| {
            let state_transitions: Vec<(u8, usize)> = transitions.iter()
                .map(|&(byte, target)| (byte, target))
                .collect();
            State { transitions: state_transitions }
        }).collect();

        Nfa {
            states,
            start: self.start,
            accept: accepting_states.into_iter().collect(),
            alphabet: self.alphabet.clone(),
        }
    }
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

// Implementer Automaton-traitten så al generisk logik virker
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
