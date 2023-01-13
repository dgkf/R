use reedline::{FileBackedHistory, Reedline, Signal};
use std::path::Path;

use super::highlight::RHighlighter;
use super::prompt::RPrompt;
use super::validator::RValidator;
use crate::lang::Environment;
use crate::parser::parse;
use crate::utils::eval;

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
            FileBackedHistory::with_file(5, "/tmp/history.txt".into())
                .expect("Error configuring history with file"),
        );

        line_editor.with_history(history)
    } else {
        line_editor
    };

    // initialize our repl prompt
    let prompt = RPrompt::default();

    // start session environment
    let mut global_env = Environment::default();

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
                        let res = eval(expr, &mut global_env);
                        match res {
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
