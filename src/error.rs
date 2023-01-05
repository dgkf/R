use crate::lang::*;
use crate::parser::*;

use core::fmt;
use pest::error::LineColLocation::Pos;

#[derive(Debug, Clone)]
pub enum RError {
    VariableNotFound(String),
    IncorrectContext(String),
    ParseFailure(pest::error::Error<Rule>),
    NotInterpretableAsLogical,
    ConditionIsNotScalar,
    CannotBeCoercedToLogical,
}

impl RError {
    fn as_str(&self) -> String {
        match self {
            RError::IncorrectContext(x) => format!("Error: '{}' used in an incorrect context", x),
            RError::VariableNotFound(v) => format!("Error: object '{}' not found", v.as_str()),
            RError::ParseFailure(e) => match e.line_col {
                Pos((line, col)) => format!("Parse failed at Line {}, Column {}", line, col),
                _ => format!("Parse failed at {:?}", e.line_col),
            },
            RError::NotInterpretableAsLogical => {
                format!("argument is not interpretable as logical")
            }
            RError::ConditionIsNotScalar => {
                format!("the condition has length > 1")
            }
            RError::CannotBeCoercedToLogical => {
                format!("object cannot be coerced to type 'logical'")
            }
        }
    }
}

impl fmt::Display for RError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
