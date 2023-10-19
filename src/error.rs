use crate::{
    lang::{CallStack, Signal},
    parser::*,
};

use core::fmt;
use pest::error::LineColLocation::Pos;

#[macro_export]
macro_rules! internal_err {
    () => {
        $crate::error::Error::Internal(None, std::file!(), std::line!()).into()
    };
    ( $x:expr ) => {
        $crate::error::Error::Internal(Some($x.to_string()), std::file!(), std::line!()).into()
    };
}

#[macro_export]
macro_rules! err {
    ( $x:expr ) => {
        $crate::error::Error::Other($x.to_string()).into()
    };
}

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    VariableNotFound(String),
    IncorrectContext(String),
    NotInterpretableAsLogical,
    ConditionIsNotScalar,
    CannotBeCoercedToCharacter,
    CannotBeCoercedToNumeric,
    CannotBeCoercedToInteger,
    CannotBeCoercedToLogical,
    CannotBeCoercedTo(&'static str),

    Missing,
    ArgumentMissing(String),
    ArgumentInvalid(String),
    Other(String),

    // parsing errors
    ParseFailureVerbose(Box<pest::error::Error<Rule>>),
    ParseFailure(Box<pest::error::Error<Rule>>),
    ParseUnexpected(Rule),

    // temporary workaround until we propagate call stack to all error locations
    WithCallStack(Box<Error>, CallStack),

    // in-dev errors
    Unimplemented(Option<String>),
    Internal(Option<String>, &'static str, u32),

    // features
    FeatureDisabledRestArgs,
}

impl Error {
    fn as_str(&self) -> String {
        match self {
            Error::IncorrectContext(x) => format!("'{}' used in an incorrect context", x),
            Error::VariableNotFound(v) => format!("object '{}' not found", v.as_str()),
            Error::ParseFailureVerbose(e) => format!("{}", e),
            Error::ParseFailure(e) => match e.line_col {
                Pos((line, col)) => format!("Parse failed at Line {}, Column {}", line, col),
                _ => format!("Parse failed at {:?}", e.line_col),
            },
            Error::ParseUnexpected(rule) => {
                format!("Parse failed. Found unexpected parsing rule '{:#?}'", rule)
            }
            Error::NotInterpretableAsLogical => {
                "argument is not interpretable as logical".to_string()
            }
            Error::ConditionIsNotScalar => "the condition has length > 1".to_string(),
            Error::CannotBeCoercedToCharacter => {
                "object cannot be coerced to type 'character'".to_string()
            }
            Error::CannotBeCoercedToLogical => {
                "object cannot be coerced to type 'logical'".to_string()
            }
            Error::CannotBeCoercedToInteger => {
                "object cannot be coerced to type 'integer'".to_string()
            }
            Error::CannotBeCoercedToNumeric => {
                "object cannot be coerced to type 'numeric'".to_string()
            }
            Error::CannotBeCoercedTo(to) => {
                format!("object cannot be coerced to type '{to}'")
            }
            Error::Other(s) => s.to_string(),
            Error::WithCallStack(e, c) => format!("{}\n{c}", e.as_str()),
            Error::ArgumentMissing(s) => format!("argument '{s}' is missing with no default"),
            Error::ArgumentInvalid(s) => format!("argument '{s}' is invalid"),
            Error::Unimplemented(Some(s)) => {
                format!("Uh, oh! Looks like '{s}' is only partially implemented")
            }
            Error::Unimplemented(_) => {
                "Uh, oh! You tried to do something that is only partially implemented".to_string()
            }
            Error::Internal(None, file, line) => format!("Internal Error ({file}:{line})"),
            Error::Internal(Some(msg), file, line) => {
                format!("Internal Error ({file}:{line})\n{msg}")
            }
            Error::FeatureDisabledRestArgs => {
                "..rest syntax currently disabled. To enable, re-build with\n\n    cargo build --features rest-args\n".to_string()
            }
            Error::Missing => "object is missing".to_string(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error: {}", self.as_str())
    }
}

impl From<Error> for Signal {
    fn from(val: Error) -> Self {
        Signal::Error(val)
    }
}

impl<T> From<Error> for Result<T, Signal> {
    fn from(val: Error) -> Self {
        Err(Signal::Error(val))
    }
}

impl From<&str> for Signal {
    fn from(msg: &str) -> Self {
        Signal::Error(Error::Other(msg.to_string()))
    }
}

impl<T> From<Signal> for Result<T, Signal> {
    fn from(value: Signal) -> Self {
        Err(value)
    }
}
