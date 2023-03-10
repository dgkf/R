use crate::parser::*;

use core::fmt;
use pest::error::LineColLocation::Pos;

#[derive(Debug, Clone)]
pub enum RError {
    VariableNotFound(String),
    IncorrectContext(String),
    ParseFailureVerbose(pest::error::Error<Rule>),
    ParseFailure(pest::error::Error<Rule>),
    ParseUnexpected(Rule),
    NotInterpretableAsLogical,
    ConditionIsNotScalar,
    CannotBeCoercedToLogical,
    CannotBeCoercedToInteger,

    Other(String),
}

impl RError {
    fn as_str(&self) -> String {
        match self {
            RError::IncorrectContext(x) => format!("Error: '{}' used in an incorrect context", x),
            RError::VariableNotFound(v) => format!("Error: object '{}' not found", v.as_str()),
            RError::ParseFailureVerbose(e) => format!("{}", e),
            RError::ParseFailure(e) => match e.line_col {
                Pos((line, col)) => format!("Parse failed at Line {}, Column {}", line, col),
                _ => format!("Parse failed at {:?}", e.line_col),
            },
            RError::ParseUnexpected(rule) => {
                format!("Parse failed. Found {:#?}", rule)
            }
            RError::NotInterpretableAsLogical => {
                format!("argument is not interpretable as logical")
            }
            RError::ConditionIsNotScalar => {
                format!("the condition has length > 1")
            }
            RError::CannotBeCoercedToLogical => {
                format!("object cannot be coerced to type 'logical'")
            }
            RError::CannotBeCoercedToInteger => {
                format!("object cannot be coerced to type 'integer'")
            }
            RError::Other(s) => {
                format!("{}", s)
            }
        }
    }
}

impl fmt::Display for RError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
