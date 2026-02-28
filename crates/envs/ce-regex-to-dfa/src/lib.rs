use ce_core::{Env, Generate, ValidationResult, define_env, rand,EnvError};
use serde::{Deserialize, Serialize};
use regex_syntax::{Parser, hir::{Hir, HirKind}};
use std::fmt;

define_env!(RegexToDfaEnv);



#[derive(tapi::Tapi, Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Input {
    pub regex: String, //temp lige pt, skal evt laves om
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
         let ast = Parser::new()
            .parse(&input.regex)
            .map_err(|e| EnvError::InvalidInputForProgram {
                message: e.to_string(),
                source: None,
            })?;

        let mut nfa = Nfa::new();

        let (_start, _accept) = nfa.build_from_hir(&ast);

        Ok(Output {
            dot: format!("{}", nfa),
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


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Symbol {
    Epsilon,
    Byte(u8),
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Symbol::Epsilon => write!(f, "ε"),
            Symbol::Byte(b) => {
                if b.is_ascii_graphic() {
                    write!(f, "'{}'", *b as char)
                } else {
                    write!(f, "0x{:02X}", b)
                }
            }
        }
    }
}
#[derive(Debug, Clone ,Default)]
pub struct Nfa {
    pub states: Vec<State>,
}
impl Nfa {
    fn build_from_hir(&mut self, hir: &Hir) -> (usize, usize) {
        match hir.kind() {
            HirKind::Literal(lit) => {
                self.build_literal(&lit.0)
            }
            _ => panic!("Unsupported HIR node for now"),
        }
    }
    fn add_state(&mut self) -> usize {
        let id = self.states.len();
        self.states.push(State { transitions: vec![] });
        id
    }

    fn add_transition(&mut self, from: usize, symbol: Symbol, to: usize) {
        self.states[from].transitions.push((symbol, to));
    }

    fn build_literal(&mut self, bytes: &[u8]) -> (usize, usize) {
        let start = self.add_state();
        let mut current = start;

        for &b in bytes {
            let next = self.add_state();
            self.add_transition(current, Symbol::Byte(b), next);
            current = next;
        }

        (start, current)
    }
    pub fn new() -> Self {
        Self::default()
    }
}
impl fmt::Display for Nfa {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, state) in self.states.iter().enumerate() {
            writeln!(f, "State {}:", i)?;

            if state.transitions.is_empty() {
                writeln!(f, "  (no outgoing transitions)")?;
            }

            for (symbol, target) in &state.transitions {
                writeln!(f, "  --{}--> {}", symbol, target)?;
            }

            writeln!(f)?; // blank line between states
        }

        Ok(())
    }
}
#[derive(Debug, Clone)]
pub struct State {
    pub transitions: Vec<(Symbol, usize)>,
}