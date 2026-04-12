use crate::automaton::State;
use crate::{Automaton, Nfa, Dfa};
use std::collections::{BTreeSet, HashSet, BTreeMap};

#[derive(Debug, Clone, PartialEq, Eq)]
struct SubsetConstructionResult {
    transitions: BTreeMap<BTreeSet<usize>, BTreeMap<u8, BTreeSet<usize>>>,
    initial_state: BTreeSet<usize>,
    accepting_states: HashSet<BTreeSet<usize>>,
    alphabet: HashSet<u8>,
}

impl Nfa {
    pub fn to_dfa(&self) -> Dfa {
        Dfa::subset_construction(self)
    }
}
// TODO: add tests showing we get the correct construction modulo state names
impl Dfa {
    //implements subset construction algorithm to convert an NFA to a DFA
    //NOTE: as we are making from enfa to dfa also, we are not prematurely refactoring this code until we have both versions.
    //IDEA: if we have a working enfa -> dfa, then this entire thing might simply become cast nfa as enfa then run enfa->dfa
    fn subset_construction(nfa: &Nfa) -> Dfa {
        let res = Dfa::subset_graph(nfa);
        Dfa::build_dfa_from_subsets(
            res.transitions,
            res.initial_state,
            res.accepting_states,
            res.alphabet,
        )

    }

    fn subset_graph(nfa: &Nfa) -> SubsetConstructionResult {
        let mut dfa: BTreeMap<BTreeSet<usize>, BTreeMap<u8, BTreeSet<usize>>> = BTreeMap::new();   
        let mut work_to_do= HashSet::new();
        let alphabet = nfa.alphabet().clone();
        
        work_to_do.insert(BTreeSet::from([nfa.start]));

        while let Some(current_subset) = work_to_do.iter().next().cloned() {
            work_to_do.remove(&current_subset);
            if dfa.contains_key(&current_subset) {
                continue;
            }

            let mut transitions: BTreeMap<u8, BTreeSet<usize>> = BTreeMap::new();
            for byte in &alphabet {  
                let mut next_states = BTreeSet::new();
                for state in &current_subset {
                    for next_state in nfa.next_states(*state, *byte) {
                        next_states.insert(next_state);
                    }
                }
                work_to_do.insert(next_states.clone());
                transitions.insert(*byte, next_states);
            }

            dfa.insert(current_subset, transitions);
        }

        let initial_state = BTreeSet::from([nfa.start]);
        let mut accepting_states: HashSet<BTreeSet<usize>> = HashSet::new();
        for state_set in dfa.keys() {
            if state_set.iter().any(|s| nfa.accept_states().contains(s)) {
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
    

    fn build_dfa_from_subsets(dfa: BTreeMap<BTreeSet<usize>, BTreeMap<u8, BTreeSet<usize>>>, initial_state: BTreeSet<usize>, accepting_states: HashSet<BTreeSet<usize>>, alphabet: HashSet<u8>) -> Dfa {        
        let mut state_indices: BTreeMap<BTreeSet<usize>, usize> = BTreeMap::new();
        let mut states: Vec<State<u8>> = vec![];
        let mut accept_states: Vec<usize> = vec![];

        for (i, state_set) in dfa.keys().enumerate() {
            state_indices.insert(state_set.clone(), i);
            states.push(State::new());
            if accepting_states.contains(state_set) {
                accept_states.push(i);
            }
        }

        for (state_set, transitions) in dfa {
            let from_index = state_indices[&state_set];
            for (byte, next_set) in transitions {
                let to_index = state_indices[&next_set];
                states[from_index].transitions.push((byte, to_index));
            }
        }


        let completed_dfa = Dfa {
            states: states,
            start: state_indices[&initial_state],
            accept: accept_states,
            alphabet: alphabet,
        };
        
        match completed_dfa.validate() {
            Ok(()) => completed_dfa,
            Err(e) => panic!("Constructed DFA is invalid: {}", e),
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

        literal.prop_recursive(
            4,
            16,
            2,
            |inner| {
                prop_oneof![
                    inner.clone().prop_map(|r| format!("({})*", r)),
                    inner.clone().prop_map(|r| format!("({})+", r)),
                    (inner.clone(), inner.clone())
                        .prop_map(|(a, b)| format!("{}{}", a, b)),
                    (inner.clone(), inner.clone())
                        .prop_map(|(a, b)| format!("{}|{}", a, b)),
                ]
            },
        )
    }

    proptest! {
        #[test]
        fn subset_initial_state_is_start(regex in regex_strategy()) {
            let enfa = crate::EpsilonNfa::from_regex(&regex, None).unwrap();
            let nfa = enfa.to_nfa();
            let result = Dfa::subset_graph(&nfa);

            prop_assert_eq!(result.initial_state, BTreeSet::from([nfa.start]));
        }
    }

    proptest! {
        #[test]
        fn subset_accepting_states_are_correct(regex in regex_strategy()) {
            let enfa = crate::EpsilonNfa::from_regex(&regex, None).unwrap();
            let nfa = enfa.to_nfa();
            let result = Dfa::subset_graph(&nfa);

            for subset in result.transitions.keys() {
                let should_accept =
                    subset.iter().any(|s| nfa.accept_states().contains(s));
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
            let nfa = enfa.to_nfa();
            let result = Dfa::subset_graph(&nfa);

            for (subset, transitions) in &result.transitions {
                for symbol in &result.alphabet {
                    let mut expected = BTreeSet::new();

                    for state in subset {
                        for next in nfa.next_states(*state, *symbol) {
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