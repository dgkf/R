use wasm_bindgen::prelude::*;
use std::rc::Rc;

use crate::lang::{CallStack, Signal, Cond};
use crate::context::Context;
use crate::object::Environment;
use pest::Parser;
use crate::parser::{RParser, parse, Rule};
use super::release::session_header;

#[wasm_bindgen]
pub fn wasm_session_header() -> String {
    session_header()
}

#[wasm_bindgen]
pub fn wasm_env() -> JsValue {
    let global_env = Rc::new(Environment {
        parent: Some(Environment::from_builtins()),
        ..Default::default()        
    });

    let cb = Closure::<dyn Fn(String) -> Option<String>>::new(move |line: String| 
        wasm_eval_in(&global_env, line.as_str())
    );

    let ret = cb.as_ref().clone();
    cb.forget();
    ret
}

#[wasm_bindgen]
pub fn wasm_parses_successfully(input: &str) -> bool {
    RParser::parse(Rule::repl, input).is_ok()
}

pub fn wasm_eval_in(env: &Rc<Environment>, input: &str) -> Option<String> {
    match parse(&input) {
        Ok(expr) => {
            let mut stack = CallStack::from(env.clone());
            match stack.eval_and_finalize(expr) {
                Err(Signal::Condition(Cond::Terminate)) => None,
                Ok(val) => Some(format!("{val}")),
                Err(e) => Some(format!("{e}")),
            }
        }
        Err(Signal::Thunk) => None,
        Err(e) => Some(format!("{e}"))
    }
}
