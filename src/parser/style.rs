#[derive(Debug, Copy, Clone, Default, PartialEq, strum::EnumString, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum Style {
    Reserved,
    Keyword,
    ControlFlow,
    Call,
    Symbol,
    Value,
    Number,
    String,
    Brackets,
    Operators,
    Infix,
    Comment,
    #[default]
    None,
}

#[cfg(feature = "repl")]
impl From<Style> for nu_ansi_term::Style {
    fn from(val: Style) -> Self {
        use super::Style::*;
        use nu_ansi_term::{Color, Style};

        match val {
            Symbol => Style::new().fg(Color::White).bold(),
            Call => Style::new().fg(Color::Rgb(122, 162, 247)).italic(),
            Value => Style::new().fg(Color::Rgb(255, 158, 101)),
            Number => Style::new().fg(Color::Rgb(240, 158, 130)),
            String => Style::new().fg(Color::Rgb(158, 206, 106)),
            Comment => Style::new().fg(Color::Rgb(100, 100, 100)),
            Reserved | ControlFlow => Style::new().fg(Color::Rgb(187, 154, 246)).italic(),
            Brackets | Operators | Infix => Style::new().fg(Color::Rgb(170, 170, 190)),
            _ => Style::new().fg(Color::White),
        }
    }
}
