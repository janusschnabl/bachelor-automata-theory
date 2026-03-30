use regex_syntax::ast::print;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::{EpsilonNfa, State, Symbol};

pub struct Nfa {
    pub states: Vec<State>,
    pub start: usize,
    pub accept: Vec<usize>,
}

impl EpsilonNfa {
    pub fn eclose(&self, state: usize) -> Vec<usize> {
        let mut visited = vec![false; self.states.len()];
        let mut stack = vec![state];
        visited[state] = true;

        while let Some(s) = stack.pop() {
            for (sym, target) in &self.states[s].transitions {
                if *sym == Symbol::Epsilon && !visited[*target] {
                    visited[*target] = true;
                    stack.push(*target);
                }
            }
        }

        visited
            .into_iter()
            .enumerate()
            .filter_map(|(i, v)| if v { Some(i) } else { None })
            .collect()
    }

    pub fn to_nfa(&self) -> Nfa {
        // Subset construction: build states as sets of epsilon-NFA states
        let mut state_map: HashMap<Vec<usize>, usize> = HashMap::new();
        let mut nfa_states: Vec<State> = Vec::new();
        let mut accept_states: Vec<usize> = Vec::new();
        let mut queue: VecDeque<Vec<usize>> = VecDeque::new();

        // Start with epsilon closure of the initial state
        let start_set = self.eclose(self.start);
        queue.push_back(start_set.clone());
        state_map.insert(start_set.clone(), 0);
        nfa_states.push(State {
            transitions: vec![],
        });

        // Check if start state is accepting
        if start_set.contains(&self.accept) {
            accept_states.push(0);
        }

        // Process all discovered states
        while let Some(current_set) = queue.pop_front() {
            let current_idx = state_map[&current_set];

            // Find all symbols reachable from this set
            let mut symbol_targets: HashMap<u8, HashSet<usize>> = HashMap::new();

            for &state in &current_set {
                for (sym, target) in &self.states[state].transitions {
                    if let Symbol::Byte(b) = sym {
                        symbol_targets.entry(*b).or_insert_with(HashSet::new);
                        // Add all states in the epsilon closure of target
                        for &s in &self.eclose(*target) {
                            symbol_targets.get_mut(b).unwrap().insert(s);
                        }
                    }
                }
            }

            // Create transitions for each symbol
            for (b, target_set) in symbol_targets {
                let mut target_vec: Vec<usize> = target_set.into_iter().collect();
                target_vec.sort();

                // Check if this target set already exists
                if !state_map.contains_key(&target_vec) {
                    let new_idx = nfa_states.len();
                    state_map.insert(target_vec.clone(), new_idx);
                    nfa_states.push(State {
                        transitions: vec![],
                    });
                    queue.push_back(target_vec.clone());

                    // Check if new state is accepting
                    if target_vec.contains(&self.accept) {
                        accept_states.push(new_idx);
                    }
                }

                let target_idx = state_map[&target_vec];
                nfa_states[current_idx]
                    .transitions
                    .push((Symbol::Byte(b), target_idx));
            }
        }

        Nfa {
            states: nfa_states,
            start: 0,
            accept: accept_states,
        }
    }
}
