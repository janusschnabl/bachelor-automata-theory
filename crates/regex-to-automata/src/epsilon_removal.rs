use crate::automaton::State;
use crate::epsilon_nfa::Symbol;
use crate::{Automaton, EpsilonNfa, Nfa};
use std::collections::HashSet;

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
