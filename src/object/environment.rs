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
        self.get_internal(name).0
    }

    fn get_internal(&self, name: String) -> (EvalResult, bool) {
        let mut caller_env = true;

        let mut env = self;

        loop {
            if let Some(value) = env.values.borrow().get(&name) {
                let result = value.clone();

                let x = match result {
                    Obj::Promise(None, expr, env) => env.clone().eval(expr),
                    Obj::Promise(Some(result), ..) => Ok(*result),
                    _ => Ok(result),
                };

                return (x, caller_env);

            // if not found, search through parent if available
            } else if let Some(parent) = &env.parent {
                caller_env = false;
                env = parent;
                continue;

            // if we're at the top level, fall back to primitives if available
            } else if let Ok(prim) = name.as_str().try_into() {
                let x = Ok(Obj::Function(
                    ExprList::new(),
                    Expr::Primitive(prim),
                    Rc::new(self.clone()), // TODO(bug): will this retain shared ref?
                ));

                return (x, caller_env);

            // otherwise, throw error
            } else {
                return (Err(Error::VariableNotFound(name).into()), caller_env);
            }
        }
    }

    pub fn get_mutable(&self, name: String) -> EvalResult {
        let (x, caller_env) = self.get_internal(name.clone());
        let x = x?;

        // It was found in the current environment we don't have to bind it, as we are
        // allowed to mutate it directly
        if caller_env {
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
