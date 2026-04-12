mod common;

use regex_to_automata::Automaton;

#[test]
fn simple_nfa_converts_to_isomorphic_dfa() {
    // Arrange
    let mut nfa = nfa! {
        start: 0,
        accept: [1],
        states: [
            0 => [(b'a', 1)],
            1 => [],
        ]
    };
    nfa.alphabet = [b'a', b'b'].iter().copied().collect();

    let mut expected_dfa = dfa! {
        start: 0,
        accept: [1],
        states: [
            0 => [(b'a', 1), (b'b', 2)],
            1 => [(b'a', 2), (b'b', 2)],
            2 => [(b'a', 2), (b'b', 2)],
        ]
    };
    expected_dfa.alphabet = [b'a', b'b'].iter().copied().collect();

    // Act
    let result_dfa = nfa.to_dfa();

    // Assert
    assert!(result_dfa.is_isomorphic_to(&expected_dfa));
}

#[test]
fn nondeterministic_transitions_merged_in_conversion() {
    // Arrange
    let mut nfa = nfa! {
        start: 0,
        accept: [2],
        states: [
            0 => [(b'a', 1), (b'a', 2)],
            1 => [],
            2 => [],
        ]
    };
    nfa.alphabet = [b'a', b'b'].iter().copied().collect();

    let mut expected_dfa = dfa! {
        start: 0,
        accept: [1],
        states: [
            0 => [(b'a', 1), (b'b', 2)],
            1 => [(b'a', 2), (b'b', 2)],
            2 => [(b'a', 2), (b'b', 2)],
        ]
    };
    expected_dfa.alphabet = [b'a', b'b'].iter().copied().collect();

    // Act
    let result_dfa = nfa.to_dfa();

    // Assert
    assert!(result_dfa.is_isomorphic_to(&expected_dfa));
}

#[test]
fn accepting_states_correctly_identified_in_subsets() {
    // Arrange
    let mut nfa = nfa! {
        start: 0,
        accept: [2, 3],
        states: [
            0 => [(b'a', 1), (b'a', 2)],
            1 => [(b'b', 3)],
            2 => [],
            3 => [],
        ]
    };
    nfa.alphabet = [b'a', b'b'].iter().copied().collect();

    let mut expected_dfa = dfa! {
        start: 0,
        accept: [1, 2],
        states: [
            0 => [(b'a', 1), (b'b', 3)],
            1 => [(b'a', 3), (b'b', 2)],
            2 => [(b'a', 3), (b'b', 3)],
            3 => [(b'a', 3), (b'b', 3)],
        ]
    };
    expected_dfa.alphabet = [b'a', b'b'].iter().copied().collect();

    // Act
    let result_dfa = nfa.to_dfa();

    // Assert
    assert!(result_dfa.is_isomorphic_to(&expected_dfa));
}

#[test]
fn nondeterminism_resolved_through_subset_exploration() {
    // Arrange: 
    let mut nfa = nfa! {
        start: 0,
        accept: [3],
        states: [
            0 => [(b'a', 1), (b'a', 2)],
            1 => [(b'b', 3)],
            2 => [(b'b', 3)],
            3 => [],
        ]
    };
    nfa.alphabet = [b'a', b'b'].iter().copied().collect();

    let mut expected_dfa = dfa! {
        start: 0,
        accept: [2],
        states: [
            0 => [(b'a', 1), (b'b', 3)],
            1 => [(b'a', 3), (b'b', 2)],
            2 => [(b'a', 3), (b'b', 3)],
            3 => [(b'a', 3), (b'b', 3)],
        ]
    };
    expected_dfa.alphabet = [b'a', b'b'].iter().copied().collect();

    // Act
    let result_dfa = nfa.to_dfa();

    // Assert
    assert!(result_dfa.is_isomorphic_to(&expected_dfa));
}

#[test]
fn result_is_deterministic_and_complete() {
    // Arrange:
    let mut nfa = nfa! {
        start: 0,
        accept: [1],
        states: [
            0 => [(b'a', 1), (b'a', 0)],  
            1 => [(b'b', 0),(b'b', 1)],
        ]
    };
    nfa.alphabet = [b'a', b'b'].iter().copied().collect();

    // Act
    let result_dfa = nfa.to_dfa();

    // Assert: DFA should have exactly one transition per symbol from each reachable state
    let alphabet = result_dfa.alphabet();
    for state_idx in 0..result_dfa.get_states().len() {
        let mut seen_symbols = std::collections::HashSet::new();
        for (symbol, _target) in &result_dfa.get_states()[state_idx].transitions {
            assert!(
                !seen_symbols.contains(symbol),
                "DFA has duplicate transitions for symbol {:?} from state {}",
                symbol,
                state_idx
            );
            assert!(
                alphabet.contains(symbol),
                "DFA has transition for symbol not in alphabet from state {}",
                state_idx
            );
            seen_symbols.insert(*symbol);
        }
        assert_eq!(
            seen_symbols.len(),
            alphabet.len(),
            "DFA state {} is incomplete: has {} transitions but alphabet has {} symbols",
            state_idx,
            seen_symbols.len(),
            alphabet.len()
        );
    }
}
