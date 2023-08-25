use reedline::{FileBackedHistory, Reedline, Signal};
use std::path::Path;
use std::rc::Rc;

use super::highlight::RHighlighter;
use super::prompt::RPrompt;
use super::validator::RValidator;
use crate::ast::Expr;
use crate::lang::{CallStack, Cond, Context, Environment, Frame, RSignal};
use crate::parser::parse;

pub fn repl<P>(history: Option<&P>) -> Result<(), ()>
where
    P: AsRef<Path>,
{
    // print session header
    println!("R version 0.0.1 -- \"Why Not?\"");

    let line_editor = Reedline::create()
        .with_validator(Box::new(RValidator))
        .with_highlighter(Box::new(RHighlighter::new()));

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

    // start session environment
    let global_env = Rc::new(Environment::default());

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
                let parse_res = parse(&line);
                match parse_res {
                    Ok(expr) => {
                        // start new callstack for input, starting with a missing at the global env 
                        let mut stack = CallStack::from(Frame { call: Expr::Missing, env: global_env.clone() });
                        let res = stack.eval(expr);
                        match res {
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
