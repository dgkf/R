use r_derive::*;

use crate::callable::core::*;
use crate::error::*;
use crate::internal_err;
use crate::lang::*;
use crate::object::rep::Rep;
use crate::object::*;

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "sum")]
pub struct PrimitiveSum;

impl Callable for PrimitiveSum {
    fn formals(&self) -> ExprList {
        ExprList::from(vec![(None, Expr::Ellipsis(None))])
    }

    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (_, ellipsis) = self.match_arg_exprs(args, stack)?;

        if ellipsis.is_empty() {
            return EvalResult::Ok(Obj::Vector(Vector::from(Rep::from(vec![0.0]))));
        }

        let objects: Vec<Obj> = force_closures(ellipsis, stack)?
            .into_iter()
            .map(|(_, value)| value)
            .collect();

        let mut any_numeric: bool = false;

        for obj in &objects {
            match obj {
                Obj::Vector(Vector::Numeric(..)) => {
                    any_numeric = true;
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

        if any_numeric {
            let mut sum: f64 = 0.0;

            for obj in objects {
                match obj {
                    Obj::Vector(vect) => {
                        match vect {
                            Vector::Logical(repr) => {
                                for x in repr.inner().borrow().iter() {
                                    match *x {
                                        OptionNA::NA => {
                                            let rep: Rep<OptionNA<f64>> =
                                                Rep::from(vec![OptionNA::NA]);
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
                                            let rep: Rep<OptionNA<f64>> =
                                                Rep::from(vec![OptionNA::NA]);
                                            return EvalResult::Ok(Obj::Vector(Vector::from(rep)));
                                        }
                                        OptionNA::Some(x) => sum += x as f64,
                                    }
                                }
                            }
                            Vector::Numeric(repr) => {
                                for x in repr.inner().borrow().iter() {
                                    match *x {
                                        OptionNA::NA => {
                                            let rep: Rep<OptionNA<f64>> =
                                                Rep::from(vec![OptionNA::NA]);
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
            EvalResult::Ok(Obj::Vector(Vector::from(Rep::from(vec![sum]))))
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
                                            let rep: Rep<OptionNA<i32>> =
                                                Rep::from(vec![OptionNA::NA]);
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
                                            let rep: Rep<OptionNA<i32>> =
                                                Rep::from(vec![OptionNA::NA]);
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
            EvalResult::Ok(Obj::Vector(Vector::from(Rep::from(vec![sum]))))
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

    // FIXME: Overly aggressive conversion to Numeric, representations for NAs
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
    fn sum_na_numeric() {
        assert_eq!(r! {sum(NA, 1)}, r! {NA * 1});
    }

    #[test]
    fn sum_null() {
        assert!((r! {sum(null)}).is_err());
    }

    #[test]
    fn sum_numeric() {
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
    fn sum_integer_numeric() {
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
    fn sum_numeric_logical() {
        assert_eq!(r! {{"sum(c(1, 4, 5), false, true)"}}, r! {{"11"}},)
    }

    #[test]
    fn sum_integer_numeric_logical() {
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
