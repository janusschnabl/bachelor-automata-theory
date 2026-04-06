use regex_to_automata::Symbol;

pub const E: Symbol = Symbol::Epsilon;
pub const fn b(c: u8) -> Symbol { Symbol::Byte(c) }

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
