use reedline::{FileBackedHistory, Reedline};
use std::path::Path;
use std::rc::Rc;

use super::prompt::RPrompt;
use super::release::*;
use crate::context::Context;
use crate::lang::{CallStack, Cond, EvalResult, Signal};
use crate::object::Environment;
use crate::parser::{Localization, LocalizedParser};

pub fn repl<P>(locale: Localization, history: Option<&P>, warranty: bool) -> Result<(), Signal>
where
    P: AsRef<Path>,
{
    println!("{}", session_header(warranty, &locale));
    let global_env = Rc::new(Environment {
        parent: Some(Environment::from_builtins()),
        ..Default::default()
    });

    let history = if let Some(_history_path) = history {
        println!("Restoring session history...");
        FileBackedHistory::with_file(1000, "/tmp/history.txt".into())
            .expect("Error configuring history with file")
    } else {
        FileBackedHistory::new(1000)
    };

    let mut line_editor = Reedline::create()
        .with_validator(Box::new(locale.clone()))
        .with_highlighter(Box::new(locale.clone()))
        .with_history(Box::new(history));

    // initialize our repl prompt
    let prompt = RPrompt;

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
                let parse_res = locale.parse_input(&line);
                match parse_res {
                    Ok(expr) => {
                        let mut stack = CallStack::from(global_env.clone());
                        match stack.eval_and_finalize(expr) {
                            Err(Signal::Condition(Cond::Terminate)) => break,
                            Err(Signal::Return(value, true)) => {
                                print!("{value}")
                            }
                            Err(Signal::Return(_value, false)) => (),
                            Err(e) => {
                                print!("{e}");
                                print!("traceback:\n{stack}");
                            }
                            Ok(val) => println!("{val}"),
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
    let mut stack = CallStack::from(global_env.clone());
    match locale.parse_input(input) {
        Ok(expr) => stack.eval_and_finalize(expr),
        Err(e) => Err(e),
    }
}
