use r_derive::*;

use crate::ast::*;
use crate::lang::*;
use crate::callable::core::*;

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "parent")]
pub struct PrimitiveParent;
impl Callable for PrimitiveParent{
    fn formals(&self) -> ExprList {
        ExprList::from(vec![
            (Some(String::from("x")), Expr::Missing),
        ])
    }

    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let R::List(vals) = stack.parent_frame().eval_list_lazy(args)? else {
            unreachable!()
        };

        let (vals, _) = match_args(self.formals(), vals, &stack);
        let mut vals = R::List(vals);

        // default when `x` is missing or not found
        let x = vals.try_get_named("x");
        if let Ok(R::Closure(Expr::Missing, _)) | Err(_) = x {
            return Ok(stack.env().parent.clone().map_or(R::Null, |e| R::Environment(e)));
        };

        match vals.try_get_named("x")?.force(stack)?.environment() {
            Some(e) => Ok(e.parent.clone().map_or(R::Null, |e| R::Environment(e))),
            None => Ok(R::Null),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{r, r_expect};

    #[test]
    fn no_args() {
        // assumes default environment has a parent... may change in the future
        r_expect!{
            parent(environment()) == parent()
        }
    }

    #[test]
    fn function_parent_env() {
        r_expect!{{"
            x <- function() { }
            parent(x) == parent()
        "}}
    }

    #[test]
    fn nested_function_parent_env() {
        r_expect!{{"
            x <- function() { function() {} }
            parent(x()) == environment(x)
        "}}
    }
}
