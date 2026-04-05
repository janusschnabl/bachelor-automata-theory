mod common;

use regex_to_automata::{EpsilonNfa, Automaton, Symbol, State};
use common::{E, b};
use rand::seq::SliceRandom;
use rand::{SeedableRng, rngs::StdRng};

#[test]
fn empty_string_structure() {
    let produced = EpsilonNfa::from_regex("", None).unwrap();

    let expected = epsilon_nfa! {
        start: 0,
        accept: 1,
        states: [
            0 => [(E,1)],
            1 => [],
        ]
    };

    assert!(produced.is_isomorphic_to(&expected));
}

#[test]
fn literal_structure() {
    let produced = EpsilonNfa::from_regex("a", None).unwrap();

    let expected = epsilon_nfa! {
        start: 0,
        accept: 1,
        states: [
            0 => [(b(b'a'),1)],
            1 => [],
        ]
    };

    assert!(produced.is_isomorphic_to(&expected));
}

#[test]
fn concat_structure() {
    let produced = EpsilonNfa::from_regex("ab", None).unwrap();

    let expected = epsilon_nfa! {
        start: 0,
        accept: 3,
        states: [
            0 => [(b(b'a'),1)],
            1 => [(E,2)],
            2 => [(b(b'b'),3)],
            3 => [],
        ]
    };

    assert!(produced.is_isomorphic_to(&expected));
}

#[test]
fn alternation_structure() {
    let produced = EpsilonNfa::from_regex("a|b", None).unwrap();

    let expected = epsilon_nfa! {
        start: 0,
        accept: 5,
        states: [
            0 => [(E,1),(E,3)],
            1 => [(b(b'a'),2)],
            2 => [(E,5)],
            3 => [(b(b'b'),4)],
            4 => [(E,5)],
            5 => [],
        ]
    };

    assert!(produced.is_isomorphic_to(&expected));
}

#[test]
fn kleene_star_structure() {
    let produced = EpsilonNfa::from_regex("a*", None).unwrap();

    let expected = epsilon_nfa! {
        start: 0,
        accept: 3,
        states: [
            0 => [(E,1),(E,3)],
            1 => [(b(b'a'),2)],
            2 => [(E,1),(E,3)],
            3 => [],
        ]
    };

    assert!(produced.is_isomorphic_to(&expected));
}

#[test]
fn plus_structure() {
    let produced = EpsilonNfa::from_regex("a+", None).unwrap();

    let expected = epsilon_nfa! {
        start: 0,
        accept: 3,
        states: [
            0 => [(E,1)],
            1 => [(b(b'a'),2)],
            2 => [(E,1),(E,3)],
            3 => [],
        ]
    };

    assert!(produced.is_isomorphic_to(&expected));
}

#[test]
fn nested_expression_structure() {
    let produced = EpsilonNfa::from_regex("(a|b)*", None).unwrap();

    let expected = epsilon_nfa! {
        start: 0,
        accept: 7,
        states: [
            0 => [(E,1),(E,7)],
            1 => [(E,2),(E,4)],
            2 => [(b(b'a'),3)],
            3 => [(E,6)],
            4 => [(b(b'b'),5)],
            5 => [(E,6)],
            6 => [(E,1),(E,7)],
            7 => [],
        ]
    };

    assert!(produced.is_isomorphic_to(&expected));
}
#[test]
fn star_of_empty_structure() {
    let produced = EpsilonNfa::from_regex("()*", None).unwrap();

    let expected = epsilon_nfa! {
        start: 0,
        accept: 3,
        states: [
            0 => [(E,1),(E,3)],
            1 => [(E,2)],
            2 => [(E,1),(E,3)],
            3 => [],
        ]
    };

    assert!(produced.is_isomorphic_to(&expected));
}

