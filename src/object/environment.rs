use core::fmt;
use std::cell::RefCell;
use std::fmt::Display;
use std::collections::HashMap;
use std::rc::Rc;

use crate::callable::builtins::BUILTIN;

use super::{Obj, Expr, ExprList};

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
}

impl Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<environment {:?}", self.values.as_ptr())?;

        // // print defined variable names
        // if self.values.borrow().len() > 0 { write!(f, " [")?; }
        // for (i, k) in self.values.borrow().keys().enumerate() {
        //     if i > 0 { write!(f, ", ")?; }
        //     write!(f, "{}", k)?;
        // }
        // if self.values.borrow().len() > 0 { write!(f, "]")?; }

        write!(f, ">")?;
        Ok(())
    }
}
