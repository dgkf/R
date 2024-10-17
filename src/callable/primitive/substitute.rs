use r_derive::*;

use crate::callable::core::*;
use crate::callable::keywords::KeywordParen;
use crate::context::Context;
use crate::formals;
use crate::internal_err;
use crate::lang::*;
use crate::object::*;

/// Substitute Expressions
///
/// <div class="warning">
///
/// Under construction! There are known differences with R's `subsitute()`.
/// Notably,  atomic values _will not_ yet be substituted.
///
/// </div>
///
/// # In-Language
///
/// ## Usage
///
/// ```custom,{class=r}
/// substitute(expr, envir = environment())
/// ```
///
/// ## Arguments
///
/// `expr`: Quoted code to evaluate.
/// `envir`: An environment in which to substitute the expression, defaulting to the current
///   environment.
///
/// ## Examples
///
/// ```custom,{class=r-repl}
/// x <- quote(1 + 2)
/// substitute(x * 10)
/// ```
///
#[doc(alias = "substitute")]
#[builtin(sym = "substitute")]
#[derive(Debug, Clone, PartialEq)]
pub struct PrimitiveSubstitute;

formals!(PrimitiveSubstitute, "(expr, envir = environment())");

impl Callable for PrimitiveSubstitute {
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

        fn recurse(exprs: ExprList, env: &Environment, paren: bool) -> ExprList {
            exprs
                .into_iter()
                .map(|(key, expr)| (key, substitute(expr, env, paren)))
                .collect()
        }

        // add parenthesis around ambigous expressions, namely anonymous functions and infix calls
        fn paren_if_infix(expr: Expr) -> Expr {
            match expr {
                Function(..) => Expr::new_primitive_call(KeywordParen, ExprList::from(vec![expr])),
                Call(what, exprs) => match *what {
                    Primitive(p) if p.is_infix() => {
                        let expr = Call(Box::new(Primitive(p)), exprs);
                        Expr::new_primitive_call(KeywordParen, ExprList::from(vec![expr]))
                    }
                    _ => Call(what, exprs),
                },
                _ => expr,
            }
        }

        fn substitute(expr: Expr, env: &Environment, paren: bool) -> Expr {
            match expr {
                Symbol(s) => {
                    // promise expressions (ie arguments) are replaced with their unevaluated expressions
                    match env.values.borrow().get(&s) {
                        Some(Obj::Expr(expr)) | Some(Obj::Promise(_, expr, _)) => {
                            if paren {
                                paren_if_infix(expr.clone())
                            } else {
                                expr.clone()
                            }
                        }
                        // NOTE: In R, substitute will further replace with deparsed values
                        _ => Symbol(s),
                    }
                }
                List(exprs) => List(recurse(exprs, env, false)),
                Function(params, body) => Function(
                    recurse(params, env, false),
                    Box::new(substitute(*body, env, false)),
                ),
                Call(what, exprs) => match *what {
                    Primitive(p) if p.is_infix() => {
                        Call(Box::new(Primitive(p)), recurse(exprs, env, true))
                    }
                    _ => Call(
                        Box::new(substitute(*what, env, true)),
                        recurse(exprs, env, false),
                    ),
                },
                other => other,
            }
        }

        match substitute(expr, env.as_ref(), false) {
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
            r! { x <- quote(a(b, c)); substitute(0 + x) },
            r! { quote(0 + a(b, c)) }
        );
    }

    #[test]
    fn substituted_minimally_parenthesizes() {
        assert_eq!(
            r! { x <- quote(1 + 2); substitute(x(a, b, x)) },
            r! { quote((1 + 2)(a, b, 1 + 2)) }
        );
    }

    #[test]
    fn substituted_infix_op_calls_get_parenthesized() {
        assert_eq!(
            r! { x <- quote(1 + 2); substitute(0 * x) },
            r! { quote(0 * (1 + 2)) } // note non-default associativity
        );
    }

    #[test]
    fn substituted_functions_gets_parenthesized() {
        assert_eq!(
            r! { x <- quote(function(a, b) a + b); substitute(0 + x) },
            r! { quote(0 + (function(a, b) a + b)) }
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
