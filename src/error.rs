use crate::{
    lang::{CallStack, Signal},
    object::Expr,
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
    CannotBeCoercedToDouble,
    CannotBeCoercedToInteger,
    CannotBeCoercedToLogical,
    CannotBeCoercedTo(&'static str),
    CannotBeCoercedToNumeric,
    CannotBeCoercedToNumerics,
    CannotBeOrdered,
    CannotBeCompared,

    // function parsing
    InvalidFunctionParameter(Expr),
    DuplicatedParameter(String),
    DuplicatedMoreParameter(),

    Missing,
    ArgumentMissing(String),
    ArgumentInvalid(String),
    Other(String),

    // parsing errors
    ParseFailureVerbose(pest::error::Error<en::Rule>),
    ParseFailure(pest::error::Error<en::Rule>),
    ParseUnexpected(en::Rule, (usize, usize)),

    // temporary workaround until we propagate call stack to all error locations
    WithCallStack(Box<Error>, CallStack),

    // in-dev errors
    Unimplemented(Option<String>),
    Internal(Option<String>, &'static str, u32),
    CannotEvaluateAsMutable(Expr),

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
            Error::ParseUnexpected(rule, _span) => {
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
            Error::CannotBeCoercedToDouble => {
                "object cannot be coerced to type 'double'".to_string()
            }
            Error::CannotBeCoercedTo(to) => {
                format!("object cannot be coerced to type '{to}'")
            }
            Error::CannotBeCoercedToNumeric => {
                "object cannot be coerced to type 'numeric'".to_string()
            },
            Error::CannotBeCoercedToNumerics=> {
                "objects cannot be coerced to type 'numeric'".to_string()
            },
            Error::CannotBeOrdered => {
                "objects cannot be ordered".to_string()
            },
            Error::CannotBeCompared => {
                "objects cannot be compared".to_string()
            },
            Error::Other(s) => s.to_string(),
            Error::WithCallStack(e, c) => format!("{}\n{c}", e.as_str()),
            Error::ArgumentMissing(s) => format!("argument '{s}' is missing with no default"),
            Error::ArgumentInvalid(s) => format!("argument '{s}' is invalid"),
            Error::Unimplemented(Some(s)) => {
                format!("Uh, oh! Looks like '{s}' is only partially implemented")
            }
            Error::CannotEvaluateAsMutable(expr) => {
                format!("Expression {expr} cannot be evaluated mutably")
            }
            Error::Unimplemented(_) => {
                "Uh, oh! You tried to do something that is only partially implemented".to_string()
            }
            Error::Internal(None, file, line) => format!("Internal Error ({file}:{line})"),
            Error::Internal(Some(msg), file, line) => {
                format!("Internal Error ({file}:{line})\n{msg}")
            }
            Error::FeatureDisabledRestArgs => {
                "..rest syntax currently disabled. To enable launch with\n\n    --experiments rest-args\n".to_string()
            }
            Error::Missing => "object is missing".to_string(),
            Error::InvalidFunctionParameter(expr) => format!("invalid function parameter: {}", expr),
            Error::DuplicatedParameter(name) => format!("duplicated parameter name: {}", name),
            Error::DuplicatedMoreParameter() => "duplicated '..<more>' parameters".to_string(),
        }
    }

    pub fn from_parse_error<R>(input: &str, error: pest::error::Error<R>) -> Error
    where
        R: pest::RuleType + Into<en::Rule>,
    {
        use pest::error::Error as E;
        use pest::error::ErrorVariant::*;
        use pest::error::InputLocation;
        use pest::{Position, Span};
        let variant = match error.variant {
            ParsingError {
                positives: p,
                negatives: n,
            } => ParsingError {
                positives: p.into_iter().map(|i| i.into()).collect(),
                negatives: n.into_iter().map(|i| i.into()).collect(),
            },
            CustomError { message } => CustomError { message },
        };

        match error.location {
            InputLocation::Pos(p) => {
                let pos = Position::new(input, p).unwrap();
                let err = E::new_from_pos(variant, pos);
                Error::ParseFailure(err)
            }
            InputLocation::Span((s, e)) => {
                let span = Span::new(input, s, e).unwrap();
                let err = E::new_from_span(variant, span);
                Error::ParseFailure(err)
            }
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
