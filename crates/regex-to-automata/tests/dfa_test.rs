mod common;

use regex_to_automata::Automaton;

#[test]
fn test_dfa_to_dot_from_dot_roundtrip_fails_on_incomplete_dfa() {
    // Act
    let incomplete_dfa = dfa! {
        start: 0,
        accept: [1],
        states: [
            0 => [(b'a', 1)],
            1 => [(b'a', 0), (b'b', 1)],
        ]
    };

    // Act
    let dot = incomplete_dfa.to_dot();
    let result = regex_to_automata::Dfa::from_dot(&dot);

    // Assert
    assert!(result.is_err(), "Expected from_dot to fail for incomplete DFA");
}
