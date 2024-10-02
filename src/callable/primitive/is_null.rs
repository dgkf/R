use lazy_static::lazy_static;
use r_derive::*;

use crate::callable::core::*;
use crate::lang::*;
use crate::object::types::Logical;
use crate::object::*;

lazy_static! {
    pub static ref FORMALS: ExprList =
        ExprList::from(vec![(Some("x".to_string()), Expr::Missing),]);
}

/// Is an object `null`
///
/// Checks whether an object is null
///
/// # In-Language
///
/// ## Usage
///
/// ```custom,{class=r}
/// is_null(x)
/// ```
///
/// ## Arguments
///
/// `x`: Object to check.
///
/// ## Examples
///
/// ```custom,{class=r-repl}
/// is_null(null)
/// is_null(1)
/// ```
#[doc(alias = "is_null")]
#[builtin(sym = "is_null")]
#[derive(Debug, Clone, PartialEq)]
pub struct PrimitiveIsNull;
impl Callable for PrimitiveIsNull {
    fn formals(&self) -> ExprList {
        FORMALS.clone()
    }

    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (args, _ellipsis) = self.match_arg_exprs(args, stack)?;
        let mut args = Obj::List(args);

        let x = args.try_get_named("x")?.force(stack)?;

        EvalResult::Ok(Obj::Vector(Vector::from(vec![Logical::Some(matches!(
            x,
            Obj::Null
        ))])))
    }
}

#[cfg(test)]
mod tests {
    use crate::{r, r_expect};
    #[test]
    fn is_null() {
        r_expect!(is_null(null))
    }
    #[test]
    fn is_not_null() {
        r_expect!(is_null(1:2))
    }
}
