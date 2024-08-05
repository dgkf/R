use std::rc::Rc;
use wasm_bindgen::prelude::*;

use super::release::session_header;
use crate::cli::Cli;
use crate::context::Context;
use crate::lang::{CallStack, Cond, Signal};
use crate::object::Environment;
use crate::parser::*;
use crate::session::{Session, SessionOutput, SessionParserConfig};

#[wasm_bindgen]
pub struct ParseError {
    start: usize,
    end: usize,
    message: String,
}

#[wasm_bindgen]
impl ParseError {
    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn message(&self) -> String {
        self.message.clone()
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

pub fn wasm_args(js: JsValue) -> Session {
    use gloo_utils::format::JsValueSerdeExt;
    let cli: Cli = js.into_serde().unwrap_or_default();
    let session = cli.into();
    session
}

#[wasm_bindgen]
pub fn wasm_session_header(args: JsValue) -> String {
    let session = wasm_args(args);
    session_header(&session)
}

#[wasm_bindgen]
pub fn wasm_runtime(args: JsValue) -> JsValue {
    // build session, apply callback
    let args = wasm_args(args);
    log(&format!("Launching runtime with args: {args:?}"));

    // build our global environment
    let global_env = Rc::new(Environment {
        parent: Some(Environment::from_builtins()),
        ..Default::default()
    });

    // build a callback to evaluate with a enclosed environment, allows
    // for a callback to be provided to handle stdout
    let cb = Closure::<dyn Fn(_, _) -> Option<String>>::new(
        move |line: String, callback: js_sys::Function| {
            let local_args =
                &args
                    .clone()
                    .with_output(SessionOutput::Callback(Rc::new(move |s| {
                        callback.call1(&JsValue::null(), &JsValue::from(s)).ok();
                    })));

            wasm_eval_in(&local_args, &global_env, line.as_str())
        },
    );

    let ret = cb.as_ref().clone();
    cb.forget();
    ret
}

/// Check whether an input produces parse errors
///
/// Returns Option::None if no errors are found, or
/// Option::Some((start, end, message)) when an error is produced.
///
#[wasm_bindgen]
pub fn wasm_parse_errors(args: JsValue, input: &str) -> Vec<ParseError> {
    use crate::error::Error::*;
    use pest::error::InputLocation::*;

    let args = wasm_args(args);
    let parser_config: SessionParserConfig = args.clone().into();
    let res = parser_config.parse_input(input);

    match res {
        Ok(_) => vec![],
        Err(Signal::Error(ParseUnexpected(r, (start, end)))) => {
            vec![ParseError { start, end, message: format!("Unexpected {r:?}") }]
        }
        Err(Signal::Error(ParseFailure(e))) => match e.location {
            Pos(p) => vec![ParseError { start: p, end: p, message: format!("{e:?}") }],
            Span((start, end)) => vec![ParseError { start, end, message: format!("{e:?}") }],
        },
        Err(e) => {
            log(&format!("{e:?}"));
            vec![]
        }
    }
}

/// returns a stream of strings. Each pair represents a style and text
#[wasm_bindgen]
pub fn wasm_highlight(args: JsValue, input: &str) -> Vec<JsValue> {
    let args = wasm_args(args);
    let parser_config: SessionParserConfig = args.clone().into();
    parser_config
        .parse_highlight(input)
        .unwrap_or_default()
        .into_iter()
        .flat_map(|(text, style)| vec![style.to_string(), text.to_string()])
        .map(JsValue::from)
        .collect()
}

pub fn wasm_eval_in(args: &Session, env: &Rc<Environment>, input: &str) -> Option<String> {
    let parser_config: SessionParserConfig = args.clone().into();
    match parser_config.parse_input(input) {
        Ok(expr) => {
            let mut stack = CallStack::from(args.clone()).with_global_env(env.clone());
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
