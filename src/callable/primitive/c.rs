use r_derive::*;

use crate::callable::core::*;
use crate::context::Context;
use crate::lang::*;
use crate::object::types::*;
use crate::object::*;

/// Concatenate Values
///
/// Construct a vector of values. Heterogeneous values will be coerced
/// into a common type.
///
/// <div class="warning">
///
/// Note that `c()` is provided for familiarity, but there are better,
/// more explicit ways of constructing homogeneous vectors using the
/// `[...]` and `(...,)` syntax.
///
/// </div>
///
/// # In-Language
///
/// ## Usage
///
/// ```custom,{class=r}
/// c(...)
/// ```
///
/// ## Arguments
///
/// `...`: Arguments to collect into a `vector`.
///
/// ## Examples
///
/// ```custom,{class=r-repl}
/// c(false, 1, "two")
/// ```
///
#[doc(alias = "c")]
#[builtin(sym = "c")]
#[derive(Debug, Clone, PartialEq)]
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
            .pairs()
            .iter()
            .map(|(_, v)| match v {
                Obj::Null => 0,
                Obj::Vector(_) => 1,
                Obj::List(_) => 2,
                _ => 0,
            })
            .fold(0, std::cmp::max);

        // if the output will have names
        let named = vals.pairs().iter().any(|(n, o)| {
            if matches!(n, OptionNA::Some(_)) {
                return true;
            }
            match o {
                Obj::Vector(v) => v.is_named(),
                Obj::List(l) => l.is_named(),
                _ => todo!(),
            }
        });

        // most complex type was NULL
        if ty == 0 {
            return Ok(Obj::Null);
        }

        // most complex type was List
        // FIXME: handle names
        if ty == 2 {
            return Ok(Obj::List(vals));
        }

        let names: Option<Vec<Character>> = if named {
            let mut pairs = vals.pairs();
            let nms = pairs.iter().flat_map(|(name, obj)| {
                let maybe_prefix = name.clone().as_option().clone();

                if let Obj::Vector(v) = obj {
                    let maybe_names_iter = v.iter_names();

                    let x: Vec<Character> = match (maybe_prefix, maybe_names_iter) {
                        (None, None) => std::iter::repeat(Character::NA).take(v.len()).collect(),
                        (Some(prefix), None) => std::iter::repeat(Character::Some(prefix.clone()))
                            .take(v.len())
                            .collect(),
                        (None, Some(names_iter)) => names_iter.collect(),
                        (Some(prefix), Some(names_iter)) => names_iter
                            .map(|maybe_name| {
                                if let OptionNA::Some(name) = maybe_name {
                                    Character::Some(format!("{}.{}", prefix, name))
                                } else {
                                    Character::Some(prefix.clone())
                                }
                            })
                            .collect(),
                    };
                    x
                } else {
                    unimplemented!()
                }
            });
            Some(nms.collect())
        } else {
            None
        };

        let ret = vals
            .pairs()
            .iter()
            .map(|(_, r)| match r {
                Obj::Vector(Vector::Logical(_)) => Vector::from(Vec::<Logical>::new()),
                Obj::Vector(Vector::Integer(_)) => Vector::from(Vec::<Integer>::new()),
                Obj::Vector(Vector::Double(_)) => Vector::from(Vec::<Double>::new()),
                Obj::Vector(Vector::Character(_)) => Vector::from(Vec::<Character>::new()),
                _ => unreachable!(),
            })
            .fold(Vector::from(Vec::<Logical>::new()), |l, r| match (l, r) {
                (v @ Vector::Character(_), _) => v,
                (_, v @ Vector::Character(_)) => v,
                (v @ Vector::Double(_), _) => v,
                (_, v @ Vector::Double(_)) => v,
                (v @ Vector::Integer(_), _) => v,
                (_, v @ Vector::Integer(_)) => v,
                (v @ Vector::Logical(_), _) => v,
            });

        // consume values and merge into a new collection
        let v =
            match ret {
                Vector::Character(_) => Vector::from(
                    Vec::<OptionNA<String>>::new()
                        .into_iter()
                        .chain(vals.pairs().iter().flat_map(
                            |(_, i)| match i.clone().as_character() {
                                Ok(Obj::Vector(Vector::Character(v))) => v.into_iter_values(),
                                _ => unreachable!(),
                            },
                        ))
                        .collect::<Vec<Character>>(),
                ),
                Vector::Double(_) => {
                    Vector::from(
                        Vec::<OptionNA<f64>>::new()
                            .into_iter()
                            .chain(vals.pairs().iter().flat_map(
                                |(_, i)| match i.clone().as_double() {
                                    Ok(Obj::Vector(Vector::Double(v))) => v.into_iter_values(),
                                    _ => unreachable!(),
                                },
                            ))
                            .collect::<Vec<Double>>(),
                    )
                }
                Vector::Integer(_) => Vector::from(
                    Vec::<OptionNA<i32>>::new()
                        .into_iter()
                        .chain(vals.pairs().iter().flat_map(
                            |(_, i)| match i.clone().as_integer() {
                                Ok(Obj::Vector(Vector::Integer(v))) => v.into_iter_values(),
                                _ => unreachable!(),
                            },
                        ))
                        .collect::<Vec<Integer>>(),
                ),
                Vector::Logical(_) => Vector::from(
                    Vec::<OptionNA<bool>>::new()
                        .into_iter()
                        .chain(vals.pairs().iter().flat_map(
                            |(_, i)| match i.clone().as_logical() {
                                Ok(Obj::Vector(Vector::Logical(v))) => v.into_iter_values(),
                                _ => unreachable!(),
                            },
                        ))
                        .collect::<Vec<Logical>>(),
                ),
            };

        if let Some(names) = names {
            v.set_names_(names.into())
        }
        Ok(Obj::Vector(v))
    }
}
