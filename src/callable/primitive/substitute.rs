use lazy_static::lazy_static;
use r_derive::*;

use crate::callable::core::*;
use crate::context::Context;
use crate::internal_err;
use crate::lang::*;
use crate::object::*;

lazy_static! {
    pub static ref FORMALS: ExprList = ExprList::from(vec![
        (Some("expr".to_string()), Expr::Missing),
        (
            Some("envir".to_string()),
            Expr::Call(
                Box::new(Expr::Symbol("environment".to_string())),
                ExprList::new()
            )
        )
    ]);
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "substitute")]
pub struct PrimitiveSubstitute;
impl Callable for PrimitiveSubstitute {
    fn formals(&self) -> ExprList {
        FORMALS.clone()
    }

    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        use Expr::*;
        let (args, _ellipsis) = self.match_arg_exprs(args, stack)?;
        let mut args = Obj::List(args);

        let Obj::Environment(env) = args.try_get_named("envir")?.force(stack)? else {
            return internal_err!();
        };

        let Obj::Promise(_, expr, _) = args.try_get_named("expr")? else {
            return internal_err!();
        };

        fn recurse(exprs: ExprList, env: &Environment) -> ExprList {
            exprs
                .into_iter()
                .map(|(key, expr)| (key, substitute(expr, env)))
                .collect()
        }

        fn substitute(expr: Expr, env: &Environment) -> Expr {
            match expr {
                Symbol(s) => {
                    // promises are replaced
                    match env.values.borrow().get(&s) {
                        Some(Obj::Promise(_, expr, _)) => expr.clone(),
                        Some(Obj::Expr(e)) => e.clone(),
                        _ => Symbol(s),
                    }
                }
                List(exprs) => List(recurse(exprs, env)),
                Function(params, body) => {
                    Function(recurse(params, env), Box::new(substitute(*body, env)))
                }
                Call(what, exprs) => Call(Box::new(substitute(*what, env)), recurse(exprs, env)),
                other => other,
            }
        }

        match substitute(expr, env.as_ref()) {
            e @ (Symbol(_) | List(..) | Function(..) | Call(..) | Primitive(..)) => {
                Ok(Obj::Expr(e))
            }
            other => stack.eval(other),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::r;

    #[test]
    fn function_param_promises() {
        assert_eq!(
            r! { f <- function(x) { x; substitute(x) }; f(1 + 2) },
            r! { quote(1 + 2) }
        );
    }

    #[test]
    fn quoted_expressions() {
        assert_eq!(
            r! { x <- quote(1 + 2); substitute(0 + x) },
            r! { quote(0 + (1 + 2)) } // note non-default associativity
        );
    }

    #[test]
    fn default_local_substitute() {
        assert_eq!(
            r! { f <- function(x) { g <- function(y) substitute(x); g() }; f(1 + 2) },
            r! { quote(x) }
        );
    }

    #[test]
    fn envir_substitute() {
        assert_eq!(
            r! {{"
                f <- function(x) {
                  g <- function(x) {
                    substitute(x, envir = parent(environment()))
                  }
                  g(1 + 2)
                }
                f(3 + 4)
            "}},
            r! { quote(3 + 4) }
        );
    }
}
