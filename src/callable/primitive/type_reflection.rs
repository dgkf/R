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

        EvalResult::Ok(Obj::Vector(Vector::Character(vec![x.type_of()].into())))
    }
}

#[cfg(test)]

mod tests {
    use crate::{r, r_expect};
    #[test]
    fn character() {
        r_expect!(typeof("a") == "character")
    }
}