#[test]
fn alternation_then_concat_structure() {
    let produced = EpsilonNfa::from_regex("(a|b)c", None).unwrap();

    let expected = epsilon_nfa! {
        start: 0,
        accept: 7,
        states: [
            0 => [(E,1),(E,3)],
            1 => [(b(b'a'),2)],
            2 => [(E,5)],
            3 => [(b(b'b'),4)],
            4 => [(E,5)],
            5 => [(E,6)],
            6 => [(b(b'c'),7)],
            7 => [],
        ]
    };

    assert!(produced.is_isomorphic_to(&expected));
}

#[test]
fn grouping_does_not_change_structure() {
    let a = EpsilonNfa::from_regex("ab", None).unwrap();
    let b = EpsilonNfa::from_regex("(ab)", None).unwrap();

    assert!(a.is_isomorphic_to(&b));
}

#[test]
fn chain_of_literals() {
    let produced = EpsilonNfa::from_regex("abc", None).unwrap();

    let expected = epsilon_nfa! {
        start: 0,
        accept: 5,
        states: [
            0 => [(b(b'a'),1)],
            1 => [(E,2)],
            2 => [(b(b'b'),3)],
            3 => [(E,4)],
            4 => [(b(b'c'),5)],
            5 => [],
        ]
    };

    assert!(produced.is_isomorphic_to(&expected));
}

#[test]
fn alternation_is_left_associative() {
    let a = EpsilonNfa::from_regex("a|b|c", None).unwrap();
    let b = EpsilonNfa::from_regex("(a|b)|c", None).unwrap();
    let c = EpsilonNfa::from_regex("a|(b|c)", None).unwrap();

    assert!(a.is_isomorphic_to(&b));
    assert!(!a.is_isomorphic_to(&c));
}


#[test]
fn precedence_rules_match_explicit_parentheses() {
    // Verify that implicit precedence matches explicit parentheses.
    // Regex precedence (highest → lowest):
    //   1. Kleene operators (*, +)
    //   2. Concatenation
    //   3. Alternation (|)

    let cases = [
        // concat > |
        ("a|bc", "a|(bc)"),
        ("ab|c", "(ab)|c"),

        // * > |
        ("a|b*", "a|(b*)"),
        ("a|b+", "a|(b+)"),

        // * > concat
        ("ab*", "a(b*)"),
        ("ab+", "a(b+)"),

        // full precedence chain
        ("a|bc*", "a|(b(c*))"),
    ];

    for (implicit, explicit) in cases {
        let a = EpsilonNfa::from_regex(implicit, None).unwrap();
        let b = EpsilonNfa::from_regex(explicit, None).unwrap();

        assert!(
            a.is_isomorphic_to(&b),
            "expected {implicit:?} and {explicit:?} to parse equivalently"
        );
    }
}

#[test]
fn renumbered_states_produce_isomorphic_epsilon_nfas() {
    let a = epsilon_nfa! {
        start: 0,
        accept: 1,
        states: [
            0 => [(b(b'a'),1)],
            1 => [],
        ]
    };

    let b = epsilon_nfa! {
        start: 1,
        accept: 0,
        states: [
            0 => [],
            1 => [(b(b'a'),0)],
        ]
    };

    assert!(a.is_isomorphic_to(&b));
}


fn permute_states(nfa: &EpsilonNfa, rng: &mut StdRng) -> EpsilonNfa {
    let mut perm: Vec<usize> = (0..nfa.states.len()).collect();
    perm.shuffle(rng);

    let mut new_states = vec![State { transitions: vec![] }; nfa.states.len()];

    for (old_id, state) in nfa.states.iter().enumerate() {
        let new_id = perm[old_id];

        new_states[new_id] = State {
            transitions: state
                .transitions
                .iter()
                .map(|(sym, to)| (*sym, perm[*to]))
                .collect(),
        };

        new_states[new_id].transitions.shuffle(rng);
    }

    EpsilonNfa {
        states: new_states,
        start: perm[nfa.start],
        accept: perm[nfa.accept],
        alphabet: nfa.alphabet.clone(),
    }
}
#[test]
fn fuzz_isomorphism_with_random_permutations() {
    let seed: u64 = rand::random();
    println!("seed = {seed}");

    let mut rng = StdRng::seed_from_u64(seed);

    let regexes = [
        "a",
        "ab",
        "a|b",
        "(a|b)*",
        "(a|b)|(c|d)",
        "(ab)*c",
        "(a|b)*abb",
    ];

    for r in regexes {
        let nfa = EpsilonNfa::from_regex(r, None).unwrap();

        for _ in 0..500 {
            let permuted = permute_states(&nfa, &mut rng);

            assert!(
                nfa.is_isomorphic_to(&permuted),
                "seed={seed}, regex={r}"
            );
        }
    }
}
#[test]


