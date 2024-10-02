use lazy_static::lazy_static;
use r_derive::*;

use crate::callable::core::*;
use crate::error::Error;
use crate::lang::*;
use crate::object::reptype::RepType;
use crate::object::types::Logical;
use crate::object::*;

lazy_static! {
    pub static ref FORMALS: ExprList =
        ExprList::from(vec![(Some("x".to_string()), Expr::Missing),]);
}

/// All true?
///
/// Checks whether all values of a logical vector are true.
///
/// # In-Language
///
/// ## Usage
///
/// ```custom,{class=r}
/// all(x)
/// ```
///
/// ## Arguments
///
/// `x`: Logical vector to check.
///
/// ## Examples
///
/// Evaluate code as though it were executed in the current environment.
///
/// ```custom,{class=r-repl}
/// all([true, false])
/// all([true, na, false])
/// all([true, true])
/// ```
/// ## Deviations from R
/// The function performs no conversions and expects all arguments to be of type logical.
#[doc(alias = "all")]
#[builtin(sym = "all")]
#[derive(Debug, Clone, PartialEq)]
pub struct PrimitiveAll;
impl Callable for PrimitiveAll {
    fn formals(&self) -> ExprList {
        FORMALS.clone()
    }

    fn call_matched(&self, args: List, _ellipsis: List, stack: &mut CallStack) -> EvalResult {
        let x = Obj::List(args).try_get_named("x")?.force(stack)?;

        let mut all = Logical::Some(true);

        if let Obj::Vector(Vector::Logical(v)) = x {
            for val in v.values_ref().iter() {
                if let Logical::Some(x) = val {
                    if !x {
                        return EvalResult::Ok(vec![Logical::Some(false)].into());
                    }
                } else {
                    all = Logical::NA;
                }
            }
        } else {
            let msg = "Argument 'x' should be logical vector.";
            return Error::Other(msg.into()).into();
        }
        return EvalResult::Ok(vec![all].into());
    }
}

#[cfg(test)]
mod tests {
    use crate::{r, r_expect};
    #[test]
    fn all_true() {
        r_expect!(all([true, true]));
    }
    #[test]
    fn not_all_true() {
        r_expect!(all([true, false]));
        r_expect!(all([na, false]));
        r_expect!(all([false, na]));
    }
    #[test]
    fn na() {
        r_expect!(all([true, true]))
    }
    #[test]
    fn all_error() {
        todo!()
    }
}
