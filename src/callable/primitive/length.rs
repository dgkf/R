use r_derive::*;

use crate::callable::core::*;
use crate::err;
use crate::lang::*;
use crate::object::*;

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "length")]
pub struct PrimitiveLength;

impl Callable for PrimitiveLength {
    fn formals(&self) -> ExprList {
        ExprList::from(vec![(Some("x".to_string()), Expr::Missing)])
    }
    fn call_matched(&self, args: List, _ellipsis: List, stack: &mut CallStack) -> EvalResult {
        let mut args = Obj::List(args);
        let x = args.try_get_named("x")?.force(stack)?;

        let length: usize = match x {
            Obj::Vector(ref vec) => match vec {
                Vector::Double(rep) => rep.len(),
                Vector::Integer(rep) => rep.len(),
                Vector::Logical(rep) => rep.len(),
                Vector::Character(rep) => rep.len(),
            },
            Obj::List(_) => todo!("Not implemented yet"),
            Obj::Environment(env) => env.len(),
            _ => return err!("Argument 'x' does not have a length"),
        };

        EvalResult::Ok(Obj::Vector(Vector::from(vec![OptionNA::Some(
            length as i32,
        )])))
    }
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::lang::EvalResult;
    use crate::{r, r_expect};

    #[test]
    fn double() {
        r_expect!(length(c(1, 2)) == 2)
    }
    #[test]
    fn integer() {
        r_expect!(length(2:11) == 10)
    }
    #[test]
    fn logical() {
        r_expect!(length(c(true, true, false)) == 3)
    }
    #[test]
    fn character() {
        r_expect!(length(c("a", "b", "c", "d")) == 4)
    }
    // TODO
    // #[test]
    // fn list() {
    //     r_expect!(length(list(1, 2, 3, 5) == 5))
    // }
    #[test]
    fn environment() {
        r_expect! {{"
            x <- 1
            length(x) == 1
        "}}
    }
    #[test]
    fn empty() {
        r_expect!(length(1[false]) == 0)
    }
    #[test]
    fn subset_mask() {
        r_expect! {{
            length((1:3)[c(true, true, false)]) == 2
        }}
    }
    #[test]
    fn null() {
        assert_eq!(
            r! {length(null)},
            EvalResult::Err(Error::Other("Argument 'x' does not have a length".to_string()).into())
        )
    }
}
