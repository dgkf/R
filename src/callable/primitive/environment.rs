use r_derive::*;

use crate::ast::*;
use crate::error::RError;
use crate::lang::*;
use crate::callable::core::*;

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "environment")]
pub struct PrimitiveEnvironment;
impl Callable for PrimitiveEnvironment {
    fn formals(&self) -> ExprList {
        ExprList::from(vec![
            (Some(String::from("fun")), Expr::Missing),
        ])
    }

    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (vals, _) = self.match_args(args, stack)?;
        let mut vals = R::List(vals);

        // default when `fun` is missing or not found
        let fun = vals.try_get_named("fun");
        if let Ok(R::Closure(Expr::Missing, _)) | Err(_) = fun {
            return Ok(R::Environment(stack.env().clone()));
        };

        // Err(_) case tested above
        let Ok(fun) = fun else { unreachable!() };

        // otherwise we can evaluate value and return result's environment
        match fun.force(stack)? {
            R::Closure(_, e) => Ok(R::Environment(e.clone())),
            R::Function(_, _, e) => Ok(R::Environment(e.clone())),
            R::Environment(e) => Ok(R::Environment(e.clone())),
            _ => RError::ArgumentInvalid(String::from("fun")).into(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{r, r_expect};

    #[test]
    fn no_args() {
        (r!{ environment() }).expect("environment() returns non-environment value");
        ()
    }

    #[test]
    fn functions_return_env() {
        r_expect!{{"
            x <- function() { function() { } }
            environment(x()) != environment(x)
        "}}
    }

    #[test]
    fn capture_local_env() {
        r_expect!{{"
            x <- function() { environment() }
            x() != environment(x)
        "}}
    }
}
