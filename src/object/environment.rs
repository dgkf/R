use core::fmt;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;

use crate::callable::builtins::BUILTIN;
use crate::context::Context;
use crate::error::Error;
use crate::lang::EvalResult;

use super::{Expr, ExprList, Obj};

#[derive(Default, Clone, PartialEq)]
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

    pub fn insert(&self, name: String, value: Obj) {
        self.values.borrow_mut().insert(name, value);
    }

    pub fn append(&self, values: Obj) {
        match values {
            Obj::List(x) => {
                for (key, value) in x.values.borrow().iter() {
                    if let Some(name) = key {
                        self.values.borrow_mut().insert(name.clone(), value.clone());
                    } else {
                        println!("Dont' know what to do with value...")
                    }
                }
            }
            _ => unimplemented!(),
        }
    }

    pub fn get(&self, name: String) -> EvalResult {
        // search in this environment for value by name
        if let Some(value) = self.values.borrow().get(&name) {
            Ok(value.clone())

        // if not found, search through parent if available
        } else if let Some(parent) = &self.parent {
            parent.clone().get(name)

        // if we're at the top level, fall back to primitives if available
        } else if let Ok(prim) = name.as_str().try_into() {
            Ok(Obj::Function(
                ExprList::new(),
                Expr::Primitive(prim),
                Rc::new(self.clone()), // TODO(bug): will this retain shared ref?
            ))

        // otherwise, throw error
        } else {
            Err(Error::VariableNotFound(name).into())
        }
    }
}

impl Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<environment {:?}>", self.values.as_ptr())
    }
}

impl std::fmt::Debug for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Environment {{ values: {:?}, parent: {:?} }}",
            self.values.as_ptr(),
            self.parent.clone().map(|i| i.values.as_ptr()),
        )?;

        // print defined variable names
        if self.values.borrow().len() > 0 {
            write!(f, " [")?;
        }

        for (i, k) in self.values.borrow().keys().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", k)?;
        }

        if self.values.borrow().len() > 0 {
            write!(f, "]")?;
        }

        Ok(())
    }
}
