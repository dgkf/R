use wasm_bindgen::prelude::*;
use std::rc::Rc;

use crate::lang::{CallStack, Context, Environment, RSignal, Cond};
use crate::parser::parse;
use super::release::session_header;

#[wasm_bindgen]
pub fn wasm_session_header() -> String {
    session_header()
}

#[wasm_bindgen]
pub fn wasm_env() -> JsValue {
    let env = Rc::new(Environment::default());
    let cb = Closure::<dyn Fn(String) -> Option<String>>::new(move |line: String| wasm_eval_in(&env, line.as_str()));
    let ret = cb.as_ref().clone();
    cb.forget();
    ret
}

pub fn wasm_eval_in(env: &Rc<Environment>, input: &str) -> Option<String> {
    match parse(&input) {
        Ok(expr) => {
            let mut stack = CallStack::from(env.clone());
            match stack.eval(expr) {
                Err(RSignal::Condition(Cond::Terminate)) => None,
                Ok(val) => Some(format!("{}", val)),
                Err(e) => Some(format!("{}", e)),
            }
        }
        Err(e) => Some(format!("{}", e))
    }
}