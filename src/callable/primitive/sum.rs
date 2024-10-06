use r_derive::*;

use crate::callable::core::*;
use crate::error::*;
use crate::formals;
use crate::internal_err;
use crate::lang::*;
use crate::object::reptype::RepType;
use crate::object::*;

/// Calculate a Sum of Elements
///
/// # In-Language
///
/// ## Usage
///
/// ```custom,{class=r}
/// sum(...)
/// ```
///
/// ## Arguments
///
/// `...`: Objects that can be coerced into numerics.
///
/// ## Examples
///
/// ```custom,{class=r-repl}
/// sum(true, 1, 2, [3, 4, 5])
/// ```
///
#[doc(alias = "sum")]
#[builtin(sym = "sum")]
#[derive(Debug, Clone, PartialEq)]
pub struct PrimitiveSum;

formals!(PrimitiveSum, "(...)");

impl Callable for PrimitiveSum {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (_, ellipsis) = self.match_arg_exprs(args, stack)?;

        if ellipsis.is_empty() {
            return EvalResult::Ok(Obj::Vector(Vector::from(RepType::from(vec![0.0]))));
        }

        let objects: Vec<Obj> = force_promises(ellipsis, stack)?
            .into_iter()
            .map(|(_, value)| value)
            .collect();

        let mut any_double: bool = false;

        for obj in &objects {
            match obj {
                Obj::Vector(Vector::Double(..)) => {
                    any_double = true;
                    break;
                }
                Obj::Vector(Vector::Logical(..)) | Obj::Vector(Vector::Integer(..)) => {
                    continue;
                }
                _ => {
                    return EvalResult::Err(Signal::Error(Error::Other(String::from(
                        "All inputs must be of type numeric, integer or logical.",
                    ))))
                }
            }
        }

        if any_double {
            let mut sum: f64 = 0.0;

            for obj in objects {
                match obj {
                    Obj::Vector(vect) => {
                        match vect {
                            Vector::Logical(repr) => {
                                for x in repr.inner().borrow().iter() {
                                    match *x {
                                        OptionNA::NA => {
                                            let rep: RepType<OptionNA<f64>> =
                                                RepType::from(vec![OptionNA::NA]);
                                            return EvalResult::Ok(Obj::Vector(Vector::from(rep)));
                                        }
                                        OptionNA::Some(x) => sum += x as i32 as f64,
                                    }
                                }
                            }
                            Vector::Integer(repr) => {
                                for x in repr.inner().borrow().iter() {
                                    match *x {
                                        OptionNA::NA => {
                                            let rep: RepType<OptionNA<f64>> =
                                                RepType::from(vec![OptionNA::NA]);
                                            return EvalResult::Ok(Obj::Vector(Vector::from(rep)));
                                        }
                                        OptionNA::Some(x) => sum += x as f64,
                                    }
                                }
                            }
                            Vector::Double(repr) => {
                                for x in repr.inner().borrow().iter() {
                                    match *x {
                                        OptionNA::NA => {
                                            let rep: RepType<OptionNA<f64>> =
                                                RepType::from(vec![OptionNA::NA]);
                                            return EvalResult::Ok(Obj::Vector(Vector::from(rep)));
                                        }
                                        OptionNA::Some(x) => sum += x,
                                    }
                                }
                            }
                            _ => return internal_err!(),
                        };
                    }
                    _ => return internal_err!(),
                }
            }
            EvalResult::Ok(Obj::Vector(Vector::from(RepType::from(vec![sum]))))
        } else {
            let mut sum: i32 = 0;

            for obj in objects {
                match obj {
                    Obj::Vector(vect) => {
                        match vect {
                            Vector::Logical(repr) => {
                                for x in repr.inner().borrow().iter() {
                                    match *x {
                                        OptionNA::NA => {
                                            let rep: RepType<OptionNA<i32>> =
                                                RepType::from(vec![OptionNA::NA]);
                                            return EvalResult::Ok(Obj::Vector(Vector::from(rep)));
                                        }
                                        OptionNA::Some(x) => sum += x as i32,
                                    }
                                }
                            }
                            Vector::Integer(repr) => {
                                for x in repr.inner().borrow().iter() {
                                    match *x {
                                        OptionNA::NA => {
                                            let rep: RepType<OptionNA<i32>> =
                                                RepType::from(vec![OptionNA::NA]);
                                            return EvalResult::Ok(Obj::Vector(Vector::from(rep)));
                                        }
                                        OptionNA::Some(x) => sum += x,
                                    }
                                }
                            }
                            _ => return internal_err!(),
                        };
                    }
                    _ => return internal_err!(),
                }
            }
            EvalResult::Ok(Obj::Vector(Vector::from(RepType::from(vec![sum]))))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::r;

    #[test]
    fn sum_empty() {
        assert_eq!(r! {sum()}, r! {0.0},)
    }

    // FIXME: Overly aggressive conversion to Double, representations for NAs
    // #[test]
    // fn sum_add() {
    //     assert_eq!(r! {sum(1L, 2L)}, r! {1L + 2L});
    // }
    // #[test]
    // fn sum_na_logical() {
    //     assert_eq!(r! {sum(NA, true)}, r! {NA + 0L});
    // }
    //
    // #[test]
    // fn sum_na_integer() {
    //     assert_eq!(r! {sum(NA, 1L)}, r! {NA * 1L});
    // }

    #[test]
    fn sum_na_double() {
        assert_eq!(r! {sum(NA, 1)}, r! {NA * 1});
    }

    #[test]
    fn sum_null() {
        assert!((r! {sum(null)}).is_err());
    }

    #[test]
    fn sum_double() {
        assert_eq!(r! {{"sum(c(1, 2), c(3, 4))"}}, r! {{"10"}})
    }

    #[test]
    fn sum_integer() {
        assert_eq!(r! {{"sum(1L, 2L, c(3L, 4L))"}}, r! {{"10L"}},)
    }

    #[test]
    fn sum_logical() {
        assert_eq!(r! {{"sum(c(true, true), false)"}}, r! {2L})
    }

    #[test]
    fn sum_integer_double() {
        assert_eq!(r! {{"sum(c(-1L, 0L), 1)"}}, r! {{"0"}},)
    }

    #[test]
    fn sum_integer_logical() {
        assert_eq!(
            r! {{"sum(c(1L, 0L), 2L, c(false, false, true))"}},
            r! {{"4L"}},
        )
    }

    #[test]
    fn sum_double_logical() {
        assert_eq!(r! {{"sum(c(1, 4, 5), false, true)"}}, r! {{"11"}},)
    }

    #[test]
    fn sum_integer_double_logical() {
        assert_eq!(
            r! {{"sum(c(1L, -2L), c(5, -5, 1), c(true, false))"}},
            r! {{"1"}},
        )
    }

    #[test]
    fn sum_named_args() {
        assert_eq!(r! {{"sum(a = 1, b = 2)"}}, r! {{"3"}},)
    }
}
