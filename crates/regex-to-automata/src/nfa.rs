use crate::automaton::State;
use crate::epsilon_nfa::Symbol;
use crate::{Automaton, EpsilonNfa, Error, Result};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Default)]
pub struct Nfa {
    pub states: Vec<State<Symbol>>,
    pub start: usize,
    pub accept: Vec<usize>,
}

impl EpsilonNfa {
    pub fn to_nfa(&self) -> Nfa {
        let ecloses = self.compute_epsilon_closures();
        let reachable = self.find_reachable_states(&ecloses);
        let (state_mapping, mut nfa_states, accept_states) =
            self.build_state_mapping(&ecloses, &reachable);
        self.build_transitions(&ecloses, &state_mapping, &mut nfa_states);

        Nfa {
            states: nfa_states,
            start: state_mapping.get(&self.start).copied().unwrap_or(0),
            accept: accept_states,
        }
    }

    fn compute_epsilon_closures(&self) -> Vec<HashSet<usize>> {
        (0..self.states.len())
            .map(|s| self.epsilon_closure(s))
            .collect()
    }

    fn find_reachable_states(&self, ecloses: &[HashSet<usize>]) -> Vec<bool> {
        let mut reachable = vec![false; self.states.len()];
        let mut queue = vec![self.start];
        reachable[self.start] = true;

        let mut i = 0;
        while i < queue.len() {
            let s = queue[i];
            i += 1;
            for &e in &ecloses[s] {
                for (sym, target) in &self.states[e].transitions {
                    if let Symbol::Byte(_) = sym {
                        if !reachable[*target] {
                            reachable[*target] = true;
                            queue.push(*target);
                        }
                    }
                }
            }
        }
        reachable
    }

    fn build_state_mapping(
        &self,
        ecloses: &[HashSet<usize>],
        reachable: &[bool],
    ) -> (HashMap<usize, usize>, Vec<State<Symbol>>, Vec<usize>) {
        let mut state_mapping = HashMap::new();
        let mut nfa_states = Vec::new();
        let mut accept_states = Vec::new();

        for (old_idx, &is_reachable) in reachable.iter().enumerate() {
            if is_reachable {
                let new_idx = nfa_states.len();
                state_mapping.insert(old_idx, new_idx);
                nfa_states.push(State::new());

                if ecloses[old_idx].contains(&self.accept) {
                    accept_states.push(new_idx);
                }
            }
        }
        (state_mapping, nfa_states, accept_states)
    }

    fn build_transitions(
        &self,
        ecloses: &[HashSet<usize>],
        state_mapping: &HashMap<usize, usize>,
        nfa_states: &mut Vec<State<Symbol>>,
    ) {
        for (&old_idx, &new_idx) in state_mapping {
            let mut symbol_targets: HashMap<u8, HashSet<usize>> = HashMap::new();

            for &e in &ecloses[old_idx] {
                for (sym, target) in &self.states[e].transitions {
                    if let Symbol::Byte(b) = sym {
                        symbol_targets
                            .entry(*b)
                            .or_default()
                            .extend(&ecloses[*target]);
                    }
                }
            }

            for (b, targets) in symbol_targets {
                for target in targets {
                    if let Some(&new_target) = state_mapping.get(&target) {
                        nfa_states[new_idx]
                            .transitions
                            .push((Symbol::Byte(b), new_target));
                    }
                }
            }
        }
    }
}

// Implementer Automaton-traitten så al generisk logik virker
impl Automaton for Nfa {
    type Label = Symbol;

    fn start_state(&self) -> usize {
        self.start
    }

    fn accept_states(&self) -> HashSet<usize> {
        self.accept.iter().copied().collect()
    }

    fn alphabet(&self) -> &HashSet<u8> {
        // NFA har ikke et eksplicit alfabet-felt som EpsilonNfa,
        // så vi returnerer en tom reference — overvej at tilføje feltet
        unimplemented!("Nfa mangler alphabet-felt — tilføj `pub alphabet: HashSet<u8>`")
    }

    fn get_states(&self) -> &Vec<State<Symbol>> {
        &self.states
    }

    fn get_states_mut(&mut self) -> &mut Vec<State<Symbol>> {
        &mut self.states
    }

    fn encode_label(label: &Symbol) -> String {
        // Genbrug EpsilonNfa's implementation
        crate::epsilon_nfa::EpsilonNfa::encode_label(label) // eller kopier logikken
    }

    fn decode_label(label: &str) -> Result<Symbol> {
        crate::epsilon_nfa::EpsilonNfa::decode_label(label)
    }

    fn next_states(&self, state: usize, byte: u8) -> HashSet<usize> {
        self.states[state]
            .transitions
            .iter()
            .filter_map(|(sym, target)| {
                if let Symbol::Byte(b) = sym {
                    if *b == byte { Some(*target) } else { None }
                } else {
                    None
                }
            })
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
