use regex_to_automata::Symbol;
use proptest::prelude::*;

pub const E: Symbol = Symbol::Epsilon;
pub const fn b(c: u8) -> Symbol { Symbol::Byte(c) }

/// Generates random valid regexes using supported operators: *, +, concatenation, alternation
pub fn regex_strategy() -> impl Strategy<Value = String> {
    let literal = prop_oneof![
        Just("".to_string()),
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
                    .prop_map(|(a, b)| format!("{}{}", a, b)),
                (inner.clone(), inner.clone())
                    .prop_map(|(a, b)| format!("{}|{}", a, b)),
            ]
        },
    )
}

/// Generates random input strings from alphabet {a, b, c}
pub fn input_string_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[abc]{0,20}").unwrap()
}

#[macro_export]
macro_rules! epsilon_nfa {
    (
        start: $start:expr,
        accept: $accept:expr,
        states: [
            $(
                $id:expr => [ $( ($sym:expr, $to:expr) ),* $(,)? ]
            ),* $(,)?
        ]
    ) => {{
        use regex_to_automata::{EpsilonNfa, State};
        let mut states = Vec::new();
        $(
            states.push(State {
                transitions: vec![
                    $( ($sym, $to) ),*
                ],
            });
        )*
        EpsilonNfa {
            states,
            start: $start,
            accept: $accept,
            alphabet: Default::default(),
        }
    }};
}

#[macro_export]
macro_rules! nfa {
    (
        start: $start:expr,
        accept: [ $($accept_state:expr),* $(,)? ],
        states: [
            $(
                $id:expr => [ $( ($sym:expr, $to:expr) ),* $(,)? ]
            ),* $(,)?
        ]
    ) => {{
        use regex_to_automata::{Nfa, State};
        let mut states = Vec::new();
        $(
            states.push(State {
                transitions: vec![
                    $( ($sym, $to) ),*
                ],
            });
        )*
        Nfa {
            states,
            start: $start,
            accept: vec![$($accept_state),*],
            alphabet: Default::default(),
        }
    }};
}

#[macro_export]
macro_rules! dfa {
    (
        start: $start:expr,
        accept: [ $($accept_state:expr),* $(,)? ],
        states: [
            $(
                $id:expr => [ $( ($sym:expr, $to:expr) ),* $(,)? ]
            ),* $(,)?
        ]
    ) => {{
        use regex_to_automata::{Dfa, State};
        let mut states = Vec::new();
        $(
            states.push(State {
                transitions: vec![
                    $( ($sym, $to) ),*
                ],
            });
        )*
        Dfa {
            states,
            start: $start,
            accept: vec![$($accept_state),*],
            alphabet: Default::default(),
        }
    }};
}
