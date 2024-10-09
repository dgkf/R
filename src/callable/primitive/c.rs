use r_derive::*;

use crate::callable::core::*;
use crate::context::Context;
use crate::object::types::*;
use crate::object::*;
use crate::{formals, lang::*};

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

formals!(PrimitiveC, "(...)");

impl Callable for PrimitiveC {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        // this can be cleaned up quite a bit, but I just need it working with
        // more types for now to test vectorized operators using different types

        let Obj::List(vals) = stack.eval_list_eager(args)? else {
            unreachable!()
        };

        // lets first see what we're aiming to build.
        let ty: u8 = vals
            .pairs_ref()
            .iter()
            .map(|(_, v)| match v {
                Obj::Null => 0,
                Obj::Vector(_) => 1,
                _ => 2,
            })
            .fold(0, std::cmp::max);

        // if the output will have names
        // either an argument was passed via a name or it has names itself
        let named = vals.pairs_ref().iter().any(|(n, o)| {
            if matches!(n, OptionNA::Some(_)) {
                return true;
            }
            match o {
                Obj::Vector(v) => v.is_named(),
                Obj::List(l) => l.is_named(),
                _ => false,
            }
        });

        // most complex type was NULL
        if ty == 0 {
            return Ok(Obj::Null);
        }

        // most complex type was List
        if ty == 2 {
            // TODO: We should use size hints here.
            let list = List::new();
            for (name1, value1) in vals.iter_pairs() {
                match value1 {
                    Obj::List(x) => {
                        for (name2, value2) in x.iter_pairs() {
                            let name = match (&name1, name2) {
                                (OptionNA::Some(x1), OptionNA::Some(x2)) => {
                                    OptionNA::Some(format!("{x1}.{x2}"))
                                }
                                (OptionNA::NA, OptionNA::Some(x2)) => OptionNA::Some(x2),
                                (OptionNA::Some(_), OptionNA::NA) => name1.clone(),
                                (OptionNA::NA, OptionNA::NA) => OptionNA::NA,
                            };

                            list.push_named(name, value2)
                        }
                    }
                    _ => list.push_named(name1, value1),
                }
            }
            return Ok(Obj::List(list));
        }

        let names: Option<Vec<Character>> = if named {
            let mut pairs = vals.pairs_ref();
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
                    unreachable!("if we are not building a list, all elements are vectors")
                }
            });
            Some(nms.collect())
        } else {
            None
        };

        // otherwise, try to collapse vectors into same type
        let ret = vals
            .values_ref()
            .iter()
            .map(|r| match r {
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
        let v = match ret {
            Vector::Character(_) => Vector::from(
                Vec::<OptionNA<String>>::new()
                    .into_iter()
                    .chain(vals.iter_values().flat_map(|i| match i.as_character() {
                        Ok(Obj::Vector(Vector::Character(v))) => v.iter_values(),
                        _ => unreachable!(),
                    }))
                    .collect::<Vec<Character>>(),
            ),
            Vector::Double(_) => Vector::from(
                Vec::<OptionNA<f64>>::new()
                    .into_iter()
                    .chain(
                        vals.iter_values()
                            .flat_map(|i| match i.clone().as_double() {
                                Ok(Obj::Vector(Vector::Double(v))) => v.iter_values(),
                                _ => unreachable!(),
                            }),
                    )
                    .collect::<Vec<Double>>(),
            ),
            Vector::Integer(_) => Vector::from(
                Vec::<OptionNA<i32>>::new()
                    .into_iter()
                    .chain(
                        vals.iter_values()
                            .flat_map(|i| match i.clone().as_integer() {
                                Ok(Obj::Vector(Vector::Integer(v))) => v.iter_values(),
                                _ => unreachable!(),
                            }),
                    )
                    .collect::<Vec<Integer>>(),
            ),
            Vector::Logical(_) => Vector::from(
                Vec::<OptionNA<bool>>::new()
                    .into_iter()
                    .chain(
                        vals.iter_values()
                            .flat_map(|i| match i.clone().as_logical() {
                                Ok(Obj::Vector(Vector::Logical(v))) => v.iter_values(),
                                _ => unreachable!(),
                            }),
                    )
                    .collect::<Vec<Logical>>(),
            ),
        };

        if let Some(names) = names {
            v.set_names(names.into())
        }
        Ok(Obj::Vector(v))
    }
}

#[cfg(test)]

mod tests {
    use crate::{r, r_expect};
    #[test]
    fn list_empty() {
        assert_eq!(r!(list()), r!(c(list())))
    }
    #[test]
    fn list_list() {
        r_expect! {{"
            l = c(list(1), list(2))
            l[[1]] == 1 & l[[2]] == 2
        "}}
    }
    #[test]
    fn list_vec() {
        r_expect! {{"
            l = c(list(1), 2:3)
            l[[1]] == 1 & l[[2]][1] == 2 & l[[2]][2] == 3
        "}}
    }
    #[test]
    fn list_fn() {
        r_expect! {{"
            l = c(list(1), fn() 2)
            l[[1]] == 1 & l[[2]]() == 2
        "}}
    }
    #[test]
    fn function() {
        r_expect! {{"
            l = c(fn() 2)
            l[[1]]() == 2
        "}}
    }
    #[test]
    fn list_names_outer() {
        r_expect! {{"
            c(a = list(1))$a == 1
        "}}
    }
    #[test]
    fn list_names_inner() {
        r_expect! {{"
            c(list(a = 1))$a == 1
        "}}
    }
    #[test]
    fn list_names_both() {
        r_expect! {{"
            c(a = list(b = 1))$a.b == 1
        "}}
    }

    #[test]
    fn vector_names() {
        r_expect! {{r#"
            x = [a = 1]
            names(x) == "a"
        "#}}
    }
}
