use std::fmt;

#[derive(Debug)]
pub enum Error {
    Parse(regex_syntax::Error),
    UnsupportedFeature(&'static str),
    InvalidAutomaton(&'static str),
    Message(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Parse(e) => write!(f, "regex parse error: {}", e),
            Error::UnsupportedFeature(s) => write!(f, "unsupported feature: {}", s),
            Error::InvalidAutomaton(s) => write!(f, "invalid automaton: {}", s),
            Error::Message(s) => write!(f, "{}", s),
        }
    }
}

impl std::error::Error for Error {}

impl From<regex_syntax::Error> for Error {
    fn from(e: regex_syntax::Error) -> Self {
        Error::Parse(e)
    }
}