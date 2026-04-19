use crate::nfa_to_dfa::SubsetConstructionResult;
use crate::{Automaton, Dfa, EpsilonNfa};
use std::collections::{BTreeMap, BTreeSet, HashSet};

impl EpsilonNfa {
    pub fn to_dfa(&self) -> Dfa {
        Dfa::subset_construction_enfa(self)
    }
}

impl Dfa {
    // Implements subset construction algorithm to convert ENFA to complete DFA
    pub fn subset_construction_enfa(enfa: &EpsilonNfa) -> Dfa {
        let res = Dfa::subset_graph_enfa(enfa);
        Dfa::build_dfa_from_subsets(
            res.transitions,
            res.initial_state,
            res.accepting_states,
            res.alphabet,
        )
    }

    fn subset_graph_enfa(enfa: &EpsilonNfa) -> SubsetConstructionResult {
        let mut dfa: BTreeMap<BTreeSet<usize>, BTreeMap<u8, BTreeSet<usize>>> = BTreeMap::new();
        let mut work_to_do = HashSet::new();
        let alphabet = enfa.alphabet().clone();

        // Start with epsilon-closure of the initial state
        let initial_state: BTreeSet<usize> = enfa
            .epsilon_closure(enfa.start_state())
            .into_iter()
            .collect();
        work_to_do.insert(initial_state.clone());

        while let Some(current_subset) = work_to_do.iter().next().cloned() {
            work_to_do.remove(&current_subset);
            if dfa.contains_key(&current_subset) {
                continue;
            }

            let mut transitions: BTreeMap<u8, BTreeSet<usize>> = BTreeMap::new();
            for byte in &alphabet {
                let mut next_states = BTreeSet::new();
                for state in &current_subset {
                    for next in enfa.next_states(*state, *byte) {
                        next_states.insert(next);
                    }
                }
                work_to_do.insert(next_states.clone());
                transitions.insert(*byte, next_states);
            }

            dfa.insert(current_subset, transitions);
        }

        let mut accepting_states: HashSet<BTreeSet<usize>> = HashSet::new();
        let accept_set = enfa.accept_states();
        for state_set in dfa.keys() {
            if state_set.iter().any(|s| accept_set.contains(s)) {
                accepting_states.insert(state_set.clone());
            }
        }

        SubsetConstructionResult {
            transitions: dfa,
            initial_state,
            accepting_states,
            alphabet,
        }
    }
}
