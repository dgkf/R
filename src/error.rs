use crate::parser::*;

use core::fmt;

#[derive(Debug)]
pub enum RError {
    VariableNotFound(String),
    IncorrectContext(String),
    ParseFailure(pest::error::Error<Rule>),
}

impl RError {
    fn as_str(&self) -> String {
        match self {
            RError::IncorrectContext(x) => format!("Error: '{}' used in an incorrect context", x),
            RError::VariableNotFound(v) => format!("Error: object '{}' not found", v.as_str()),
            RError::ParseFailure(e) => format!("Parse failed: {:?}", e),
        }
    }
}

impl fmt::Display for RError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
