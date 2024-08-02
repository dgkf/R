use lazy_static::lazy_static;
use r_derive::*;

use crate::callable::core::*;
use crate::context::Context;
use crate::error::Error;
use crate::lang::*;
use crate::object::*;

lazy_static! {
    pub static ref FORMALS: ExprList = ExprList::from(vec![
        (Some("x".to_string()), Expr::Missing),
        (
            Some("envir".to_string()),
            Expr::Call(
                Box::new(Expr::Symbol("parent".to_string())),
                ExprList::new()
            )
        )
    ]);
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "eval")]
pub struct PrimitiveEval;
impl Callable for PrimitiveEval {
    fn formals(&self) -> ExprList {
        FORMALS.clone()
    }

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
