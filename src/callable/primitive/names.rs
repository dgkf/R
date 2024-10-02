use r_derive::*;

use crate::callable::core::*;
use crate::lang::*;
use crate::object::types::Character;
use crate::object::*;

/// Get Names of an Object
///
/// Returns the element names for vector-like objects, or names of
/// symbols assigned in environments.
///
/// # In-Language
///
/// ## Usage
///
/// ```custom,{class=rust}
/// names(x)
/// ```
///
/// ## Arguments
///
/// * `x`: An object from which to retrieve names
///
/// ## Examples
///
/// Accessing the names of elements in a `list`
///
/// ```custom,{class=r-repl}
/// names((a = 1, b = 2, c = 3))
/// ```
///
/// <div class="warning">
///
/// Unlike R, `names()` will always return a `character` vector, even if
/// no element is named.
///
/// </div>
///
/// ```custom,{class=r-repl}
/// names((1, 2, 3))
/// ```
///
/// Accessing names in an `environment`
///
/// ```custom,{class=r-repl}
/// x <- 3; y <- 4
/// names(environment())
/// ```
///
#[doc(alias = "names")]
#[builtin(sym = "names")]
#[derive(Debug, Clone, PartialEq)]
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
            Promise(..) => Ok(Null),
            Vector(v) => match v.names() {
                Some(n) => Ok(Obj::Vector(n.into())),
                None => Ok(Null),
            },
            Expr(..) => Ok(Null),     // handle arg lists?
            Function(..) => Ok(Null), // return formals?
            List(x) => {
                Ok(x.pairs_ref()
                    .iter()
                    .map(|(k, _)| match k {
                        Character::Some(name) => OptionNA::Some(name.clone()),
                        OptionNA::NA => OptionNA::NA, // unlike R, unnamed elements are NAs
                    })
                    .collect::<Vec<OptionNA<String>>>()
                    .into())
            }
            Environment(e) => {
                let mut names = e.values.borrow().keys().cloned().collect::<Vec<String>>();
                names.sort();
                Ok(names.into())
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::error::Error;
    use crate::r;

    #[test]
    fn no_args() {
        assert_eq!(
            r! { names() },
            Error::ArgumentMissing(String::from("x")).into()
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
