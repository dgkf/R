use crate::{
    lang::{CallStack, Signal},
    parser::*,
};

use core::fmt;
use pest::error::LineColLocation::Pos;

#[macro_export]
macro_rules! internal_err {
    () => {
        $crate::error::RError::Internal(None, std::file!(), std::line!()).into()
    };
    ( $x:expr ) => {
        $crate::error::RError::Internal(Some($x.to_string()), std::file!(), std::line!()).into()
    };
}

#[derive(Debug, Clone, PartialEq)]
pub enum RError {
    VariableNotFound(String),
    IncorrectContext(String),
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

    // parsing errors
    ParseFailureVerbose(Box<pest::error::Error<Rule>>),
    ParseFailure(Box<pest::error::Error<Rule>>),
    ParseUnexpected(Rule),

    // temporary workaround until we propagate call stack to all error locations
    WithCallStack(Box<RError>, CallStack),

    // in-dev errors
    Unimplemented(Option<String>),
    Internal(Option<String>, &'static str, u32),

    // features
    FeatureDisabledRestArgs,
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
                format!("Parse failed. Found unexpected parsing rule '{:#?}'", rule)
            }
            RError::NotInterpretableAsLogical => {
                "argument is not interpretable as logical".to_string()
            }
            RError::ConditionIsNotScalar => "the condition has length > 1".to_string(),
            RError::CannotBeCoercedToCharacter => {
                "object cannot be coerced to type 'character'".to_string()
            }
            RError::CannotBeCoercedToLogical => {
                "object cannot be coerced to type 'logical'".to_string()
            }
            RError::CannotBeCoercedToInteger => {
                "object cannot be coerced to type 'integer'".to_string()
            }
            RError::CannotBeCoercedToNumeric => {
                "object cannot be coerced to type 'numeric'".to_string()
            }
            RError::CannotBeCoercedTo(to) => {
                format!("object cannot be coerced to type '{to}'")
            }
            RError::Other(s) => s.to_string(),
            RError::WithCallStack(e, c) => format!("{}\n{c}", e.as_str()),
            RError::ArgumentMissing(s) => format!("argument '{s}' is missing with no default"),
            RError::ArgumentInvalid(s) => format!("argument '{s}' is invalid"),
            RError::Unimplemented(Some(s)) => {
                format!("Uh, oh! Looks like '{s}' is only partially implemented")
            }
            RError::Unimplemented(_) => {
                "Uh, oh! You tried to do something that is only partially implemented".to_string()
            }
            RError::Internal(None, file, line) => format!("Internal Error ({file}:{line})"),
            RError::Internal(Some(msg), file, line) => {
                format!("Internal Error ({file}:{line})\n{msg}")
            }
            RError::FeatureDisabledRestArgs => {
                "..rest syntax currently disabled. To enable, re-build with\n\n    cargo build --features rest-args\n".to_string()
            }
        }
    }
}

impl fmt::Display for RError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error: {}", self.as_str())
    }
}

impl From<RError> for Signal {
    fn from(val: RError) -> Self {
        Signal::Error(val)
    }
}

impl<T> From<RError> for Result<T, Signal> {
    fn from(val: RError) -> Self {
        Err(Signal::Error(val))
    }
}
