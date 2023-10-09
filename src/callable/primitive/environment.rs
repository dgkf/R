use r_derive::*;

use crate::callable::core::*;
use crate::context::Context;
use crate::error::RError;
use crate::lang::*;
use crate::object::*;

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "environment")]
pub struct PrimitiveEnvironment;
impl Callable for PrimitiveEnvironment {
    fn formals(&self) -> ExprList {
        ExprList::from(vec![(Some(String::from("fun")), Expr::Missing)])
    }

    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (vals, _) = self.match_arg_exprs(args, stack)?;
        let mut vals = Obj::List(vals);

        // default when `fun` is missing or not found
        let fun = vals.try_get_named("fun");
        if let Ok(Obj::Closure(Expr::Missing, _)) | Err(_) = fun {
            return Ok(Obj::Environment(stack.env().clone()));
        };

        // Err(_) case tested above
        let Ok(fun) = fun else { unreachable!() };

        // otherwise we can evaluate value and return result's environment
        match fun.force(stack)? {
            Obj::Closure(_, e) => Ok(Obj::Environment(e.clone())),
            Obj::Function(_, _, e) => Ok(Obj::Environment(e.clone())),
            Obj::Environment(e) => Ok(Obj::Environment(e.clone())),
            _ => RError::ArgumentInvalid(String::from("fun")).into(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{r, r_expect};

    #[test]
    fn no_args() {
        (r! { environment() }).expect("environment() returns non-environment value");
    }

    #[test]
    fn functions_return_env() {
        r_expect! {{"
            x <- function() { function() { } }
            environment(x()) != environment(x)
        "}}
    }

    #[test]
    fn capture_local_env() {
        r_expect! {{"
            x <- function() { environment() }
            x() != environment(x)
        "}}
    }
}
