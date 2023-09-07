use r_derive::*;

use crate::ast::*;
use crate::lang::*;
use crate::vector::*;
use crate::callable::core::*;
use crate::vector::types::OptionNa;

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "c")]
pub struct PrimitiveC;
impl Callable for PrimitiveC {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        // this can be cleaned up quite a bit, but I just need it working with
        // more types for now to test vectorized operators using different types

        let R::List(vals) = stack.eval_list_eager(args)? else {
            unreachable!()
        };

        // until there's a better way of handling type hierarchy, this will do
        let t: u8 = vals
            .iter()
            .map(|(_, v)| match v {
                R::Null => 0,
                R::Vector(vec) => match vec {
                    Vector::Logical(_) => 1,
                    Vector::Integer(_) => 2,
                    Vector::Numeric(_) => 3,
                    Vector::Character(_) => 4,
                },
                R::List(_) => 5,
                _ => 0,
            })
            .fold(0, std::cmp::max);

        match t {
            0 => Ok(R::Null),
            // Coerce everything into logical
            1 => {
                let mut output = vec![OptionNa(Some(true)); 0];
                for (_, val) in vals {
                    match val {
                        R::Null => continue,
                        R::Vector(Vector::Logical(mut v)) => output.append(&mut v),
                        _ => unimplemented!(),
                    }
                }
                Ok(R::Vector(Vector::Logical(output)))
            }
            // Coerce everything into integer
            2 => {
                let mut output = vec![OptionNa(Some(0)); 0];
                for (_, val) in vals {
                    match val {
                        R::Null => continue,
                        R::Vector(Vector::Integer(mut v)) => output.append(&mut v),
                        R::Vector(Vector::Logical(v)) => {
                            output.append(&mut Vector::vec_coerce::<bool, i32>(&v))
                        }
                        _ => unimplemented!(),
                    }
                }
                Ok(R::Vector(Vector::Integer(output)))
            }
            // Coerce everything into numeric
            3 => {
                let mut output = vec![OptionNa(Some(0.0)); 0];
                for (_, val) in vals {
                    match val {
                        R::Null => continue,
                        R::Vector(Vector::Numeric(mut v)) => output.append(&mut v),
                        R::Vector(Vector::Integer(v)) => {
                            output.append(&mut Vector::vec_coerce::<i32, f64>(&v))
                        }
                        R::Vector(Vector::Logical(v)) => {
                            output.append(&mut Vector::vec_coerce::<bool, f64>(&v))
                        }
                        _ => unimplemented!("{:#?}", val)
                    }
                }
                Ok(R::Vector(Vector::Numeric(output)))
            }
            // coerce everything into strings
            4 => {
                let mut output = vec![OptionNa(Some("".to_string())); 0];
                for (_, val) in vals {
                    match val {
                        R::Null => continue,
                        R::Vector(Vector::Numeric(v)) => {
                            output.append(&mut Vector::vec_coerce::<f64, String>(&v))
                        }
                        R::Vector(Vector::Integer(v)) => {
                            output.append(&mut Vector::vec_coerce::<i32, String>(&v))
                        }
                        R::Vector(Vector::Logical(v)) => {
                            output.append(&mut Vector::vec_coerce::<bool, String>(&v))
                        }
                        R::Vector(Vector::Character(mut v)) => output.append(&mut v),
                        _ => unimplemented!("{:#?}", val)
                    }
                }
                Ok(R::Vector(Vector::Character(output)))
            }
            _ => Ok(R::Null),
        }
    }
}