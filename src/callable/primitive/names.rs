use r_derive::*;

use crate::ast::*;
use crate::lang::*;
use crate::callable::core::*;
use crate::vector::types::OptionNa;

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "names")]
pub struct PrimitiveNames;
impl Callable for PrimitiveNames{
    fn formals(&self) -> ExprList {
        ExprList::from(vec![
            (Some(String::from("x")), Expr::Missing),
        ])
    }

    fn call_matched(&self, mut args: R, mut _ellipsis: R, stack: &mut CallStack) -> EvalResult {
        let x = args.try_get_named("x")?.force(stack)?;

        use R::*;
        match x {
            Null => Ok(Null),
            Closure(_, _) => Ok(Null),
            Vector(_) => Ok(Null),  // named vectors currently not supported...
            Expr(_) => Ok(Null),  // handle arg lists?
            Function(_, _, _) => Ok(Null), // return formals?
            List(x) => {
                Ok(x.iter()
                    .map(|(k, _)| match k {
                        Some(name) => OptionNa(Some(name.clone())),
                        None => OptionNa(None),  // unlike R, unnamed elements are NAs
                    })
                    .collect::<Vec<OptionNa<String>>>()
                    .into())
            },
            Environment(e) => {
                let mut names = e.values.borrow().keys()
                    .map(|k| k.clone())
                    .collect::<Vec<String>>();

                names.sort();
                Ok(names.into())
            },
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
            r!{ names() },
            RError::ArgumentMissing(String::from("x")).into()
        )
    }

    #[test]
    fn from_environment() {
        assert_eq!(
            r!{ x <- 3; names(environment()) },
            r!{ "x" }
        )
    }

    #[test]
    fn from_list() {
        assert_eq!(
            r!{ names(list(a = 1, b = 2, 3, d = 4)) },
            r!{ c("a", "b", NA, "d") }
        )
    }
}
