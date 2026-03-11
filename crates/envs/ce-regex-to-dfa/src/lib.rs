use ce_core::{Env, EnvError, Generate, ValidationResult, define_env, rand};
use regex_to_automata::EpsilonNfa;
use serde::{Deserialize, Serialize};

define_env!(RegexToDfaEnv);

#[derive(tapi::Tapi, Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Input {
    pub regex: String,
}

#[derive(tapi::Tapi, Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Output {
    pub dot: String,
}

impl Env for RegexToDfaEnv {
    type Input = Input;

    type Output = Output;

    type Meta = ();

    fn run(input: &Self::Input) -> ce_core::Result<Self::Output> {
        let nfa = EpsilonNfa::from_regex(&input.regex)
            .map_err(|e| EnvError::InvalidInputForProgram {
                message: e.to_string(),
                source: None,
            })?;

        Ok(Output { dot: nfa.to_dot() })
    }

    fn validate(_input: &Self::Input, _output: &Self::Output) -> ce_core::Result<ValidationResult> {
        Ok(ValidationResult::Correct)
    }
}

impl Generate for Input {
    type Context = ();

    fn gn<R: rand::Rng>(_cx: &mut Self::Context, rng: &mut R) -> Self {
        fn gen_regex<R: rand::Rng>(rng: &mut R, depth: usize) -> String {
            const LITS: &[char] = &['a', 'b', 'c'];

            if depth == 0 {
                return LITS[rng.random_range(0..LITS.len())].to_string();
            }

            match rng.random_range(0..5) {
                0 => LITS[rng.random_range(0..LITS.len())].to_string(),

                1 => format!(
                    "{}{}",
                    gen_regex(rng, depth - 1),
                    gen_regex(rng, depth - 1)
                ),

                2 => format!(
                    "{}|{}",
                    gen_regex(rng, depth - 1),
                    gen_regex(rng, depth - 1)
                ),

                3 => format!("({})*", gen_regex(rng, depth - 1)),

                _ => format!("({})", gen_regex(rng, depth - 1)),
            }
        }

        Self {
            regex: gen_regex(rng, 5),
        }
    }
}
