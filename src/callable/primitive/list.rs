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
        let vals: Result<Vec<_>, _> = args
            .into_iter()
            .map(|(n, v)| match stack.eval(v) {
                Ok(val) => Ok((n, val)),
                Err(err) => Err(err),
            })
            .collect();

        Ok(R::List(vals?))
    }
}
