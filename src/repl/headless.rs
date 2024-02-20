use std::rc::Rc;
use wasm_bindgen::prelude::*;

use super::release::session_header;
use crate::cli::Cli;
use crate::context::Context;
use crate::lang::{CallStack, Cond, Signal};
use crate::object::Environment;
use crate::parser::*;

#[wasm_bindgen]
pub fn wasm_cli_args(warranty: bool, locale: Option<String>) -> Cli {
    use std::str::FromStr;

    Cli {
        locale: FromStr::from_str(&locale.unwrap_or("".to_string())).unwrap_or_default(),
        warranty,
    }
}

#[wasm_bindgen]
pub fn wasm_session_header(args: &Cli) -> String {
    session_header(args.warranty, &args.locale)
}

#[wasm_bindgen]
pub fn wasm_runtime(args: &Cli) -> JsValue {
    let local_args: Cli = args.clone();
    let global_env = Rc::new(Environment {
        parent: Some(Environment::from_builtins()),
        ..Default::default()
    });

    let cb = Closure::<dyn Fn(String) -> Option<String>>::new(move |line: String| {
        wasm_eval_in(&local_args, &global_env, line.as_str())
    });

    let ret = cb.as_ref().clone();
    cb.forget();
    ret
}

#[wasm_bindgen]
pub fn wasm_parses_successfully(args: Cli, input: &str) -> bool {
    args.locale.parse_input(input).is_ok()
}

pub fn wasm_eval_in(args: &Cli, env: &Rc<Environment>, input: &str) -> Option<String> {
    match args.locale.parse_input(input) {
        Ok(expr) => {
            let mut stack = CallStack::from(env.clone());
            match stack.eval_and_finalize(expr) {
                Err(Signal::Condition(Cond::Terminate)) => None,
                Ok(val) => Some(format!("{val}")),
                Err(e) => Some(format!("{e}")),
            }
        }
        Err(Signal::Thunk) => None,
        Err(e) => Some(format!("{e}")),
    }
}
