mod common;

use regex_to_automata::{EpsilonNfa, Automaton};
use common::{E, b};


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
fn from_regex_rejects_non_ascii() {
    let result = EpsilonNfa::from_regex("café", None);
    assert!(result.is_err());
}

#[test]
fn from_regex_rejects_backslash() {
    let result = EpsilonNfa::from_regex("a\\b", None);
    assert!(result.is_err());
}

#[test]
fn from_regex_rejects_unsupported_quantifier() {
    let result = EpsilonNfa::from_regex("a?", None);
    assert!(result.is_err());
}

#[test]
fn from_regex_accepts_space_in_literal() {
    let result = EpsilonNfa::from_regex("a b", None);
    assert!(result.is_ok());
}

#[test]
fn from_regex_rejects_tab_in_literal() {
    let result = EpsilonNfa::from_regex("a\tb", None);
    assert!(result.is_err());
}

#[test]
fn from_regex_rejects_newline_in_literal() {
    let result = EpsilonNfa::from_regex("a\nb", None);
    assert!(result.is_err());
}
