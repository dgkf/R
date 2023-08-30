use crate::{parser::*, lang::{CallStack, RSignal}};

use core::fmt;
use pest::error::LineColLocation::Pos;

#[derive(Debug, Clone, PartialEq)]
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
    CannotBeCoercedToNumeric,

    // temporary workaround until we propagate call stack to all error locations
    WithCallStack(Box<RError>, CallStack),  
    Other(String),
    ArgumentMissing(String),
    ArgumentInvalid(String),
}

impl RError {
    fn as_str(&self) -> String {
        match self {
            RError::IncorrectContext(x) => format!("'{}' used in an incorrect context", x),
            RError::VariableNotFound(v) => format!("object '{}' not found", v.as_str()),
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
            RError::CannotBeCoercedToNumeric => {
                format!("object cannot be coerced to type 'numeric'")
            }
            RError::Other(s) => {
                format!("{}", s)
            }
            RError::WithCallStack(e, c) => format!("{}\n{c}", e.as_str()),
            RError::ArgumentMissing(s) => format!("argument '{s}' is missing with no default"),
            RError::ArgumentInvalid(s) => format!("argument '{s}' is invalid."),
        }
    }
}

impl fmt::Display for RError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error: {}", self.as_str())
    }
}

impl Into<RSignal> for RError {
    fn into(self) -> RSignal {
        RSignal::Error(self)
    }
}

impl<T> Into<Result<T, RSignal>> for RError {
    fn into(self) -> Result<T, RSignal> {
        Err(RSignal::Error(self))
    }
}
