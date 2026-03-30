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
        // STEP 1: Find the Epsilon Closure
        let ecloses: Vec<Vec<usize>> = (0..self.states.len()).map(|s| self.eclose(s)).collect();

        // STEP 2: Create New States for the NFA
        let mut reachable: HashSet<usize> = HashSet::new();
        reachable.insert(self.start);
        let mut queue: VecDeque<usize> = VecDeque::new();
        queue.push_back(self.start);

        // STEP 3: Define the Transitions
        let mut transitions: HashMap<(usize, u8), usize> = HashMap::new();

        while let Some(s) = queue.pop_front() {
            let eclose_s = &ecloses[s];
            for &e in eclose_s {
                for (sym, target) in &self.states[e].transitions {
                    if let Symbol::Byte(b) = sym {
                        let key = (s, *b);
                        if !transitions.contains_key(&key) {
                            // Follow the actual target; its epsilon closure handles the rest
                            transitions.insert(key, *target);

                            if !reachable.contains(target) {
                                reachable.insert(*target);
                                queue.push_back(*target);
                            }
                        }
                    }
                }
            }
        }

        // Map original states to new NFA states
        let mut reachable_sorted: Vec<usize> = reachable.into_iter().collect();
        reachable_sorted.sort();

        let mut state_mapping: HashMap<usize, usize> = HashMap::new();
        let mut nfa_states: Vec<State> = Vec::new();
        let mut accept_states: Vec<usize> = Vec::new();

        for (idx, &old_state) in reachable_sorted.iter().enumerate() {
            state_mapping.insert(old_state, idx);
            nfa_states.push(State {
                transitions: vec![],
            });

            // STEP 4: Set Accepting States
            if ecloses[old_state].contains(&self.accept) {
                accept_states.push(idx);
            }
        }

        for ((from, b), to) in transitions {
            if let Some(&new_from) = state_mapping.get(&from) {
                if let Some(&new_to) = state_mapping.get(&to) {
                    nfa_states[new_from]
                        .transitions
                        .push((Symbol::Byte(b), new_to));
                }
            }
        }

        let new_start = state_mapping.get(&self.start).copied().unwrap_or_default();

        Nfa {
            states: nfa_states,
            start: new_start,
            accept: accept_states,
        }
    }
}
