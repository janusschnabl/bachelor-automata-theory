mod common;

use common::{E, b, regex_strategy};
use regex_to_automata::{EpsilonNfa, Automaton};
use proptest::prelude::*;

proptest! {
    #[test]
    fn dot_roundtrip_preserves_structure(regex in regex_strategy()) {
        // Arrange
        let nfa = EpsilonNfa::from_regex(&regex, None).unwrap();
        
        // Act
        let dot = nfa.to_dot();
        let parsed = EpsilonNfa::from_dot(&dot).unwrap();
        
        // Assert
        prop_assert!(nfa.is_isomorphic_to(&parsed));
    }
}

#[test]
fn from_dot_parses_simple_linear() {
    // Arrange
    let dot = r#"
digraph NFA {
  rankdir=LR;
  0 [isInitial=true];
  1 [isAccepting=true];
  0 -> 1 [label="a"];
}
"#;

    let expected = epsilon_nfa! {
        start: 0,
        accept: 1,
        states: [
            0 => [(b(b'a'), 1)],
            1 => [],
        ]
    };
    
    // Act
    let parsed = EpsilonNfa::from_dot(dot).unwrap();
    
    // Assert
    assert!(parsed.is_isomorphic_to(&expected));
}

#[test]
fn from_dot_parses_epsilon_transitions() {
    // Arrange
    let dot = r#"
digraph NFA {
  rankdir=LR;
  0 [isInitial=true];
  1;
  2 [isAccepting=true];
  0 -> 1 [label="ε"];
  1 -> 2 [label="a"];
}
"#;

    let expected = epsilon_nfa! {
        start: 0,
        accept: 2,
        states: [
            0 => [(E, 1)],
            1 => [(b(b'a'), 2)],
            2 => [],
        ]
    };
    
    // Act
    let parsed = EpsilonNfa::from_dot(dot).unwrap();
    
    // Assert
    assert!(parsed.is_isomorphic_to(&expected));
}
#[test]
fn from_dot_rejects_missing_start_state() {
    // Arrange
    let dot = r#"
digraph NFA {
  rankdir=LR;
  0;
  1 [isAccepting=true];
  0 -> 1 [label="a"];
}
"#;

    // Act
    let result = EpsilonNfa::from_dot(dot);
    
    // Assert
    assert!(result.is_err());
}

#[test]
fn from_dot_rejects_missing_accept_state() {
    // Arrange
    let dot = r#"
digraph NFA {
  rankdir=LR;
  0 [isInitial=true];
  1;
  0 -> 1 [label="a"];
}
"#;

    // Act
    let result = EpsilonNfa::from_dot(dot);
    
    // Assert
    assert!(result.is_err());
}

#[test]
fn enfa_from_dot_rejects_multiple_accept_states() {
    // Arrange
    let dot = r#"
digraph NFA {
  rankdir=LR;
  0 [isInitial=true];
  1 [isAccepting=true];
  2 [isAccepting=true];
  0 -> 1 [label="a"];
  0 -> 2 [label="b"];
}
"#;

    // Act
    let result = EpsilonNfa::from_dot(dot);
    
    // Assert
    assert!(result.is_err());
}


#[test]
fn round_trip_to_dot_from_dot_produces_isomorphic_automaton() {
    // Arrange
    let original = EpsilonNfa::from_regex("(a|b)*abb", None).unwrap();

    // Act
    let dot = original.to_dot();
    let reconstructed = EpsilonNfa::from_dot(&dot).unwrap();

    // Assert
    assert!(original.is_isomorphic_to(&reconstructed));
}

#[test]
fn round_trip_empty_string() {
    // Arrange
    let original = EpsilonNfa::from_regex("", None).unwrap();

    // Act
    let dot = original.to_dot();
    let reconstructed = EpsilonNfa::from_dot(&dot).unwrap();

    // Assert
    assert!(original.is_isomorphic_to(&reconstructed));
}

#[test]
fn round_trip_complex_pattern() {
    // Arrange
    let patterns = vec!["a", "ab", "a|b", "(a|b)*", "(ab)*c", "(a|b)*abb"];

    for pattern in patterns {
        let original = EpsilonNfa::from_regex(pattern, None).unwrap();

        // Act
        let dot = original.to_dot();
        let reconstructed = EpsilonNfa::from_dot(&dot).unwrap();

        // Assert
        assert!(
            original.is_isomorphic_to(&reconstructed),
            "round-trip failed for pattern: {pattern}"
        );
    }
}
