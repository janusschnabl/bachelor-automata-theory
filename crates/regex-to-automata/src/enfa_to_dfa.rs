use crate::epsilon_nfa::Symbol;
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
                let mut targets = BTreeSet::new();

                for state in &current_subset {
                    for (symbol, target) in &enfa.states[*state].transitions {
                        if let Symbol::Byte(b) = symbol {
                            if *b == *byte {
                                targets.insert(*target);
                            }
                        }
                    }
                }

                let next_states: BTreeSet<usize> = targets
                    .iter()
                    .flat_map(|t| enfa.epsilon_closure(*t))
                    .collect();

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



#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::BTreeSet;

    fn regex_strategy() -> impl Strategy<Value = String> {
        let literal = prop_oneof![
            Just("a".to_string()),
            Just("b".to_string()),
            Just("c".to_string()),
        ];

        literal.prop_recursive(4, 16, 2, |inner| {
            prop_oneof![
                inner.clone().prop_map(|r| format!("({})*", r)),
                inner.clone().prop_map(|r| format!("({})+", r)),
                (inner.clone(), inner.clone()).prop_map(|(a, b)| format!("{}{}", a, b)),
                (inner.clone(), inner.clone()).prop_map(|(a, b)| format!("{}|{}", a, b)),
            ]
        })
    }

    proptest! {
        #[test]
        fn subset_initial_state_is_start(regex in regex_strategy()) {
            let enfa = crate::EpsilonNfa::from_regex(&regex, None).unwrap();
            let result = Dfa::subset_graph_enfa(&enfa);
            let epsilon_closure_start: BTreeSet<usize> = enfa
                .epsilon_closure(enfa.start_state())
                .into_iter()
                .collect();

            prop_assert_eq!(result.initial_state, epsilon_closure_start);
        }
    }

    proptest! {
        #[test]
        fn subset_accepting_states_are_correct(regex in regex_strategy()) {
            let enfa = crate::EpsilonNfa::from_regex(&regex, None).unwrap();
            let result = Dfa::subset_graph_enfa(&enfa);

            for subset in result.transitions.keys() {
                let should_accept =
                    subset.iter().any(|s| enfa.accept_states().contains(s));
                let is_accepting =
                    result.accepting_states.contains(subset);

                prop_assert_eq!(
                    is_accepting,
                    should_accept,
                    "Wrong accepting status for subset {:?}",
                    subset
                );
            }
        }
    }

    proptest! {
        #[test]
        fn subset_transitions_match_nfa_union(regex in regex_strategy()) {
            let enfa = crate::EpsilonNfa::from_regex(&regex, None).unwrap();
            let result = Dfa::subset_graph_enfa(&enfa);

            for (subset, transitions) in &result.transitions {
                for symbol in &result.alphabet {
                    let mut expected = BTreeSet::new();

                    for state in subset {
                        for next in enfa.next_states(*state, *symbol) {
                            expected.insert(next);
                        }
                    }

                    prop_assert_eq!(
                        transitions.get(symbol),
                        Some(&expected),
                        "Mismatch for subset {:?} on symbol {:?}",
                        subset,
                        *symbol as char
                    );
                }
            }
        }
    }
}
