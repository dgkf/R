use core::fmt;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;

use crate::callable::builtins::BUILTIN;
use crate::context::Context;
use crate::error::Error;
use crate::lang::EvalResult;

use super::{Expr, ExprList, List, Obj};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Environment {
    pub values: RefCell<HashMap<String, Obj>>,
    pub parent: Option<Rc<Environment>>,
}

impl Environment {
    pub fn from_builtins() -> Rc<Environment> {
        let env = Rc::new(Environment::default());
        for (name, builtin) in BUILTIN.iter() {
            let builtin_fn = Obj::Function(
                ExprList::new(),
                Expr::Primitive(builtin.clone()),
                env.clone(),
            );

            env.insert(String::from(*name), builtin_fn);
        }
        env
    }

    pub fn len(&self) -> usize {
        self.values.borrow().len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn insert(&self, name: String, value: Obj) {
        self.values.borrow_mut().insert(name, value);
    }

    pub fn append(&self, l: List) {
        for (key, value) in l.values.borrow().iter() {
            if let Some(name) = key {
                self.values.borrow_mut().insert(name.clone(), value.clone());
            }
        }
    }

    pub fn get(&self, name: String) -> EvalResult {
        self.get_internal(name, 0).0
    }

    fn get_internal(&self, name: String, i: u32) -> (EvalResult, i32) {
        if let Some(value) = self.values.borrow().get(&name) {
            let result = value.clone();

            let mut is_promise = false;
            let x = match result {
                Obj::Promise(None, expr, env) => {
                    is_promise = true;
                    env.clone().eval(expr)
                }
                Obj::Promise(Some(result), ..) => {
                    is_promise = true;
                    Ok(*result)
                }
                _ => Ok(result),
            };

            if is_promise {
                (x, -3)
            } else {
                (x, i as i32)
            }
        // if not found, search through parent if available
        } else if let Some(parent) = &self.parent {
            parent.clone().get_internal(name, i + 1)

        // if we're at the top level, fall back to primitives if available
        } else if let Ok(prim) = name.as_str().try_into() {
            let x = Ok(Obj::Function(
                ExprList::new(),
                Expr::Primitive(prim),
                Rc::new(self.clone()), // TODO(bug): will this retain shared ref?
            ));

            (x, -1)

        // otherwise, throw error
        } else {
            (Err(Error::VariableNotFound(name).into()), -2)
        }
    }

    pub fn get_mutable(&self, name: String) -> EvalResult {
        let (x, i) = self.get_internal(name.clone(), 0);
        let x = x?;

        // It was found in the current environment we don't have to bind it, as we are
        // allowed to mutate it directly

        if i == 0 {
            return EvalResult::Ok(x.mutable_view());
        }
        // If it was found in a parent environment or is a promise,
        // we first bind it in the current environment
        // so we don't accidentally change variables in the parent environment

        let xc = x.lazy_copy();
        let xm = xc.mutable_view();
        self.insert(name, xc);

        EvalResult::Ok(xm)
    }
}

impl Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<environment {:?}", self.values.as_ptr())?;

        // // print defined variable names
        // if self.values.borrow().len() > 0 {
        //     write!(f, " [")?;
        // }
        // for (i, k) in self.values.borrow().keys().enumerate() {
        //     if i > 0 {
        //         write!(f, ", ")?;
        //     }
        //     write!(f, "{}", k)?;
        // }
        // if self.values.borrow().len() > 0 {
        //     write!(f, "]")?;
        // }

        write!(f, ">")?;
        Ok(())
    }
}

#[cfg(test)]

mod tests {
    use crate::{r, r_expect};
    #[test]
    fn vectors_are_mutable() {
        r_expect! {{"
            x = 1
            x[1] = 2
            x == 2
        "}}
    }

    #[test]
    fn dont_mutate_value_from_parent() {
        r_expect! {{"
            f = fn() x[1] <- -99
            x = 10
            f()
            x == 10
        "}}
    }
    #[test]
    fn promises_can_be_mutated() {
        r_expect! {{"
             f = fn(x) {
               x[1] <- -99
               x
             }
             x1 = 10
             x2 = f(x1)
             (x1 == 10) && x2 == -99
         "}}
    }
}
