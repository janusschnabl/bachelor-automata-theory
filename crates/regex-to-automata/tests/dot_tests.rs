mod common;
use common::{E, b, regex_strategy};
use regex_to_automata::{EpsilonNfa, Nfa, Dfa, Automaton};
use proptest::prelude::*;


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

fn test_epsilon_nfa_roundtrip(pattern: &str) {
    let original = EpsilonNfa::from_regex(pattern, None).unwrap();
    let dot = original.to_dot();
    let reconstructed = EpsilonNfa::from_dot(&dot).unwrap();
    assert!(
        original.is_isomorphic_to(&reconstructed),
        "EpsilonNfa roundtrip failed for pattern: {pattern}"
    );
}

fn test_nfa_roundtrip(pattern: &str) {
    let original = EpsilonNfa::from_regex(pattern, None).unwrap().to_nfa();
    let dot = original.to_dot();
    let reconstructed = Nfa::from_dot(&dot).unwrap();
    assert!(
        original.is_isomorphic_to(&reconstructed),
        "Nfa roundtrip failed for pattern: {pattern}"
    );
}

fn test_dfa_roundtrip(pattern: &str) {
    let original = EpsilonNfa::from_regex(pattern, None).unwrap().to_nfa().to_dfa();
    let dot = original.to_dot();
    let reconstructed = Dfa::from_dot(&dot).unwrap();
    assert!(
        original.is_isomorphic_to(&reconstructed),
        "Dfa roundtrip failed for pattern: {pattern}"
    );
}

fn test_all_automaton_types_roundtrip(patterns: &[&str]) {
    for pattern in patterns {
        test_epsilon_nfa_roundtrip(pattern);
        test_nfa_roundtrip(pattern);
        test_dfa_roundtrip(pattern);
    }
}

proptest! {
    #[test]
    fn dot_roundtrip_all_types_preserves_structure(regex in regex_strategy()) {
        test_epsilon_nfa_roundtrip(&regex);
        test_nfa_roundtrip(&regex);
        test_dfa_roundtrip(&regex);
    }
}

#[test]
fn round_trip_preserves_structure_all_types() {
    let patterns = &["", "a", "ab", "a|b", "(a|b)*", "(ab)*c", "(a|b)*abb"];
    test_all_automaton_types_roundtrip(patterns);
}

#[test]
fn round_trip_empty_string_all_types() {
    test_all_automaton_types_roundtrip(&[""]);
}
