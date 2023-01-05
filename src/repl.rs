use std::borrow::Cow::{self, Borrowed, Owned};
use std::path::Path;

use rustyline::completion::FilenameCompleter;
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::validate::MatchingBracketValidator;
use rustyline::{Cmd, Editor, EventHandler, KeyCode, KeyEvent, Modifiers};
use rustyline_derive::{Completer, Helper, Hinter, Validator};

use crate::lang::Environment;
use crate::parser::parse;
use crate::utils::eval;

#[derive(Completer, Helper, Hinter, Validator)]
pub struct REPLHelper {
    #[rustyline(Validator)]
    brackets: MatchingBracketValidator,
    #[rustyline(Completer)]
    completer: FilenameCompleter,
    highlighter: MatchingBracketHighlighter,
}

impl Highlighter for REPLHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _default: bool,
    ) -> Cow<'b, str> {
        Borrowed(prompt)
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1b[1m".to_owned() + hint + "\x1b[m")
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        self.highlighter.highlight_char(line, pos)
    }
}

pub fn repl<P>(history: Option<&P>) -> rustyline::Result<()>
where
    P: AsRef<Path>,
{
    // configure repl editor
    let h = REPLHelper {
        brackets: MatchingBracketValidator::new(),
        completer: FilenameCompleter::new(),
        highlighter: MatchingBracketHighlighter::new(),
    };

    let mut rl = Editor::new()?;
    rl.set_helper(Some(h));
    rl.bind_sequence(
        KeyEvent(KeyCode::Char('s'), Modifiers::CTRL),
        EventHandler::Simple(Cmd::Newline),
    );

    // print session header
    println!("R version 0.0.1 -- \"Why Not?\"");

    if let Some(history_path) = history {
        if rl.load_history(history_path).is_ok() {
            println!("Restoring session history...");
        }
    }

    // start session environment
    let mut global_env = Environment::default();

    // REPL
    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                // skip all-whitespace entries
                if line.chars().all(char::is_whitespace) {
                    continue;
                }

                // otherwise parse and evaluate entry
                rl.add_history_entry(line.as_str());
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
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("REPL Error: {:?}", err);
                break;
            }
        }
    }

    if let Some(history_path) = history {
        rl.save_history(&history_path)?
    }

    Ok(())
}