fn fuzz_non_isomorphic_graphs() {
    let seed: u64 = rand::random();
    println!("seed = {seed}");

    let mut rng = StdRng::seed_from_u64(seed);

    let a = EpsilonNfa::from_regex("ab", None).unwrap();
    let b = EpsilonNfa::from_regex("a|b", None).unwrap();

    for _ in 0..100 {
        let pa = permute_states(&a, &mut rng);
        let pb = permute_states(&b, &mut rng);

        assert!(
            !pa.is_isomorphic_to(&pb),
            "seed={seed}"
        );
    }
}

#[test]
fn epsilon_closure_contains_itself() {
    // Arrange
    let nfa = epsilon_nfa! {
        start: 0,
        accept: 1,
        states: [
            0 => [(b(b'a'), 1)],
            1 => [],
        ]
    };

    // Act
    let closure = nfa.epsilon_closure(0);

    // Assert
    assert_eq!(closure.len(), 1);
    assert!(closure.contains(&0));
}

#[test]
fn epsilon_closure_follows_chain_of_epsilon_transitions() {
    // Arrange
    let nfa = epsilon_nfa! {
        start: 0,
        accept: 3,
        states: [
            0 => [(E, 1)],
            1 => [(E, 2)],
            2 => [(b(b'a'), 3)],
            3 => [],
        ]
    };

    // Act
    let closure = nfa.epsilon_closure(0);

    // Assert
    assert_eq!(closure.len(), 3);
    assert!(closure.contains(&0));
    assert!(closure.contains(&1));
    assert!(closure.contains(&2));
}

#[test]
fn epsilon_closure_explores_all_reachable_branches() {
    // Arrange
    let nfa = epsilon_nfa! {
        start: 0,
        accept: 4,
        states: [
            0 => [(E, 1), (E, 2)],
            1 => [(b(b'a'), 3)],
            2 => [(b(b'b'), 4)],
            3 => [],
            4 => [],
        ]
    };

    // Act
    let closure = nfa.epsilon_closure(0);

    // Assert
    assert_eq!(closure.len(), 3);
    assert!(closure.contains(&0));
    assert!(closure.contains(&1));
    assert!(closure.contains(&2));
}

#[test]
fn empty_regex_accepts_empty_string() {
    // Arrange
    let nfa = EpsilonNfa::from_regex("", None).unwrap();

    // Act & Assert
    assert!(nfa.accepts(""));
    assert!(!nfa.accepts("a"));
}

#[test]
fn single_character_regex_accepts_exact_match() {
    // Arrange
    let nfa = EpsilonNfa::from_regex("a", None).unwrap();

    // Act & Assert
    assert!(nfa.accepts("a"));
    assert!(!nfa.accepts(""));
    assert!(!nfa.accepts("b"));
    assert!(!nfa.accepts("aa"));
}

#[test]
fn concatenation_accepts_exact_sequence() {
    // Arrange
    let nfa = EpsilonNfa::from_regex("ab", None).unwrap();

    // Act & Assert
    assert!(nfa.accepts("ab"));
    assert!(!nfa.accepts("a"));
    assert!(!nfa.accepts("b"));
    assert!(!nfa.accepts("ba"));
    assert!(!nfa.accepts(""));
}

#[test]
fn alternation_accepts_any_option() {
    // Arrange
    let nfa = EpsilonNfa::from_regex("a|b", None).unwrap();

    // Act & Assert
    assert!(nfa.accepts("a"));
    assert!(nfa.accepts("b"));
    assert!(!nfa.accepts("ab"));
    assert!(!nfa.accepts(""));
    assert!(!nfa.accepts("c"));
}

