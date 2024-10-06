use core::fmt;
use hashbrown::HashMap;
use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

use crate::callable::builtins::BUILTIN;
use crate::context::Context;
use crate::error::Error;
use crate::lang::{EvalResult, Signal};
use crate::object::types::Character;
use crate::object::ViewMut;

use super::{Expr, ExprList, List, Obj};

#[derive(Default, Clone, PartialEq)]
pub struct Environment {
    pub values: RefCell<HashMap<String, Obj>>,
    pub parent: Option<Rc<Environment>>,
}

impl fmt::Debug for Environment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Environment")
            .field("map", &self.values)
            .finish()
    }
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
        for (key, value) in l.pairs_ref().iter() {
            if let Character::Some(name) = key {
                self.values.borrow_mut().insert(name.clone(), value.clone());
            }
        }
    }

    pub fn get(&self, name: String) -> EvalResult {
        let (x, _) = self.find(name.clone())?;
        EvalResult::Ok(x.clone())
    }

    /// Find a variable in the environment or one of its parents.
    /// If the variable is found, a mutable view on it is returned.
    pub fn find(&self, name: String) -> Result<(Obj, Rc<Environment>), Signal> {
        let mut env = self;

        loop {
            if let Some(value) = env.values.borrow().get(&name) {
                let result = value.view_mut();

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
            } else if let Some(prim) = BUILTIN.get(name.as_str()) {
                let x = Obj::Function(
                    ExprList::new(),
                    Expr::Primitive(prim.clone()),
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
            return EvalResult::Ok(x.view_mut());
        }

        // we found it in the parent environment, which means we first have to find it in the
        // current environment so we then modify it in the correct scope
        let xc = x.clone();
        let xm = xc.view_mut();
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
    fn dollar() {
        r_expect! {{"
            e = environment()
            e$x = 1
            e$x == 1
        "}}
    }
}
