pub mod en {
    use r_derive::LocalizedParser;
    #[derive(Parser, Clone, Copy, LocalizedParser)]
    #[grammar = "grammar/localizations/en.pest"]
    #[grammar = "grammar/grammar.pest"]
    pub struct Parser;

    impl From<Rule> for Style {
        fn from(rule: Rule) -> Self {
            match rule {
                Rule::hl_sym => Self::Symbol,
                Rule::hl_callname => Self::Call,
                Rule::hl_value => Self::Value,
                Rule::hl_num => Self::Number,
                Rule::hl_str => Self::String,
                Rule::hl_comment => Self::Comment,
                Rule::hl_function => Self::Function,
                Rule::hl_signal => Self::Signal,
                Rule::hl_control => Self::ControlFlow,
                Rule::hl_open | Rule::hl_brackets => Self::Brackets,
                Rule::hl_ops => Self::Operators,
                Rule::hl_infix => Self::Infix,
                _ => Self::None,
            }
        }
    }
}

pub mod es {
    use r_derive::{LocalizedParser, Translate};
    #[derive(Parser, Clone, Copy, Translate, LocalizedParser)]
    #[grammar = "grammar/localizations/es.pest"]
    #[grammar = "grammar/grammar.pest"]
    pub struct Parser;
}

pub mod zh {
    use r_derive::{LocalizedParser, Translate};
    #[derive(Parser, Clone, Copy, Translate, LocalizedParser)]
    #[grammar = "grammar/localizations/zh.pest"]
    #[grammar = "grammar/grammar.pest"]
    pub struct Parser;
}

pub mod de {
    use r_derive::{LocalizedParser, Translate};
    #[derive(Parser, Clone, Copy, Translate, LocalizedParser)]
    #[grammar = "grammar/localizations/de.pest"]
    #[grammar = "grammar/grammar.pest"]
    pub struct Parser;
}

pub mod pirate {
    use r_derive::{LocalizedParser, Translate};
    #[derive(Parser, Clone, Copy, Translate, LocalizedParser)]
    #[grammar = "grammar/localizations/pirate.pest"]
    #[grammar = "grammar/grammar.pest"]
    pub struct Parser;
}

pub mod emoji {
    use r_derive::{LocalizedParser, Translate};
    #[derive(Parser, Clone, Copy, Translate, LocalizedParser)]
    #[grammar = "grammar/localizations/emoji.pest"]
    #[grammar = "grammar/grammar.pest"]
    pub struct Parser;
}
