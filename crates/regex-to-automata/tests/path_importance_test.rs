mod common;

use regex_to_automata::{EpsilonNfa, Automaton};
use common::{E, b, regex_strategy};
use proptest::prelude::*;
use std::collections::HashSet;
use rand::{Rng};

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

#[test]
fn dfa_construction_path_doesnt_matter_for_random_enfas_without_incoming_start() {
    //Arrange
    let mut rng = rand::rng();

    let cases = 1_000;
    let alphabet = b"abcdefghijklmnopqrstuvwxyz";
    let epsilon_probability = 0.4;

    for case in 0..cases {
        let node_count = rng.random_range(2..=12);
        let edges_per_node = rng.random_range(0..=8);
        let enfa = random_enfa_without_incoming_start(
            &mut rng,
            node_count,
            edges_per_node,
            alphabet,
            epsilon_probability,
        );


        //Act
        let long = enfa.to_nfa().to_dfa();
        let direct = enfa.to_dfa();


        //Assert
        assert_no_incoming_start(&enfa);
        assert!(
            direct.is_isomorphic_to(&long),
            "DFA construction paths differed on case {case}\nENFA:\n{}\nDirect DFA:\n{}\nLong-route DFA:\n{}",
            enfa.to_dot(),
            direct.to_dot(),
            long.to_dot(),
        );
    }
}



//HELPERS/GENERATORS
fn random_enfa_without_incoming_start<R: Rng>(
    rng: &mut R,
    node_count: usize,
    edges_per_node: usize,
    alphabet: &[u8],
    epsilon_probability: f64,
) -> EpsilonNfa {
    assert!(node_count >= 2);
    assert!(!alphabet.is_empty());

    let mut enfa = EpsilonNfa::new();

    for _ in 0..node_count {
        enfa.add_state();
    }

    let start = rng.random_range(0..node_count);
    let accept = rng.random_range(0..node_count);

    enfa.set_start(start);
    enfa.set_accept_states(HashSet::from([accept])).unwrap();
    enfa.alphabet = alphabet.iter().copied().collect();


    for _ in 0..(node_count * edges_per_node) {
        let from = rng.random_range(0..node_count);
        let to = random_state_except(rng, node_count, start);
        let symbol = if rng.random_bool(epsilon_probability) {
            E
        } else {
            let byte = alphabet[rng.random_range(0..alphabet.len())];
            b(byte)
        };
        enfa.add_transition(from, symbol, to);
    }

    enfa
}

fn random_state_except<R: Rng>(
    rng: &mut R,
    node_count: usize,
    forbidden: usize,
) -> usize {
    debug_assert!(node_count >= 2);

    loop {
        let state = rng.random_range(0..node_count);
        if state != forbidden {
            return state;
        }
    }
}

fn assert_no_incoming_start(enfa: &EpsilonNfa) {
    for from in 0..enfa.state_count() {
        for (_, to) in enfa.transitions_from(from) {
            assert_ne!(
                to,
                enfa.start_state(),
                "Found incoming transition to start state: {from} -> {to}\n{}",
                enfa.to_dot()
            );
        }
    }
}

