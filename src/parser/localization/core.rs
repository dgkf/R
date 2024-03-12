use crate::{lang::Signal, parser::*, session::Session};

pub type HighlightResult = Result<Vec<(String, Style)>, Signal>;
pub trait LocalizedParser: std::marker::Sync {
    fn parse_input_with(&self, input: &str, session: &Session) -> ParseResult;
    fn parse_input(&self, input: &str) -> ParseResult {
        self.parse_input_with(input, &Session::default())
    }
    fn parse_highlight_with(&self, input: &str, session: &Session) -> HighlightResult;
    fn parse_highlight(&self, input: &str) -> HighlightResult {
        self.parse_highlight_with(input, &Session::default())
    }
}

#[cfg(feature = "wasm")]
use serde::{Deserialize, Serialize};

#[cfg_attr(
    feature = "wasm",
    wasm_bindgen::prelude::wasm_bindgen,
    derive(Serialize, Deserialize),
    serde(rename_all(serialize = "kebab-case", deserialize = "kebab-case"))
)]
#[derive(Debug, Copy, Clone, Default, PartialEq, clap::ValueEnum, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum Localization {
    #[default]
    En,
    Es,
    Cn,
    #[value(skip)]
    Pirate,
    #[value(skip)]
    Emoji,
}

impl LocalizedParser for Localization {
    fn parse_input_with(&self, input: &str, session: &Session) -> ParseResult {
        use Localization::*;
        match self {
            En => LocalizedParser::parse_input_with(&en::Parser, input, session),
            Es => LocalizedParser::parse_input_with(&es::Parser, input, session),
            Cn => LocalizedParser::parse_input_with(&cn::Parser, input, session),
            Pirate => LocalizedParser::parse_input_with(&pirate::Parser, input, session),
            Emoji => LocalizedParser::parse_input_with(&emoji::Parser, input, session),
        }
    }

    fn parse_highlight_with(&self, input: &str, session: &Session) -> HighlightResult {
        use Localization::*;
        match self {
            En => LocalizedParser::parse_highlight_with(&en::Parser, input, session),
            Es => LocalizedParser::parse_highlight_with(&es::Parser, input, session),
            Cn => LocalizedParser::parse_highlight_with(&cn::Parser, input, session),
            Pirate => LocalizedParser::parse_highlight_with(&pirate::Parser, input, session),
            Emoji => LocalizedParser::parse_highlight_with(&emoji::Parser, input, session),
        }
    }
}
