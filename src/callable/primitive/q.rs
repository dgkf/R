use r_derive::*;

use crate::lang::*;
use crate::callable::core::*;
use crate::object::ExprList;

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "q")]
pub struct PrimitiveQ;
impl Callable for PrimitiveQ {
    fn call(&self, _args: ExprList, _stack: &mut CallStack) -> EvalResult {
        Err(Signal::Condition(Cond::Terminate))
    }
}
