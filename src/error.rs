use crate::{
    lang::{CallStack, RSignal},
    parser::*,
};

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
    ArgumentMissing(String),
    ArgumentInvalid(String),
    Other(String),

    // temporary workaround until we propagate call stack to all error locations
    WithCallStack(Box<RError>, CallStack),

    // in-dev errors
    Unimplemented(Option<String>),
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
                "argument is not interpretable as logical".to_string()
            }
            RError::ConditionIsNotScalar => "the condition has length > 1".to_string(),
            RError::CannotBeCoercedToLogical => {
                "object cannot be coerced to type 'logical'".to_string()
            }
            RError::CannotBeCoercedToInteger => {
                "object cannot be coerced to type 'integer'".to_string()
            }
            RError::CannotBeCoercedToNumeric => {
                "object cannot be coerced to type 'numeric'".to_string()
            }
            RError::Other(s) => s.to_string(),
            RError::WithCallStack(e, c) => format!("{}\n{c}", e.as_str()),
            RError::ArgumentMissing(s) => format!("argument '{s}' is missing with no default"),
            RError::ArgumentInvalid(s) => format!("argument '{s}' is invalid."),
            RError::Unimplemented(Some(s)) => {
                format!("Uh, oh! Looks like '{s}' is only partially implemented.")
            }
            RError::Unimplemented(_) => {
                "Uh, oh! You tried to do something that is only partially implemented.".to_string()
            }
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
