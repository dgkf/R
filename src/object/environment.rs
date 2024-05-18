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
        // search in this environment for value by name
        if let Some(value) = self.values.borrow().get(&name) {
            let result = value.clone();
            match result {
                Obj::Promise(None, expr, env) => env.clone().eval(expr),
                Obj::Promise(Some(result), ..) => Ok(*result),
                _ => Ok(result),
            }

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
    pub fn get_mutable(&self, name: String) -> EvalResult {
        // FIXME: bind a lazy copy in the environment if it was recieved from somewhere else,
        self.get(name).map(|x| x.mutable_view())
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
    use super::*;
    use crate::object::vector::rep::*;
    use crate::object::vector::reptype::*;
    use crate::object::vector::*;
    use crate::object::*;
    use crate::r;
    use std::borrow::BorrowMut;
    use std::cell::RefCell;
    use std::collections::HashMap;
    #[test]
    fn get_mutable() {
        let e = Environment {
            values: RefCell::new(HashMap::<String, Obj>::new()),
            parent: None,
        };

        let v = r! {[true, false]}.unwrap();

        e.insert("x".to_string(), v);

        {
            let em = e.get_mutable("x".to_string()).unwrap();

            if let Obj::Vector(Vector::Logical(Rep(v))) = em {
                let vc = v.clone().into_inner();
                match vc {
                    RepType::Subset(v, _) => {
                        let x = &mut *v.borrow_mut();
                        // let x = &mut *v.borrow_mut();
                        assert_eq!(Rc::strong_count(x), 1);
                        let mut xm = Rc::make_mut(x);
                        xm.push(OptionNA::Some(true))
                    }
                }
            }
        }

        assert_eq!(
            r! {[true, false, true]}.unwrap(),
            e.get("x".to_string()).unwrap()
        )

        // let e = Environment { RefCell::new(HashMap::new()), None};
    }
}
