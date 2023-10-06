use r_derive::*;

use crate::callable::core::*;
use crate::lang::*;
use crate::object::*;

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "names")]
pub struct PrimitiveNames;
impl Callable for PrimitiveNames {
    fn formals(&self) -> ExprList {
        ExprList::from(vec![(Some(String::from("x")), Expr::Missing)])
    }

    fn call_matched(&self, args: List, mut _ellipsis: List, stack: &mut CallStack) -> EvalResult {
        let x = Obj::List(args).try_get_named("x")?.force(stack)?;

        use Obj::*;
        match x {
            Null => Ok(Null),
            Closure(_, _) => Ok(Null),
            Vector(_) => Ok(Null), // named vectors currently not supported...
            Expr(_) => Ok(Null),   // handle arg lists?
            Function(_, _, _) => Ok(Null), // return formals?
            List(x) => {
                Ok(x.values
                    .borrow()
                    .iter()
                    .map(|(k, _)| match k {
                        Some(name) => OptionNA::Some(name.clone()),
                        None => OptionNA::NA, // unlike R, unnamed elements are NAs
                    })
                    .collect::<Vec<OptionNA<String>>>()
                    .into())
            }
            Environment(e) => {
                let mut names = e
                    .values
                    .borrow()
                    .keys()
                    .map(|k| k.clone())
                    .collect::<Vec<String>>();

                names.sort();
                Ok(names.into())
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::error::RError;
    use crate::r;

    #[test]
    fn no_args() {
        assert_eq!(
            r! { names() },
            RError::ArgumentMissing(String::from("x")).into()
        )
    }

    #[test]
    fn from_environment() {
        assert_eq!(r! { x <- 3; names(environment()) }, r! { "x" })
    }

    #[test]
    fn from_list() {
        assert_eq!(
            r! { names(list(a = 1, b = 2, 3, d = 4)) },
            r! { c("a", "b", NA, "d") }
        )
    }
}
