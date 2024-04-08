use reedline::{FileBackedHistory, Reedline};
use std::io::Write;
use std::rc::Rc;

use super::prompt::Prompt;
use super::release::*;
use crate::context::Context;
use crate::lang::{CallStack, Cond, EvalResult, Signal};
use crate::object::Environment;
use crate::parser::{Localization, LocalizedParser};
use crate::session::Session;

pub fn repl(mut session: Session) -> Result<(), Signal> {
    writeln!(session.output, "{}", session_header(&session)).ok();
    let global_env = Rc::new(Environment {
        parent: Some(Environment::from_builtins()),
        ..Default::default()
    });

    let history = session
        .history
        .clone()
        .map_or(FileBackedHistory::new(1000), |file| {
            writeln!(session.output, "Restoring session history...").ok();
            FileBackedHistory::with_file(1000, file.into())
                .expect("Error configuring history with file")
        });

    let mut line_editor = Reedline::create()
        .with_validator(Box::new(session.locale))
        .with_highlighter(Box::new(session.locale))
        .with_history(Box::new(history));

    // initialize our repl prompt
    let prompt = Prompt;

    // REPL
    loop {
        let signal = line_editor.read_line(&prompt);
        match signal {
            Ok(reedline::Signal::Success(line)) => {
                // skip all-whitespace entries
                if line.chars().all(char::is_whitespace) {
                    continue;
                }

                // otherwise parse and evaluate entry
                let parse_res = session.locale.parse_input(&line);
                match parse_res {
                    Ok(expr) => {
                        let mut stack =
                            CallStack::from(session.clone()).with_global_env(global_env.clone());

                        match stack.eval_and_finalize(expr) {
                            Err(Signal::Condition(Cond::Terminate)) => break,
                            Err(Signal::Return(value, true)) => {
                                write!(session.output, "{value}").ok();
                            }
                            Err(Signal::Return(_value, false)) => (),
                            Err(e) => {
                                write!(session.output, "{e}").ok();
                                write!(session.output, "backtrace:\n{stack}").ok();
                            }
                            Ok(val) => {
                                writeln!(session.output, "{val}").ok();
                            }
                        }
                    }
                    Err(e) => eprint!("{e}"),
                }
            }
            Ok(reedline::Signal::CtrlD) => break,
            Ok(reedline::Signal::CtrlC) => continue,
            Err(err) => {
                println!("REPL Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}

pub fn eval(input: &str) -> EvalResult {
    let global_env = Rc::new(Environment {
        parent: Some(Environment::from_builtins()),
        ..Default::default()
    });

    let locale = Localization::En;
    let mut stack = CallStack::from(Session::default()).with_global_env(global_env);
    match locale.parse_input(input) {
        Ok(expr) => stack.eval_and_finalize(expr),
        Err(e) => Err(e),
    }
}
