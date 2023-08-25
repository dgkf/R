use crate::ast::ExprList;
use crate::lang::{CallStack, EvalResult, R};

pub fn primitive_callstack(_: ExprList, stack: &mut CallStack) -> EvalResult {
    Ok(R::List(
        stack.frames.iter()
            .skip(1)
            .map(|f| (None, R::Expr(f.call.clone())))
            .collect()
    ))
}
