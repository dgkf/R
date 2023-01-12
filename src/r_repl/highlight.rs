use std::borrow::Cow::{self, Borrowed, Owned};

use colored::Colorize;
use rustyline::highlight::Highlighter;

// use crate::{ast::RExpr};
use crate::parser::RParser;
use crate::parser::Rule;
use pest::Parser;

#[derive(Debug, Clone)]
pub enum RHighlights {
    Keyword,
    Symbol,
    Number,
    String,
    Brackets,
    Operators,
    Reserved,
    ControlFlow,
    Value,
    None,
}

pub struct RHighlighter {}
impl RHighlighter {
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

impl Highlighter for RHighlighter {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        match RParser::parse(Rule::hl, line) {
            Ok(pairs) => {
                let mut out = String::new();
                for pair in pairs.into_iter() {
                    let next = match pair.as_rule() {
                        Rule::hl_sym => pair.as_str().white().bold(),
                        Rule::hl_callname => pair.as_str().truecolor(122, 162, 247).italic(),
                        Rule::hl_value => pair.as_str().truecolor(255, 158, 101),
                        Rule::hl_num => pair.as_str().truecolor(240, 158, 130),
                        Rule::hl_str => pair.as_str().truecolor(158, 206, 106),
                        Rule::hl_reserved | Rule::hl_control => {
                            pair.as_str().truecolor(187, 154, 246).italic()
                        }
                        Rule::hl_open | Rule::hl_brackets | Rule::hl_ops | Rule::hl_infix => {
                            pair.as_str().truecolor(170, 170, 190)
                        }
                        _ => pair.as_str().white(),
                    };
                    out.push_str(&next.to_string());
                }
                Owned(out)
            }
            Err(_) => Borrowed(line),
        }
    }

    fn highlight_char(&self, _line: &str, _pos: usize) -> bool {
        true
    }
}
