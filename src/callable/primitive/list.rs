use r_derive::Primitive;

use crate::ast::*;
use crate::lang::*;
use crate::callable::core::*;

#[derive(Debug, Clone, Primitive)]
pub struct PrimitiveList;

impl PrimitiveSYM for PrimitiveList {
    const SYM: &'static str = "list";
}

impl Callable for PrimitiveList {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        stack.eval_list(args)
    }
}