#[test]
fn kleene_star_accepts_zero_or_more_occurrences() {
    // Arrange
    let nfa = EpsilonNfa::from_regex("a*", None).unwrap();

    // Act & Assert
    assert!(nfa.accepts(""));
    assert!(nfa.accepts("a"));
    assert!(nfa.accepts("aa"));
    assert!(nfa.accepts("aaa"));
    assert!(!nfa.accepts("b"));
    assert!(!nfa.accepts("ab"));
    assert!(!nfa.accepts("aab"));
}

#[test]
fn plus_accepts_one_or_more_occurrences() {
    // Arrange
    let nfa = EpsilonNfa::from_regex("a+", None).unwrap();

    // Act & Assert
    assert!(!nfa.accepts(""));
    assert!(nfa.accepts("a"));
    assert!(nfa.accepts("aa"));
    assert!(nfa.accepts("aaa"));
    assert!(!nfa.accepts("b"));
    assert!(!nfa.accepts("ab"));
}

#[test]
fn complex_pattern_accepts_valid_strings_and_rejects_invalid() {
    // Arrange
    let nfa = EpsilonNfa::from_regex("(a|b)*abb", None).unwrap();

    // Act & Assert
    assert!(nfa.accepts("abb"));
    assert!(nfa.accepts("aabb"));
    assert!(nfa.accepts("babb"));
    assert!(nfa.accepts("aababb"));
    assert!(nfa.accepts("bababb"));
    assert!(!nfa.accepts(""));
    assert!(!nfa.accepts("ab"));
    assert!(!nfa.accepts("aba"));
    assert!(!nfa.accepts("abba"));
}

#[test]
fn encode_label_epsilon_to_string() {
    // Arrange
    let epsilon = Symbol::Epsilon;

    // Act
    let encoded = EpsilonNfa::encode_label(&epsilon);

    // Assert
    assert_eq!(encoded, "ε");
}

#[test]
fn encode_label_byte_ascii_graphic() {
    // Arrange
    let byte_a = Symbol::Byte(b'A');

    // Act
    let encoded = EpsilonNfa::encode_label(&byte_a);

    // Assert
    assert_eq!(encoded, "A");
}

#[test]
fn encode_label_byte_non_graphic_hex() {
    // Arrange
    let byte_null = Symbol::Byte(0x00);

    // Act
    let encoded = EpsilonNfa::encode_label(&byte_null);

    // Assert
    assert_eq!(encoded, "0x00");
}

#[test]
fn decode_label_epsilon_from_string() {
    // Arrange
    let label_str = "ε";

    // Act
    let decoded = EpsilonNfa::decode_label(label_str).unwrap();

    // Assert
    assert_eq!(decoded, Symbol::Epsilon);
}

#[test]
fn decode_label_byte_ascii_graphic() {
    // Arrange
    let label_str = "A";

    // Act
    let decoded = EpsilonNfa::decode_label(label_str).unwrap();

    // Assert
    assert_eq!(decoded, Symbol::Byte(b'A'));
}

#[test]
fn encode_decode_roundtrip_epsilon() {
    // Arrange
    let original = Symbol::Epsilon;

    // Act
    let encoded = EpsilonNfa::encode_label(&original);
    let decoded = EpsilonNfa::decode_label(&encoded).unwrap();

    // Assert
    assert_eq!(decoded, original);
}

#[test]
fn encode_decode_roundtrip_byte() {
    // Arrange
    let original = Symbol::Byte(b'X');

    // Act
    let encoded = EpsilonNfa::encode_label(&original);
    let decoded = EpsilonNfa::decode_label(&encoded).unwrap();

    // Assert
    assert_eq!(decoded, original);
}

#[test]
fn decode_label_rejects_invalid_input() {
    // Arrange
    let invalid_label = "xyz";

    // Act & Assert
    assert!(EpsilonNfa::decode_label(invalid_label).is_err());
}
