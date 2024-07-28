use r_derive::*;

use crate::callable::core::*;
use crate::internal_err;
use crate::lang::*;
use crate::object::Vector;
use crate::object::*;
use crate::r_vec;

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "type")]
pub struct PrimitiveType;

impl Callable for PrimitiveType {
    fn formals(&self) -> ExprList {
        ExprList::from(vec![(Some("x".to_string()), Expr::Missing)])
    }
    fn call_matched(&self, args: List, _ellipsis: List, stack: &mut CallStack) -> EvalResult {
        let mut args = Obj::List(args);
        let x = args.try_get_named("x")?.force(stack)?;

        let t = match x {
            Obj::Null => "null",
            Obj::Vector(v) => match v {
                Vector::Character(_) => "character",
                Vector::Integer(_) => "integer",
                Vector::Double(_) => "double",
                Vector::Logical(_) => "logical",
            },
            Obj::List(_) => "list",
            Obj::Expr(_) => "expression",
            Obj::Promise(..) => "promise",
            Obj::Function(..) => "function",
            Obj::Environment(..) => "environment",
        };

        EvalResult::Ok(r_vec!(t))
    }
}
