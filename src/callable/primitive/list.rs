use r_derive::*;

use crate::ast::*;
use crate::lang::*;
use crate::callable::core::*;

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "list")]
pub struct PrimitiveList;
impl Callable for PrimitiveList {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        stack.eval_list_greedy(args)
    }
}
