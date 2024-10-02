use lazy_static::lazy_static;
use r_derive::*;

use crate::callable::core::*;
use crate::error::*;
use crate::lang::*;
use crate::object::types::Logical;
use crate::object::*;

lazy_static! {
    pub static ref FORMALS: ExprList = ExprList::from(vec![(Some("x".to_string()), Expr::Missing)]);
}

/// Is an object NA?
///
/// Checks whether an object is NA (Not Available)
///
/// # In-Language
///
/// ## Usage
///
/// ```custom,{class=r}
/// is_na(x)
/// ```
///
/// ## Arguments
///
/// `x`: Object to check.
///
/// ## Examples
///
/// ```custom,{class=r-repl}
/// is_na(NA)
/// is_na(c(1, NA, 3))
/// ```
#[doc(alias = "is_na")]
#[builtin(sym = "is_na")]
#[derive(Debug, Clone, PartialEq)]
pub struct PrimitiveIsNA;

impl Callable for PrimitiveIsNA {
    fn formals(&self) -> ExprList {
        FORMALS.clone()
    }

    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (args, _ellipsis) = self.match_arg_exprs(args, stack)?;
        let mut args = Obj::List(args);

        let x = args.try_get_named("x")?.force(stack)?;

        match x {
            Obj::Vector(v) => {
                let result: Vec<Logical> = match v {
                    Vector::Logical(rep) => rep
                        .values_ref()
                        .iter()
                        .map(|x| Logical::Some(x.is_na()))
                        .collect(),
                    Vector::Integer(rep) => rep
                        .values_ref()
                        .iter()
                        .map(|x| Logical::Some(x.is_na()))
                        .collect(),
                    Vector::Double(rep) => rep
                        .values_ref()
                        .iter()
                        .map(|x| Logical::Some(x.is_na()))
                        .collect(),
                    Vector::Character(rep) => rep
                        .values_ref()
                        .iter()
                        .map(|x| Logical::Some(x.is_na()))
                        .collect(),
                };
                Ok(Obj::Vector(Vector::Logical(result.into())))
            }
            _ => Err(Error::ArgumentInvalid("x".to_string()).into()),
        }
    }
}

#[cfg(test)]

mod tests {
    use crate::{r, r_expect};
    // #[test]
    // todo
}
