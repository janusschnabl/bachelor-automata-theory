mod common;

use regex_to_automata::Automaton;

#[test]
fn test_nfa_pruned_removes_unreachable_states() {
    //Arrange
    let nfa_with_unreachable = nfa! {
        start: 0,
        accept: [1],
        states: [
            0 => [(b'a', 1)],
            1 => [],
            2 => [(b'b', 3)],
            3 => [],
        ]
    };

    let expected_pruned = nfa! {
        start: 0,
        accept: [1],
        states: [
            0 => [(b'a', 1)],
            1 => [],
        ]
    };

    //Act
    let pruned = nfa_with_unreachable.pruned();

    //Assert
    assert!(pruned.is_isomorphic_to(&expected_pruned));
}

#[test]
fn next_states_includes_all_posibilities() {
    // Arrange
    let nfa = nfa! {
        start: 0,
        accept: [2, 3],
        states: [
            0 => [(b'a', 1), (b'a', 2)],
            1 => [(b'b', 3)],
            2 => [],
            3 => [],
        ]
    };
    let expected_0_a = [1, 2].iter().copied().collect();
    let expected_1_b = [3].iter().copied().collect();

    // Act
    let next_from_0_on_a = nfa.next_states(0, b'a');
    let next_from_1_on_b = nfa.next_states(1, b'b');

    // Assert
    assert_eq!(next_from_0_on_a, expected_0_a);
    assert_eq!(next_from_1_on_b, expected_1_b);
}
