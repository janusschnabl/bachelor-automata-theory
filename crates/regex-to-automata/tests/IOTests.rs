use regex_to_automata::{EpsilonNfa, Automaton};
use proptest::prelude::*;

//TODO: ok this file name is pretty shit. Should probably be more descriptive and include a few tests of specific structs and their dot formats
fn regex_strategy() -> impl Strategy<Value = String> {
    let literal = prop_oneof![
        Just("a".to_string()),
        Just("b".to_string()),
        Just("c".to_string()),
    ];

    literal.prop_recursive(
        4,  // depth
        16, // max size
        2,  // items per level
        |inner| {
            prop_oneof![
                inner.clone().prop_map(|r| format!("({})*", r)),
                inner.clone().prop_map(|r| format!("({})+", r)),
                (inner.clone(), inner.clone())
                    .prop_map(|(a,b)| format!("{}{}", a,b)),
                (inner.clone(), inner.clone())
                    .prop_map(|(a,b)| format!("{}|{}", a,b)),
            ]
        }
    )
}


proptest! {
    #[test]
    fn dot_roundtrip_preserves_structure(regex in regex_strategy()) {

        let nfa = EpsilonNfa::from_regex(&regex, None).unwrap();

        let dot = nfa.to_dot();
        let parsed = EpsilonNfa::from_dot(&dot).unwrap();

        prop_assert!(nfa.is_isomorphic_to(&parsed));
    }
}