mod common;

use regex_to_automata::{EpsilonNfa, Automaton};
use common::{E, b, regex_strategy};
use proptest::prelude::*;


#[test]
fn dfa_construction_path_matters_for_specific_enfa() {
    // Arrange
    let mut enfa = epsilon_nfa! {
        start: 0,
        accept: 1,
        states: [
            0 => [(E, 1),(b(b'a'), 0)],
            1 => [(b(b'a'), 1)]
        ]
    };
    enfa.alphabet = [b'a'].iter().copied().collect();

    // Act
    let long = enfa.to_nfa().to_dfa();
    let direct = enfa.to_dfa();

    // Assert
    assert!(!long.is_isomorphic_to(&direct));
}



proptest! {
    #[test]
    fn dfa_construction_path_doesnt_matter_for_random_regex(
        regex in regex_strategy()
    ) {
        // Act
        let enfa = EpsilonNfa::from_regex(&regex, None).unwrap();

        // Act - Path 1: ENFA -> NFA -> DFA
        let nfa = enfa.to_nfa();
        let dfa_via_nfa = nfa.to_dfa();

        // Act - Path 2: ENFA -> DFA (direct)
        let dfa_direct = enfa.to_dfa();

        // Assert: Both conversion paths should produce isomorphic DFAs
        prop_assert!(dfa_direct.is_isomorphic_to(&dfa_via_nfa));
    }
}
