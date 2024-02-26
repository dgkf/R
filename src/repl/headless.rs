use std::rc::Rc;
use wasm_bindgen::prelude::*;

use super::release::session_header;
use crate::cli::{Cli, Experiment};
use crate::context::Context;
use crate::lang::{CallStack, Cond, Signal};
use crate::object::Environment;
use crate::parser::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn wasm_args(js: JsValue) -> Cli {
    use gloo_utils::format::JsValueSerdeExt;
    js.into_serde().unwrap_or_default()
}

#[wasm_bindgen]
pub fn wasm_session_header(args: JsValue) -> String {
    let args = wasm_args(args);
    session_header(args.warranty, &args.locale)
}

#[wasm_bindgen]
pub fn wasm_runtime(args: JsValue) -> JsValue {
    let args = wasm_args(args);
    log(&format!("Launching runtime with args: {args:?}"));

    crate::experiments::use_tail_calls(Some(args.experiments.contains(&Experiment::TailCalls)));
    crate::experiments::use_rest_args(Some(args.experiments.contains(&Experiment::RestArgs)));

    let global_env = Rc::new(Environment {
        parent: Some(Environment::from_builtins()),
        ..Default::default()
    });

    let cb = Closure::<dyn Fn(String) -> Option<String>>::new(move |line: String| {
        wasm_eval_in(&args, &global_env, line.as_str())
    });

    let ret = cb.as_ref().clone();
    cb.forget();
    ret
}

#[wasm_bindgen]
pub fn wasm_parses_successfully(args: JsValue, input: &str) -> bool {
    let args = wasm_args(args);
    args.locale.parse_input(input).is_ok()
}

/// returns a stream of strings. Each pair represents a style and text
#[wasm_bindgen]
pub fn wasm_highlight(args: JsValue, input: &str) -> Vec<JsValue> {
    let args = wasm_args(args);
    args.locale
        .parse_highlight(input)
        .unwrap_or_default()
        .into_iter()
        .flat_map(|(text, style)| vec![style.to_string(), text.to_string()])
        .map(JsValue::from)
        .collect()
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
