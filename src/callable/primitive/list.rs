use r_derive::*;

use crate::ast::*;
use crate::callable::core::*;
use crate::lang::*;

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "list")]
pub struct PrimitiveList;
impl Callable for PrimitiveList {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        stack.eval_list_eager(args)
    }

    fn call_assign(&self, _value: Expr, _args: ExprList, _stack: &mut CallStack) -> EvalResult {
        // unpacking here!
        todo!();
    }
}
