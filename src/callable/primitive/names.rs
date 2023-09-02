use r_derive::*;

use crate::ast::*;
use crate::lang::*;
use crate::callable::core::*;
use crate::vector::vectors::OptionNA;

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "names")]
pub struct PrimitiveNames;
impl Callable for PrimitiveNames{
    fn formals(&self) -> ExprList {
        ExprList::from(vec![
            (Some(String::from("x")), Expr::Missing(String::from("x"))),
        ])
    }

    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let R::List(vals) = stack.parent_frame().eval_list_lazy(args)? else {
            unreachable!()
        };

        let (vals, _) = match_args(self.formals(), vals, &stack);
        let mut vals = R::List(vals);
        let x = vals.try_get_named("x")?;

        match x.force(stack)? {
            R::Null => Ok(R::Null),
            R::Closure(_, _) => Ok(R::Null),
            R::Vector(_) => Ok(R::Null),  // named vectors currently not supported...
            R::Expr(_) => Ok(R::Null),  // handle arg lists?
            R::Function(_, _, _) => Ok(R::Null), // return formals?
            R::List(x) => {
                Ok(x.iter()
                    .map(|(k, _)| match k {
                        Some(name) => OptionNA::Some(name.clone()),
                        None => OptionNA::NA,  // unlike R, unnamed elements are NAs
                    })
                    .collect::<Vec<OptionNA<String>>>()
                    .into())
            },
            R::Environment(e) => {
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
