use reedline::{FileBackedHistory, Reedline, Signal};
use std::path::Path;
use std::rc::Rc;

use super::highlight::ExprHighlighter;
use super::prompt::RPrompt;
use super::validator::ExprValidator;
use super::release::*;
use crate::lang::{CallStack, Cond, Context, RSignal, EvalResult};
use crate::object::Environment;
use crate::parser::*;

pub fn repl<P>(history: Option<&P>) -> Result<(), ()>
where
    P: AsRef<Path>,
{
    println!("{}", session_header());
    let global_env = Rc::new(Environment {
        parent: Some(Environment::from_builtins()),
        ..Default::default()        
    });

    // initialize parser
    let line_editor = Reedline::create()
        .with_validator(Box::new(ExprValidator::new()))
        .with_highlighter(Box::new(ExprHighlighter::new()));

    let mut line_editor = if let Some(_history_path) = history {
        println!("Restoring session history...");

        let history = Box::new(
            FileBackedHistory::with_file(1000, "/tmp/history.txt".into())
                .expect("Error configuring history with file"),
        );

        line_editor.with_history(history)
    } else {
        line_editor
    };

    // initialize our repl prompt
    let prompt = RPrompt::default();

    // REPL
    loop {
        let signal = line_editor.read_line(&prompt);
        match signal {
            Ok(Signal::Success(line)) => {
                // skip all-whitespace entries
                if line.chars().all(char::is_whitespace) {
                    continue;
                }

                // otherwise parse and evaluate entry
                let parse_res = parse_with(es::ExprParser, es::Rule::repl, &line);
                match parse_res {
                    Ok(expr) => {
                        let mut stack = CallStack::from(global_env.clone());
                        match stack.eval(expr) {
                            Err(RSignal::Condition(Cond::Terminate)) => break,
                            Ok(val) => println!("{}", val),
                            Err(e) => println!("{}", e),
                        }
                    }
                    Err(e) => {
                        eprintln!("{}", e)
                    }
                }
            }
            Ok(Signal::CtrlD) => break,
            Ok(Signal::CtrlC) => continue,
            Err(err) => {
                println!("REPL Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}

pub fn eval(input: &str) -> EvalResult {
    // initialize global env
    let global_env = Rc::new(Environment {
        parent: Some(Environment::from_builtins()),
        ..Default::default()        
    });

    let mut stack = CallStack::from(global_env.clone());
    match parse_with(ExprParser, Rule::repl, input) {
        Ok(expr) => stack.eval(expr),
        Err(e) => Err(e)
    }
}
