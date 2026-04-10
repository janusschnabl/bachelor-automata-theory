mod common;
use regex_to_automata::{EpsilonNfa, Automaton, Nfa, Dfa};
use common::{regex_strategy, input_string_strategy};
use proptest::prelude::*;
use regex::Regex;

fn produce_all_automata_from_regex(regex: &str) -> (EpsilonNfa,Nfa,Dfa) {
    let enfa = EpsilonNfa::from_regex(regex, None).unwrap();
    let nfa = enfa.to_nfa();
    let dfa = nfa.to_dfa();
    (enfa, nfa, dfa)
}

fn assert_all_automata_match(
    all_automata: &(EpsilonNfa, Nfa, Dfa),
    accepting: &[&str],
    rejecting: &[&str],
) {
    for input in accepting {
        assert!(
            all_automata.0.accepts(input),
            "EpsilonNfa should accept '{}'",
            input
        );
        assert!(all_automata.1.accepts(input), "Nfa should accept '{}'", input);
        assert!(all_automata.2.accepts(input), "Dfa should accept '{}'", input);
    }
    for input in rejecting {
        assert!(
            !all_automata.0.accepts(input),
            "EpsilonNfa should reject '{}'",
            input
        );
        assert!(!all_automata.1.accepts(input), "Nfa should reject '{}'", input);
        assert!(!all_automata.2.accepts(input), "Dfa should reject '{}'", input);
    }
}

#[test]
fn empty_regex_accepts_empty_string() {
    // Arrange
    let all_automata = produce_all_automata_from_regex("");

    // Act & Assert
    assert_all_automata_match(&all_automata, &[""], &["a"]);
}

#[test]
fn single_character_regex_accepts_exact_match() {
    // Arrange
    let all_automata = produce_all_automata_from_regex("a");

    // Act & Assert
    assert_all_automata_match(&all_automata, &["a"], &["", "b", "aa"]);
}

#[test]
fn concatenation_accepts_exact_sequence() {
    // Arrange
    let all_automata = produce_all_automata_from_regex("ab");

    // Act & Assert
    assert_all_automata_match(&all_automata, &["ab"], &["a", "b", "ba", ""]);
}

#[test]
fn alternation_accepts_any_option() {
    // Arrange
    let all_automata = produce_all_automata_from_regex("a|b");

    // Act & Assert
    assert_all_automata_match(&all_automata, &["a", "b"], &["ab", "", "c"]);
}

#[test]
fn kleene_star_accepts_zero_or_more_occurrences() {
    // Arrange
    let all_automata = produce_all_automata_from_regex("a*");

    // Act & Assert
    assert_all_automata_match(&all_automata, &["", "a", "aa", "aaa"], &["b", "ab", "aab"]);
}

#[test]
fn plus_accepts_one_or_more_occurrences() {
    // Arrange
    let all_automata = produce_all_automata_from_regex("a+");

    // Act & Assert
    assert_all_automata_match(&all_automata, &["a", "aa", "aaa"], &["", "b", "ab"]);
}

#[test]
fn complex_pattern_accepts_valid_strings_and_rejects_invalid() {
    // Arrange
    let all_automata = produce_all_automata_from_regex("(a|b)*abb");

    // Act & Assert
    assert_all_automata_match(
        &all_automata,
        &["abb", "aabb", "babb", "aababb", "bababb"],
        &["", "ab", "aba", "abba"],
    );
}

proptest! {
    #[test]
    fn all_automata_agree_on_accept_reject(
        regex in regex_strategy(),
        input in input_string_strategy()
    ) {
        // Arrange: Build all three automata types from the same regex
        let enfa = EpsilonNfa::from_regex(&regex, None).unwrap();
        let nfa = enfa.to_nfa();
        let dfa = nfa.to_dfa();
        
        // Act & Assert: All three should give identical accept/reject results
        let enfa_accepts = enfa.accepts(&input);
        let nfa_accepts = nfa.accepts(&input);
        let dfa_accepts = dfa.accepts(&input);
        
        prop_assert_eq!(enfa_accepts, nfa_accepts, 
            "EpsilonNfa and Nfa disagree on input '{}' for regex '{}'", input, regex);
        prop_assert_eq!(nfa_accepts, dfa_accepts,
            "Nfa and Dfa disagree on input '{}' for regex '{}'", input, regex);
        prop_assert_eq!(enfa_accepts, dfa_accepts,
            "EpsilonNfa and Dfa disagree on input '{}' for regex '{}'", input, regex);
    }
}

fn oracle_accepts(regex: &str, input: &str) -> bool {
    let anchored = format!("^(?:{})$", regex);
    Regex::new(&anchored).unwrap().is_match(input)
}

proptest! {
    #[test]
    fn automata_match_regex_oracle(
        regex in regex_strategy(),
        input in input_string_strategy()
    ) {
        let enfa = EpsilonNfa::from_regex(&regex, None).unwrap();
        let nfa = enfa.to_nfa();
        let dfa = nfa.to_dfa();

        let expected = oracle_accepts(&regex, &input);

        prop_assert_eq!(enfa.accepts(&input), expected);
        prop_assert_eq!(nfa.accepts(&input), expected);
        prop_assert_eq!(dfa.accepts(&input), expected);
    }
}