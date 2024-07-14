use core::fmt;
use hashbrown::HashMap;
use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

use crate::callable::builtins::BUILTIN;
use crate::context::Context;
use crate::error::Error;
use crate::lang::{EvalResult, Signal};

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
        let (x, _) = self.find(name.clone())?;
        EvalResult::Ok(x.lazy_copy())
    }

    /// Find a variable in the environment or one of its parents.
    /// If the variable is found, a mutable view on it is returned.
    pub fn find(&self, name: String) -> Result<(Obj, Rc<Environment>), Signal> {
        let mut env = self;

        loop {
            if let Some(value) = env.values.borrow().get(&name) {
                let result = value.mutable_view();

                let x = match result {
                    Obj::Promise(None, expr, env) => env.clone().eval(expr)?,
                    Obj::Promise(Some(result), ..) => *result,
                    _ => result,
                };

                return Result::Ok((x, Rc::new(env.clone())));

            // if not found, search through parent if available
            } else if let Some(parent) = &env.parent {
                env = parent;
                continue;

            // if we're at the top level, fall back to primitives if available
            } else if let Ok(prim) = name.as_str().try_into() {
                let x = Obj::Function(
                    ExprList::new(),
                    Expr::Primitive(prim),
                    Rc::new(self.clone()), // TODO(bug): will this retain shared ref?
                );

                return Result::Ok((x, Rc::new(env.clone())));

            // otherwise, throw error
            } else {
                return Result::Err(Signal::Error(Error::VariableNotFound(name)));
            }
        }
    }

    pub fn get_mut(&self, name: String) -> EvalResult {
        let (x, env) = self.find(name.clone())?;
        if *self == *env {
            return EvalResult::Ok(x.mutable_view());
        }

        // we found it in the parent environment, which means we first have to find it in the
        // current environment so we then modify it in the correct scope
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
