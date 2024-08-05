use r_derive::*;

use crate::callable::core::*;
use crate::context::Context;
use crate::error::Error;
use crate::formals;
use crate::lang::*;
use crate::object::*;

/// Evaluate Code in an Environment
///
/// Evaluates a language object in a specified environment.
///
/// # In-Language
///
/// ## Usage
///
/// ```custom,{class=r}
/// eval(x, envir)
/// ```
///
/// ## Arguments
///
/// `x`: Quoted code to evaluate.
/// `envir`: An environment in which to evaluate the expression.
///
/// ## Examples
///
/// Evaluate code as though it were executed in the current environment.
///
/// ```custom,{class=r-repl}
/// x <- 1; y <- 2
/// eval(quote(x + y))
/// ```
///
/// Or specify another environment in which to search for symbols during
/// evaluation.
///
/// ```custom,{class=r-repl}
/// f <- fn() { x <- 10; y <- 2; environment() }
/// eval(quote(x * y), f())
/// ```
///
#[doc(alias = "eval")]
#[builtin(sym = "eval")]
#[derive(Debug, Clone, PartialEq)]
pub struct PrimitiveEval;

formals!(PrimitiveEval, "(x, envir = environment())");

impl Callable for PrimitiveEval {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (args, _ellipsis) = self.match_arg_exprs(args, stack)?;
        let mut args = Obj::List(args);

        let Obj::Expr(expr) = args.try_get_named("x")?.force(stack)? else {
            let msg = "Argument 'x' should be a quoted expression.";
            return Error::Other(msg.into()).into();
        };

        let Obj::Environment(envir) = args.try_get_named("envir")?.force(stack)? else {
            let msg = "Argument 'envir' should be an environment or data context.";
            return Error::Other(msg.into()).into();
        };

        stack.add_frame(expr.clone(), envir);
        let result = stack.eval(expr);
        stack.pop_frame_and_return(result)
    }
}
