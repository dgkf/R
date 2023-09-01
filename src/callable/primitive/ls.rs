use std::collections::HashSet;

use r_derive::*;

use crate::ast::*;
use crate::callable::builtins::BUILTIN;
use crate::error::RError;
use crate::lang::*;
use crate::callable::core::*;
use crate::vector::vectors::Vector;

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "ls")]
pub struct PrimitiveLs;
impl Callable for PrimitiveLs {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let mut names: HashSet<String> = HashSet::new();

        // handling args to look outside current environment currently out of 
        // scope
        if args.len() > 0 {
            return RError::Unimplemented(Some("ls".to_string())).into();
        }

        // look for values defined in the current or parent environments
        let mut env = stack.env();
        loop {
            for name in env.values.borrow().keys() {
                names.insert(name.clone());
            }

            if let Some(parent) = env.parent.clone() {
                env = parent;
            } else {
                break 
            };
        };

        // once we hit the top environment, add in builtins
        for name in BUILTIN.keys() {
            names.insert(name.to_string());
        }

        let mut res = names.into_iter().collect::<Vec<String>>();
        res.sort();

        Ok(R::Vector(Vector::from(res)))
    }
}
