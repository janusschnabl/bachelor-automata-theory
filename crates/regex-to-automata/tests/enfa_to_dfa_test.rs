mod common;

use regex_to_automata::Automaton;
use common::{E, b};

#[test]
fn simple_epsilon_nfa_converts_to_dfa() {
    // Arrange
    let mut enfa = epsilon_nfa! {
        start: 0,
        accept: 1,
        states: [
            0 => [(E, 1)],
            1 => [],
        ]
    };
    enfa.alphabet = [b'a'].iter().copied().collect();

    let mut expected_dfa = dfa! {
        start: 0,
        accept: [0],
        states: [
            0 => [(b'a', 1)],
            1 => [(b'a', 1)],
        ]
    };
    expected_dfa.alphabet = [b'a'].iter().copied().collect();

    // Act
    // Act
    let result_dfa = enfa.to_dfa();

    // Assert

    // Assert
    assert!(result_dfa.is_isomorphic_to(&expected_dfa));
}

#[test]
fn epsilon_before_character_enables_direct_transition() {
    // Arrange
    let mut enfa = epsilon_nfa! {
        start: 0,
        accept: 2,
        states: [
            0 => [(E, 1)],
            1 => [(b(b'a'), 2)],
            2 => [],
        ]
    };
    enfa.alphabet = [b'a'].iter().copied().collect();

    let mut expected_dfa = dfa! {
        start: 0,
        accept: [1],
        states: [
            0 => [(b'a', 1)],
            1 => [(b'a', 2)],
            2 => [(b'a', 2)],
        ]
    };
    expected_dfa.alphabet = [b'a'].iter().copied().collect();

    // Act
    let result_dfa = enfa.to_dfa();

    // Assert
    assert!(result_dfa.is_isomorphic_to(&expected_dfa));
}

#[test]
fn epsilon_nondeterminism_creates_subset() {
    // Arrange
    let mut enfa = epsilon_nfa! {
        start: 0,
        accept: 2,
        states: [
            0 => [(E, 1), (b(b'a'), 2)],
            1 => [(b(b'a'), 3)],
            2 => [],
            3 => [],
        ]
    };
    enfa.alphabet = [b'a'].iter().copied().collect();

    let mut expected_dfa = dfa! {
        start: 0,
        accept: [1],
        states: [
            0 => [(b'a', 1)],
            1 => [(b'a', 2)],
            2 => [(b'a', 2)],
        ]
    };
    expected_dfa.alphabet = [b'a'].iter().copied().collect();

    // Act
    let result_dfa = enfa.to_dfa();

    // Assert
    assert!(result_dfa.is_isomorphic_to(&expected_dfa));
}

#[test]
fn multiple_epsilon_branches_merge() {
    // Arrange
    let mut enfa = epsilon_nfa! {
        start: 0,
        accept: 3,
        states: [
            0 => [(b(b'a'), 1), (b(b'a'), 2)],
            1 => [(E, 3)],
            2 => [(E, 3)],
            3 => [],
        ]
    };
    enfa.alphabet = [b'a'].iter().copied().collect();

    let mut expected_dfa = dfa! {
        start: 0,
        accept: [1],
        states: [
            0 => [(b'a', 1)],
            1 => [(b'a', 2)],
            2 => [(b'a', 2)],
        ]
    };
    expected_dfa.alphabet = [b'a'].iter().copied().collect();

    // Act
    let result_dfa = enfa.to_dfa();

    // Assert
    assert!(result_dfa.is_isomorphic_to(&expected_dfa));
}

#[test]
fn epsilon_cycles_handled_correctly() {
    // Arrange
    let mut enfa = epsilon_nfa! {
        start: 0,
        accept: 2,
        states: [
            0 => [(E, 1)],
            1 => [(E, 0), (b(b'a'), 2)],
            2 => [],
        ]
    };
    enfa.alphabet = [b'a'].iter().copied().collect();

    let mut expected_dfa = dfa! {
        start: 0,
        accept: [1],
        states: [
            0 => [(b'a', 1)],
            1 => [(b'a', 2)],
            2 => [(b'a', 2)],
        ]
    };
    expected_dfa.alphabet = [b'a'].iter().copied().collect();

    // Act
    let result_dfa = enfa.to_dfa();

    // Assert
    assert!(result_dfa.is_isomorphic_to(&expected_dfa));
}

#[test]
fn epsilon_and_byte_on_same_state() {
    // Arrange
    let mut enfa = epsilon_nfa! {
        start: 0,
        accept: 2,
        states: [
            0 => [(E, 1), (b(b'a'), 2)],
            1 => [],
            2 => [],
        ]
    };
    enfa.alphabet = [b'a'].iter().copied().collect();

    let mut expected_dfa = dfa! {
        start: 0,
        accept: [1],
        states: [
            0 => [(b'a', 1)],
            1 => [(b'a', 2)],
            2 => [(b'a', 2)],
        ]
    };
    expected_dfa.alphabet = [b'a'].iter().copied().collect();

    // Act
    let result_dfa = enfa.to_dfa();

    // Assert
    assert!(result_dfa.is_isomorphic_to(&expected_dfa));
}

#[test]
fn multi_symbol_with_epsilon_transitions() {
    // Arrange
    let mut enfa = epsilon_nfa! {
        start: 0,
        accept: 3,
        states: [
            0 => [(E, 1)],
            1 => [(b(b'a'), 2)],
            2 => [(E, 3)],
            3 => [],
        ]
    };
    enfa.alphabet = [b'a', b'b'].iter().copied().collect();

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

    let result_dfa = enfa.to_dfa();

    // Assert
    assert!(result_dfa.is_isomorphic_to(&expected_dfa));
}

#[test]
fn epsilon_creates_accepting_subset_from_nondeterminism() {
    // Arrange
    let mut enfa = epsilon_nfa! {
        start: 0,
        accept: 3,
        states: [
            0 => [(b(b'a'), 1), (b(b'a'), 2)],
            1 => [(E, 3)],
            2 => [],
            3 => [],
        ]
    };
    enfa.alphabet = [b'a'].iter().copied().collect();

    let mut expected_dfa = dfa! {
        start: 0,
        accept: [1],
        states: [
            0 => [(b'a', 1)],
            1 => [(b'a', 2)],
            2 => [(b'a', 2)],
        ]
    };
    expected_dfa.alphabet = [b'a'].iter().copied().collect();

    // Act
    let result_dfa = enfa.to_dfa();

    // Assert
    assert!(result_dfa.is_isomorphic_to(&expected_dfa));
}

#[test]
fn nested_epsilon_transitions() {
    // Arrange
    let mut enfa = epsilon_nfa! {
        start: 0,
        accept: 4,
        states: [
            0 => [(E, 1)],
            1 => [(E, 2)],
            2 => [(E, 3)],
            3 => [(b(b'a'), 4)],
            4 => [],
        ]
    };
    enfa.alphabet = [b'a'].iter().copied().collect();

    let mut expected_dfa = dfa! {
        start: 0,
        accept: [1],
        states: [
            0 => [(b'a', 1)],
            1 => [(b'a', 2)],
            2 => [(b'a', 2)],
        ]
    };
    expected_dfa.alphabet = [b'a'].iter().copied().collect();

    // Act
    let result_dfa = enfa.to_dfa();

    // Assert
    assert!(result_dfa.is_isomorphic_to(&expected_dfa));
}


