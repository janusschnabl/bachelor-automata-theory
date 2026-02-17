use ce_core::{Env, Generate, ValidationResult, define_env, rand};
use serde::{Deserialize, Serialize};

define_env!(RegexToDfaEnv);

#[derive(tapi::Tapi, Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Input {
    pub regex: String, //temp lige pt, skal evt laves om
}

#[derive(tapi::Tapi, Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Output {
    pub dot: String, //burde være final, du må godt lige tjekke gennem
}

impl Env for RegexToDfaEnv {
    type Input = Input;

    type Output = Output;

    type Meta = ();

    fn run(input: &Self::Input) -> ce_core::Result<Self::Output> {
        Ok(Output {
            dot: input.regex.clone(),
        })
    }

    fn validate(_input: &Self::Input, _output: &Self::Output) -> ce_core::Result<ValidationResult> {
        Ok(ValidationResult::Correct)
    }
}

impl Generate for Input {
    type Context = ();

    fn gn<R: rand::Rng>(_cx: &mut Self::Context, _rng: &mut R) -> Self {
        Self::default()
    }
}
