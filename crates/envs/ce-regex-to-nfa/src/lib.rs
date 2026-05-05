use ce_core::{Env, EnvError, Generate, ValidationResult, define_env, rand};
use itertools::enumerate;
use regex_to_automata::{Automaton, EpsilonNfa, Nfa,generate_random_regex};
use serde::{Deserialize, Serialize};

define_env!(RegexToNfaEnv);

#[derive(tapi::Tapi, Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Input {
    pub regex: String,
}

#[derive(tapi::Tapi, Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Output {
    pub dot: String,
}

impl Env for RegexToNfaEnv {
    type Input = Input;

    type Output = Output;

    type Meta = ();
    type Annotation = ();

    fn run(input: &Self::Input) -> ce_core::Result<Self::Output> {
        let nfa = EpsilonNfa::from_regex(&input.regex, None)
            .map_err(|e| EnvError::InvalidInputForProgram {
                message: e.to_string(),
                source: None,
            })?
            .to_nfa();

        if is_too_complex_to_visualize(&nfa) {
            return Err(EnvError::InvalidInputForProgram {
                message: "Automaton likely takes a long time to visualize and given it's complexity it won't be useful for learning".into(),
                source: None,
            });
        }

        Ok(Output { dot: nfa.to_dot() })
    }

    fn validate(_input: &Self::Input, _output: &Self::Output) -> ce_core::Result<(ValidationResult, ())> {
        let expected = EpsilonNfa::from_regex(&_input.regex, None)
            .map_err(|e| EnvError::InvalidInputForProgram {
                message: e.to_string(),
                source: None,
            })?
            .to_nfa();

        let produced =
            Nfa::from_dot(_output.dot.as_str()).map_err(|e| EnvError::InvalidInputForProgram {
                message: e.to_string(),
                source: None,
            })?;

        if produced.is_isomorphic_to(&expected) {
            Ok((ValidationResult::Correct, ()))
        } else {
            Ok((ValidationResult::Mismatch {
                reason: "produced automaton is not isomorphic to expected".into(),
            }, ()))
        }
    }
}

impl Generate for Input {
    type Context = ();

    fn gn<R: rand::Rng>(_cx: &mut Self::Context, rng: &mut R) -> Self {
        Self {
            regex: generate_random_regex(rng, 2, 3).unwrap(),
        }
    }
}


fn is_too_complex_to_visualize<A: Automaton>(automaton: &A) -> bool {
    let states = automaton.get_states();
    let node_count = states.len();
    let mut edge_count = 0;
    for state in states {
        edge_count += state.transitions.len();
    }
    if node_count == 0{
        return false;
    }
    let edge_per_node = edge_count / node_count;
    edge_per_node > 7 && node_count > 10
}