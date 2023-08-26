use nu_ansi_term::Style;
use nu_ansi_term::Color;
use reedline::Highlighter;
use reedline::StyledText;

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
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> StyledText {
        let mut styled_text = StyledText::new();
        match RParser::parse(Rule::hl, line) {
            Ok(pairs) => {
                for pair in pairs.into_iter() {
                    let style = match pair.as_rule() {
                        Rule::hl_sym => Style::new().fg(Color::White).bold(),
                        Rule::hl_callname => Style::new().fg(Color::Rgb(122, 162, 247)).italic(),
                        Rule::hl_value => Style::new().fg(Color::Rgb(255, 158, 101)),
                        Rule::hl_num => Style::new().fg(Color::Rgb(240, 158, 130)),
                        Rule::hl_str => Style::new().fg(Color::Rgb(158, 206, 106)),
                        Rule::hl_reserved | Rule::hl_control => {
                            Style::new().fg(Color::Rgb(187, 154, 246)).italic()
                        }
                        Rule::hl_open | Rule::hl_brackets | Rule::hl_ops | Rule::hl_infix => {
                            Style::new().fg(Color::Rgb(170, 170, 190))
                        }
                        _ => Style::new().fg(Color::White),
                    };
                    styled_text.push((style, pair.as_str().to_string()));
                }
                styled_text
            }
            Err(_) => {
                styled_text.push((Style::new(), line.to_string()));
                styled_text
            }
        }
    }
}
