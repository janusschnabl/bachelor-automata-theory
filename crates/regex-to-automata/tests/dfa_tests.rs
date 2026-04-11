use regex_to_automata::{Dfa, Automaton};

#[test]
fn from_dot_accepts_valid_dfa() {
    // Arrange
    let dot = r#"
        digraph DFA {
        rankdir=LR;
        0 [isInitial=true];
        1 [isAccepting=true];
        0 -> 1 [label="a"];
        0 -> 0 [label="b"];
        1 -> 0 [label="a"];
        1 -> 1 [label="b"];
        }
        "#;

    // Act
    let result = Dfa::from_dot(dot);

    // Assert
    assert!(result.is_ok(), "Valid DFA should parse without error: {:?}", result);
    let dfa = result.unwrap();
    assert_eq!(dfa.start, 0);
    assert!(dfa.accept.contains(&1));
}

#[test]
fn from_dot_rejects_incomplete_dfa() {
    // Arrange: DFA that is missing a transition (state 0 has no transition for 'b')
    let dot = r#"
        digraph DFA {
        rankdir=LR;
        0 [isInitial=true];
        1 [isAccepting=true];
        0 -> 1 [label="a"];
        }
        "#;

    // Act
    let result = Dfa::from_dot(dot);

    // Assert
    assert!(
        result.is_err(),
        "Incomplete DFA should fail validation"
    );
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("incomplete"),
        "Error message should indicate incomplete DFA: {}",
        err
    );
}

#[test]
fn from_dot_rejects_multiple_transitions_same_symbol() {
    // Arrange: DFA with multiple transitions on the same symbol from one state
    // This violates the determinism requirement and should be caught during validation
    let dot = r#"
        digraph DFA {
        rankdir=LR;
        0 [isInitial=true];
        1 [isAccepting=true];
        2 [isAccepting=true];
        0 -> 1 [label="a"];
        0 -> 2 [label="a"];
        0 -> 0 [label="b"];
        1 -> 0 [label="a"];
        1 -> 1 [label="b"];
        2 -> 0 [label="a"];
        2 -> 2 [label="b"];
        }
        "#;

    // Act
    let result = Dfa::from_dot(dot);

    // Assert
    assert!(
        result.is_err(),
        "DFA with multiple transitions on same symbol should fail validation"
    );
}

#[test]
fn from_dot_rejects_missing_transitions_on_alphabet() {
    // Arrange: DFA that declares alphabet symbols but doesn't have transitions for all
    let dot = r#"
        digraph DFA {
        rankdir=LR;
        0 [isInitial=true, alphabet="a,b"];
        1 [alphabet="a,b"];
        0 -> 1 [label="a"];
        }
        "#;

    // Act
    let result = Dfa::from_dot(dot);

    // Assert
    assert!(
        result.is_err(),
        "DFA missing transitions should fail validation"
    );
}

#[test]
fn from_dot_accepts_single_state_dfa() {
    // Arrange: Minimal valid DFA - single state that accepts all strings
    let dot = r#"
        digraph DFA {
        rankdir=LR;
        0 [isInitial=true, isAccepting=true];
        0 -> 0 [label="a"];
        0 -> 0 [label="b"];
        }
        "#;

    // Act
    let result = Dfa::from_dot(dot);

    // Assert
    assert!(result.is_ok(), "Single state complete DFA should parse without error");
    let dfa = result.unwrap();
    assert_eq!(dfa.start, 0);
    assert!(dfa.accept.contains(&0));
}

#[test]
fn from_dot_complex_valid_dfa() {
    // Arrange: More complex valid DFA with 3 states
    let dot = r#"
        digraph DFA {
        rankdir=LR;
        0 [isInitial=true];
        2 [isAccepting=true];
        0 -> 1 [label="a"];
        0 -> 0 [label="b"];
        1 -> 2 [label="a"];
        1 -> 0 [label="b"];
        2 -> 0 [label="a"];
        2 -> 2 [label="b"];
        }
        "#;

    // Act
    let result = Dfa::from_dot(dot);

    // Assert
    assert!(result.is_ok(), "Valid 3-state DFA should parse without error");
    let dfa = result.unwrap();
    assert_eq!(dfa.states.len(), 3);
    assert_eq!(dfa.start, 0);
}
