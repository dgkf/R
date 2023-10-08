use crate::{parser::*, lang::{CallStack, Signal}};

use core::fmt;
use pest::error::LineColLocation::Pos;

#[macro_export]
macro_rules! internal_err {
    () => { 
        crate::error::RError::Internal(
            None, 
            std::file!(), 
            std::line!()
        ).into()
    };
    ( $x:expr ) => { 
        crate::error::RError::Internal(
            Some($x.to_string()),
            std::file!(), 
            std::line!()
        ).into() 
    };
}

#[derive(Debug, Clone, PartialEq)]
pub enum RError {
    VariableNotFound(String),
    IncorrectContext(String),
    ParseFailureVerbose(pest::error::Error<Rule>),
    ParseFailure(pest::error::Error<Rule>),
    ParseUnexpected(Rule),
    NotInterpretableAsLogical,
    ConditionIsNotScalar,
    CannotBeCoercedToCharacter,
    CannotBeCoercedToNumeric,
    CannotBeCoercedToInteger,
    CannotBeCoercedToLogical,
    CannotBeCoercedTo(&'static str),
    ArgumentMissing(String),
    ArgumentInvalid(String),
    Other(String),

    // temporary workaround until we propagate call stack to all error locations
    WithCallStack(Box<RError>, CallStack),      

    // in-dev errors
    Unimplemented(Option<String>),
    Internal(Option<String>, &'static str, u32),
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
            RError::CannotBeCoercedToCharacter => {
                format!("object cannot be coerced to type 'character'")
            },
            RError::CannotBeCoercedToLogical => {
                format!("object cannot be coerced to type 'logical'")
            }
            RError::CannotBeCoercedToInteger => {
                format!("object cannot be coerced to type 'integer'")
            }
            RError::CannotBeCoercedToNumeric => {
                format!("object cannot be coerced to type 'numeric'")
            }
            RError::CannotBeCoercedTo(to) => {
                format!("object cannot be coerced to type '{to}'")
            }
            RError::Other(s) => {
                format!("{}", s)
            }
            RError::WithCallStack(e, c) => format!("{}\n{c}", e.as_str()),
            RError::ArgumentMissing(s) => format!("argument '{s}' is missing with no default"),
            RError::ArgumentInvalid(s) => format!("argument '{s}' is invalid"),
            RError::Unimplemented(Some(s)) => format!("Uh, oh! Looks like '{s}' is only partially implemented"),
            RError::Unimplemented(_) => format!("Uh, oh! You tried to do something that is only partially implemented"),
            RError::Internal(None, file, line) => format!("Internal Error ({file}:{line})"),
            RError::Internal(Some(msg), file, line) => format!("Internal Error ({file}:{line})\n{msg}"),
        }
    }
}

impl fmt::Display for RError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error: {}", self.as_str())
    }
}

impl Into<Signal> for RError {
    fn into(self) -> Signal {
        Signal::Error(self)
    }
}

impl<T> Into<Result<T, Signal>> for RError {
    fn into(self) -> Result<T, Signal> {
        Err(Signal::Error(self))
    }
}
