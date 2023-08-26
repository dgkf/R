use r_derive::Primitive;

use crate::ast::*;
use crate::lang::*;
use crate::vector::vectors::*;
use crate::callable::core::*;

#[derive(Debug, Clone, Primitive)]
pub struct PrimitiveC;

impl PrimitiveSYM for PrimitiveC {
    const SYM: &'static str = "c";
}

impl Callable for PrimitiveC {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        // this can be cleaned up quite a bit, but I just need it working with
        // more types for now to test vectorized operators using different types

        let R::List(vals) = stack.eval_list(args)? else {
            unreachable!()
        };

        let vals = force_closures(vals, stack);

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
                let mut output = vec![OptionNA::Some(true); 0];
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
                let mut output = vec![OptionNA::Some(0); 0];
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
                let mut output = vec![OptionNA::Some(0.0); 0];
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
                let mut output = vec![OptionNA::Some("".to_string()); 0];
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