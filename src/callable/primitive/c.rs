use r_derive::*;

use crate::callable::core::*;
use crate::context::Context;
use crate::lang::*;
use crate::object::types::*;
use crate::object::*;

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "c")]
pub struct PrimitiveC;
impl Callable for PrimitiveC {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        // this can be cleaned up quite a bit, but I just need it working with
        // more types for now to test vectorized operators using different types

        let Obj::List(vals) = stack.eval_list_eager(args)? else {
            unreachable!()
        };

        // lets first see what we're aiming to build.
        let ty: u8 = vals
            .values
            .borrow()
            .iter()
            .map(|(_, v)| match v {
                Obj::Null => 0,
                Obj::Vector(_) => 1,
                Obj::List(_) => 2,
                _ => 0,
            })
            .fold(0, std::cmp::max);

        // most complex type was NULL
        if ty == 0 {
            return Ok(Obj::Null);
        }

        // most complex type was List
        if ty == 2 {
            return Ok(Obj::List(vals));
        }

        // otherwise, try to collapse vectors into same type
        let ret = vals
            .values
            .borrow()
            .iter()
            .map(|(_, r)| match r {
                Obj::Vector(Vector::Logical(_)) => Vector::from(Vec::<Logical>::new()),
                Obj::Vector(Vector::Integer(_)) => Vector::from(Vec::<Integer>::new()),
                Obj::Vector(Vector::Numeric(_)) => Vector::from(Vec::<Numeric>::new()),
                Obj::Vector(Vector::Character(_)) => Vector::from(Vec::<Character>::new()),
                _ => unreachable!(),
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
            Vector::Character(v) => Ok(Obj::Vector(Vector::from(
                v.inner()
                    .clone()
                    .borrow_mut()
                    .clone()
                    .into_iter()
                    .chain(
                        vals.values
                            .borrow_mut()
                            .clone()
                            .into_iter()
                            .flat_map(|(_, i)| match i.as_character() {
                                Ok(Obj::Vector(Vector::Character(v))) => {
                                    v.inner().clone().borrow().clone().into_iter()
                                }
                                _ => unreachable!(),
                            }),
                    )
                    .collect::<Vec<Character>>(),
            ))),
            Vector::Numeric(v) => Ok(Obj::Vector(Vector::from(
                v.inner()
                    .clone()
                    .borrow_mut()
                    .clone()
                    .into_iter()
                    .chain(
                        vals.values
                            .borrow_mut()
                            .clone()
                            .into_iter()
                            .flat_map(|(_, i)| match i.as_numeric() {
                                Ok(Obj::Vector(Vector::Numeric(v))) => {
                                    v.inner().clone().borrow().clone().into_iter()
                                }
                                _ => unreachable!(),
                            }),
                    )
                    .collect::<Vec<Numeric>>(),
            ))),
            Vector::Integer(v) => Ok(Obj::Vector(Vector::from(
                v.inner()
                    .clone()
                    .borrow_mut()
                    .clone()
                    .into_iter()
                    .chain(
                        vals.values
                            .borrow_mut()
                            .clone()
                            .into_iter()
                            .flat_map(|(_, i)| match i.as_integer() {
                                Ok(Obj::Vector(Vector::Integer(v))) => {
                                    v.inner().clone().borrow().clone().into_iter()
                                }
                                _ => unreachable!(),
                            }),
                    )
                    .collect::<Vec<Integer>>(),
            ))),
            Vector::Logical(v) => Ok(Obj::Vector(Vector::from(
                v.inner()
                    .clone()
                    .borrow_mut()
                    .clone()
                    .into_iter()
                    .chain(
                        vals.values
                            .borrow_mut()
                            .clone()
                            .into_iter()
                            .flat_map(|(_, i)| match i.as_logical() {
                                Ok(Obj::Vector(Vector::Logical(v))) => {
                                    v.inner().clone().borrow().clone().into_iter()
                                }
                                _ => unreachable!(),
                            }),
                    )
                    .collect::<Vec<Logical>>(),
            ))),
        }
    }
}
