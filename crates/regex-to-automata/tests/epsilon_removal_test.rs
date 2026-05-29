mod common;

use regex_to_automata::Automaton;
use common::{E, b};

#[test]
fn test_epsilon_transition_makes_start_accepting() {
    // Arrange
    let enfa = epsilon_nfa! {
        start: 0,
        accept: 1,
        states: [
            0 => [(E, 1)],
            1 => [],
        ]
    };
    
    let expected = nfa! {
        start: 0,
        accept: [0],
        states: [
            0 => [],
        ]
    };

    // Act
    let result = enfa.to_nfa();

    // Assert
    assert!(result.is_isomorphic_to(&expected));
}

#[test]
fn test_character_creates_both_direct_and_epsilon_closed_transitions() {
    // Arrange
    let enfa = epsilon_nfa! {
        start: 0,
        accept: 2,
        states: [
            0 => [(b(b'a'), 1)],
            1 => [(E, 2)],
            2 => [],
        ]
    };
    
    let expected = nfa! {
        start: 0,
        accept: [1, 2],
        states: [
            0 => [(b'a', 1), (b'a', 2)],
            1 => [],
            2 => [],
        ]
    };

    // Act
    let result = enfa.to_nfa();

    // Assert
    assert!(result.is_isomorphic_to(&expected));
}

#[test]
fn test_epsilon_before_character_creates_direct_transition_from_start() {
    // Arrange
    let enfa = epsilon_nfa! {
        start: 0,
        accept: 2,
        states: [
            0 => [(E, 1)],
            1 => [(b(b'a'), 2)],
            2 => [],
        ]
    };
    
    let expected = nfa! {
        start: 0,
        accept: [1],
        states: [
            0 => [(b'a', 1)],
            1 => [],
        ]
    };

    // Act
    let result = enfa.to_nfa();

    // Assert
    assert!(result.is_isomorphic_to(&expected));
}

#[test]
fn test_multiple_epsilon_branches_merge_into_single_accepting_state() {
    // Arrange
    let enfa = epsilon_nfa! {
        start: 0,
        accept: 3,
        states: [
            0 => [(E, 1), (E, 2)],
            1 => [(b(b'a'), 3)],
            2 => [(b(b'b'), 3)],
            3 => [],
        ]
    };
    
    let expected = nfa! {
        start: 0,
        accept: [1],
        states: [
            0 => [(b'a', 1), (b'b', 1)],
            1 => [],
        ]
    };

    // Act
    let result = enfa.to_nfa();

    // Assert
    assert!(result.is_isomorphic_to(&expected));
}

#[test]
fn test_multiple_epsilon_levels_create_direct_transition_to_accept() {
    // Arrange
    let enfa = epsilon_nfa! {
        start: 0,
        accept: 3,
        states: [
            0 => [(E, 1)],
            1 => [(E, 2)],
            2 => [(b(b'a'), 3)],
            3 => [],
        ]
    };
    
    let expected = nfa! {
        start: 0,
        accept: [1],
        states: [
            0 => [(b'a', 1)],
            1 => [],
        ]
    };

    // Act
    let result = enfa.to_nfa();

    // Assert
    assert!(result.is_isomorphic_to(&expected));
}

#[test]
fn test_epsilon_paths_from_different_characters_reach_accept() {
    // Arrange
    let enfa = epsilon_nfa! {
        start: 0,
        accept: 2,
        states: [
            0 => [(b(b'a'), 1), (b(b'b'), 3)],
            1 => [(E, 2)],
            2 => [],
            3 => [(E, 2)],
        ]
    };
    
    let expected = nfa! {
        start: 0,
        accept: [1, 2, 3],
        states: [
            0 => [(b'a', 1), (b'a', 2), (b'b', 3), (b'b', 2)],
            1 => [],
            2 => [],
            3 => [],
        ]
    };

    // Act
    let result = enfa.to_nfa();

    // Assert
    assert!(result.is_isomorphic_to(&expected));
}

#[test]
fn test_no_epsilon_transitions_remains_unchanged() {
    // Arrange
    let enfa = epsilon_nfa! {
        start: 0,
        accept: 2,
        states: [
            0 => [(b(b'a'), 1)],
            1 => [(b(b'b'), 2)],
            2 => [],
        ]
    };
    
    let expected = nfa! {
        start: 0,
        accept: [2],
        states: [
            0 => [(b'a', 1)],
            1 => [(b'b', 2)],
            2 => [],
        ]
    };

    // Act
    let result = enfa.to_nfa();

    // Assert
    assert!(result.is_isomorphic_to(&expected));
}

#[test]
fn test_epsilon_cycle_does_not_create_infinite_states() {
    // Arrange
    let enfa = epsilon_nfa! {
        start: 0,
        accept: 2,
        states: [
            0 => [(E, 1)],
            1 => [(E, 0), (b(b'a'), 2)],
            2 => [],
        ]
    };
    
    let expected = nfa! {
        start: 0,
        accept: [1],
        states: [
            0 => [(b'a', 1)],
            1 => [],
        ]
    };

    // Act
    let result = enfa.to_nfa();

    // Assert
    assert!(result.is_isomorphic_to(&expected));
}

#[test]
fn test_unreachable_accepting_state_is_pruned() {
    // Arrange
    let enfa = epsilon_nfa! {
        start: 0,
        accept: 3,
        states: [
            0 => [(b(b'a'), 1)],
            1 => [],
            2 => [(E, 3)],
            3 => [],
        ]
    };
    
    let expected = nfa! {
        start: 0,
        accept: [],
        states: [
            0 => [(b'a', 1)],
            1 => [],
        ]
    };

    // Act
    let result = enfa.to_nfa();

    // Assert
    assert!(result.is_isomorphic_to(&expected));
}

#[test]
fn test_same_character_from_multiple_epsilon_paths_to_same_accept() {
    // Arrange
    let enfa = epsilon_nfa! {
        start: 0,
        accept: 4,
        states: [
            0 => [(E, 1), (E, 2)],
            1 => [(b(b'a'), 3)],
            2 => [(b(b'a'), 4)],
            3 => [(E, 4)],
            4 => [],
        ]
    };
    
    let expected = nfa! {
        start: 0,
        accept: [1, 2],
        states: [
            0 => [(b'a', 1), (b'a', 2)],
            1 => [],
            2 => [],
        ]
    };

    // Act
    let result = enfa.to_nfa();

    // Assert
    assert!(result.is_isomorphic_to(&expected));
}
