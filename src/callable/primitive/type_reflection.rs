use r_derive::*;

use crate::callable::core::*;
use crate::formals;
use crate::lang::*;
use crate::object::*;

#[doc(alias = "typeof")]
#[builtin(sym = "typeof")]
#[derive(Debug, Clone, PartialEq)]
pub struct PrimitiveTypeOf;

formals!(PrimitiveTypeOf, "(x,)");

impl Callable for PrimitiveTypeOf {
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
        EvalResult::Ok(Obj::Vector(Vector::Character(vec![t.to_string()].into())))
    }
}
