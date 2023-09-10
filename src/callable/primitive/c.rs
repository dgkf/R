use r_derive::*;

use crate::ast::*;
use crate::lang::*;
use crate::vector::vectors::*;
use crate::callable::core::*;

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

        // lets first see what we're aiming to build.
        let ty: u8 = vals
            .iter()
            .map(|(_, v)| match v {
                R::Null => 0,
                R::Vector(_) => 1,
                R::List(_) => 2,
                _ => 0,
            })
            .fold(0, std::cmp::max);

        // most complex type was NULL
        if ty == 0 { 
            return Ok(R::Null)
        }

        // most complex type was List
        if ty == 2 { 
            return Ok(R::List(vals))
        }

        // otherwise, try to collapse vectors into same type
        let ret = vals.iter()
            .map(|(_, r)| match r {
                R::Vector(Vector::Logical(_)) => Vector::from(Vec::<Logical>::new()),
                R::Vector(Vector::Integer(_)) => Vector::from(Vec::<Integer>::new()),
                R::Vector(Vector::Numeric(_)) => Vector::from(Vec::<Numeric>::new()),
                R::Vector(Vector::Character(_)) => Vector::from(Vec::<Character>::new()),
                _ => unreachable!()
            })
            .fold(Vector::from(Vec::<Logical>::new()), |l, r| match (l, r) {
                (v @ Vector::Character(_), _) => v,
                (_, v @ Vector::Character(_)) => v,
                (v @ Vector::Numeric(_), _) => v,
                (_, v @ Vector::Numeric(_)) => v,
                (v @ Vector::Integer(_), _) => v,
                (_, v @ Vector::Integer(_)) => v,
                (v @ Vector::Logical(_), _) => v,
            });

        // consume values and merge into a new collection
        match ret {
            Vector::Character(v) => {
                Ok(R::Vector(Vector::from(
                    v.inner().clone().borrow_mut().clone().into_iter()
                        .chain(vals.into_iter().flat_map(|(_, i)| match i.as_character() {
                            Ok(R::Vector(Vector::Character(v))) => {
                                v.inner().clone().borrow().clone().into_iter()
                            },
                            _ => unreachable!()
                        }))
                        .map(|i| i.clone())
                        .collect::<Vec<Character>>()
                )))
            },
            Vector::Numeric(v) => {
                Ok(R::Vector(Vector::from(
                    v.inner().clone().borrow_mut().clone().into_iter()
                        .chain(vals.into_iter().flat_map(|(_, i)| match i.as_numeric() {
                            Ok(R::Vector(Vector::Numeric(v))) => {
                                v.inner().clone().borrow().clone().into_iter()
                            },
                            _ => unreachable!()
                        }))
                        .map(|i| i.clone())
                        .collect::<Vec<Numeric>>()
                )))
            },
            Vector::Integer(v) => {
                Ok(R::Vector(Vector::from(
                    v.inner().clone().borrow_mut().clone().into_iter()
                        .chain(vals.into_iter().flat_map(|(_, i)| match i.as_integer() {
                            Ok(R::Vector(Vector::Integer(v))) => {
                                v.inner().clone().borrow().clone().into_iter()
                            },
                            _ => unreachable!()
                        }))
                        .map(|i| i.clone())
                        .collect::<Vec<Integer>>()
                )))
            },
            Vector::Logical(v) => {
                Ok(R::Vector(Vector::from(
                    v.inner().clone().borrow_mut().clone().into_iter()
                        .chain(vals.into_iter().flat_map(|(_, i)| match i.as_logical() {
                            Ok(R::Vector(Vector::Logical(v))) => {
                                v.inner().clone().borrow().clone().into_iter()
                            },
                            _ => unreachable!()
                        }))
                        .map(|i| i.clone())
                        .collect::<Vec<Logical>>()
                )))
            },
        }
        

    }
}