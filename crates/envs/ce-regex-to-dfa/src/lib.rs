use ce_core::{Env, Generate, ValidationResult, define_env, rand,EnvError};
use serde::{Deserialize, Serialize};
use regex_syntax::{Parser,ParserBuilder, hir::{Hir, HirKind,Class}};
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
         let ast = ParserBuilder::new()
            .unicode(false)
            .build()
            .parse(&input.regex)
            .map_err(|e| EnvError::InvalidInputForProgram {
                message: e.to_string(),
                source: None,
            })?;

        let mut nfa = EpsilonNfa::new();

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
pub struct EpsilonNfa {
    pub states: Vec<State>,
}
impl EpsilonNfa {
    fn build_from_hir(&mut self, hir: &Hir) -> (usize, usize) {
        match hir.kind() {
            HirKind::Literal(lit) => {
                self.build_literal(&lit.0)
            }
            HirKind::Alternation(subs) => {
                self.build_alternation(subs)
            }
            HirKind::Concat(subs) => {
                self.build_concat(subs)
            }

            HirKind::Class(class) => {
                self.build_class(class)
            }
            _ => panic!("Unsupported HIR node for now {:?}", hir.kind()),
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
    fn build_alternation(&mut self, subs: &[Hir]) -> (usize, usize) {
        let start = self.add_state();
        let accept = self.add_state();

        for sub in subs {
            let (sub_start, sub_accept) = self.build_from_hir(sub);

            // start -> branch
            self.add_transition(start, Symbol::Epsilon, sub_start);

            // branch -> accept
            self.add_transition(sub_accept, Symbol::Epsilon, accept);
        }

        (start, accept)
    }
    fn build_concat(&mut self, subs: &[Hir]) -> (usize, usize) {
        assert!(!subs.is_empty());

        // build first
        let (mut start, mut accept) = self.build_from_hir(&subs[0]);

        // chain the rest
        for sub in &subs[1..] {
            let (next_start, next_accept) = self.build_from_hir(sub);

            // connect previous accept to next start
            self.add_transition(accept, Symbol::Epsilon, next_start);

            accept = next_accept;
        }

        (start, accept)
    }

    fn build_class(&mut self, class: &Class) -> (usize, usize) {
        let start = self.add_state();
        let accept = self.add_state();

        match class {
            Class::Bytes(bytes) => {
                for range in bytes.iter() {
                    for b in range.start()..=range.end() {
                        self.add_transition(start, Symbol::Byte(b), accept);
                    }
                }
            }
            Class::Unicode(_) => {
                panic!("Unicode classes not supported yet");
            }
        }

        (start, accept)
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
impl fmt::Display for EpsilonNfa {
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