use lazy_static::lazy_static;
use r_derive::*;

use crate::callable::core::*;
use crate::lang::*;
use crate::object::*;

lazy_static! {
    pub static ref FORMALS: ExprList = ExprList::from(vec![
        (Some("x".to_string()), Expr::Missing),
        (None, Expr::Ellipsis(None))
    ]);
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "print")]
pub struct PrimitivePrint;
impl Callable for PrimitivePrint {
    fn formals(&self) -> ExprList {
        FORMALS.clone()
    }

    fn call_matched(&self, args: List, _ellipsis: List, stack: &mut CallStack) -> EvalResult {
        let mut args = Obj::List(args);
        let x = args.try_get_named("x")?.force(stack)?;
        println!("{x}");
        Obj::Null.into()
    }
}

#[cfg(test)]
mod tests {
    // test that print returns Null
    use crate::lang::EvalResult;
    use crate::object::Obj;
    use crate::r;
    #[test]
    fn test_print() {
        assert_eq!(r!(print(10)), EvalResult::Ok(Obj::Null))
    }
    // TODO: Tests that check for output to stdout
}
