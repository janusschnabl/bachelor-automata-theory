use ce_core::{Env, EnvError, Generate, ValidationResult, define_env, rand};
use regex_to_automata::{Automaton, Dfa, EpsilonNfa, generate_random_regex};
use serde::{Deserialize, Serialize};

define_env!(RegexToDfaDirectEnv);

#[derive(tapi::Tapi, Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Input {
    pub regex: String,
}

#[derive(tapi::Tapi, Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Output {
    pub dot: String,
}

impl Env for RegexToDfaDirectEnv {
    type Input = Input;

    type Output = Output;

    type Meta = ();

    fn run(input: &Self::Input) -> ce_core::Result<Self::Output> {
        let nfa = EpsilonNfa::from_regex(&input.regex, None)
            .map_err(|e| EnvError::InvalidInputForProgram {
                message: e.to_string(),
                source: None,
            })?
            .to_nfa()
            .to_dfa();

        Ok(Output { dot: nfa.to_dot() })
    }

    fn validate(_input: &Self::Input, _output: &Self::Output) -> ce_core::Result<ValidationResult> {
        let expected = EpsilonNfa::from_regex(&_input.regex, None)
            .map_err(|e| EnvError::InvalidInputForProgram {
                message: e.to_string(),
                source: None,
            })?
            .to_nfa()
            .to_dfa();

        let produced =
            Dfa::from_dot(_output.dot.as_str()).map_err(|e| EnvError::InvalidInputForProgram {
                message: e.to_string(),
                source: None,
            })?;

        if produced.is_isomorphic_to(&expected) {
            Ok(ValidationResult::Correct)
        } else {
            Ok(ValidationResult::Mismatch {
                reason: "produced automaton is not isomorphic to expected".into(),
            })
        }
    }
}

impl Generate for Input {
    type Context = ();

    fn gn<R: rand::Rng>(_cx: &mut Self::Context, rng: &mut R) -> Self {
        Self {
            regex: generate_random_regex(rng, 5, 5).unwrap(),
        }
    }
}
