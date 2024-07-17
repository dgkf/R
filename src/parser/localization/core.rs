use crate::{lang::Signal, parser::*, session::SessionParserConfig};

pub type HighlightResult = Result<Vec<(String, Style)>, Signal>;
pub trait LocalizedParser: std::marker::Sync {
    fn parse_input_with(&self, input: &str, config: &SessionParserConfig) -> ParseResult;
    fn parse_input(&self, input: &str) -> ParseResult {
        self.parse_input_with(input, &SessionParserConfig::default())
    }
    fn parse_highlight_with(&self, input: &str, config: &SessionParserConfig) -> HighlightResult;
    fn parse_highlight(&self, input: &str) -> HighlightResult {
        self.parse_highlight_with(input, &SessionParserConfig::default())
    }
}

#[cfg(target_family = "wasm")]
use serde::{Deserialize, Serialize};

#[cfg_attr(
    target_family = "wasm",
    wasm_bindgen::prelude::wasm_bindgen,
    derive(Serialize, Deserialize),
    serde(rename_all(serialize = "kebab-case", deserialize = "kebab-case"))
)]
#[derive(Debug, Copy, Clone, Default, PartialEq, clap::ValueEnum, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum Localization {
    // ISO-639 codes
    #[default]
    En, // english
    Es, // spanish
    Zh, // chinese
    De, // german
    #[value(skip)]
    Pirate,
    #[value(skip)]
    Emoji,
}

impl LocalizedParser for Localization {
    fn parse_input_with(&self, input: &str, config: &SessionParserConfig) -> ParseResult {
        use Localization::*;
        match self {
            En => LocalizedParser::parse_input_with(&en::Parser, input, config),
            Es => LocalizedParser::parse_input_with(&es::Parser, input, config),
            De => LocalizedParser::parse_input_with(&de::Parser, input, config),
            Zh => LocalizedParser::parse_input_with(&zh::Parser, input, config),
            Pirate => LocalizedParser::parse_input_with(&pirate::Parser, input, config),
            Emoji => LocalizedParser::parse_input_with(&emoji::Parser, input, config),
        }
    }

    fn parse_highlight_with(&self, input: &str, config: &SessionParserConfig) -> HighlightResult {
        use Localization::*;
        match self {
            En => LocalizedParser::parse_highlight_with(&en::Parser, input, config),
            Es => LocalizedParser::parse_highlight_with(&es::Parser, input, config),
            De => LocalizedParser::parse_highlight_with(&de::Parser, input, config),
            Zh => LocalizedParser::parse_highlight_with(&zh::Parser, input, config),
            Pirate => LocalizedParser::parse_highlight_with(&pirate::Parser, input, config),
            Emoji => LocalizedParser::parse_highlight_with(&emoji::Parser, input, config),
        }
    }
}

impl LocalizedParser for SessionParserConfig {
    fn parse_input_with(&self, _input: &str, _config: &SessionParserConfig) -> ParseResult {
        unimplemented!()
    }

    fn parse_input(&self, input: &str) -> ParseResult {
        use Localization::*;
        match self.locale {
            En => LocalizedParser::parse_input_with(&en::Parser, input, self),
            Es => LocalizedParser::parse_input_with(&es::Parser, input, self),
            De => LocalizedParser::parse_input_with(&de::Parser, input, self),
            Zh => LocalizedParser::parse_input_with(&zh::Parser, input, self),
            Pirate => LocalizedParser::parse_input_with(&pirate::Parser, input, self),
            Emoji => LocalizedParser::parse_input_with(&emoji::Parser, input, self),
        }
    }

    fn parse_highlight_with(&self, _input: &str, _config: &SessionParserConfig) -> HighlightResult {
        unimplemented!()
    }

    fn parse_highlight(&self, input: &str) -> HighlightResult {
        use Localization::*;
        match self.locale {
            En => LocalizedParser::parse_highlight_with(&en::Parser, input, self),
            Es => LocalizedParser::parse_highlight_with(&es::Parser, input, self),
            De => LocalizedParser::parse_highlight_with(&de::Parser, input, self),
            Zh => LocalizedParser::parse_highlight_with(&zh::Parser, input, self),
            Pirate => LocalizedParser::parse_highlight_with(&pirate::Parser, input, self),
            Emoji => LocalizedParser::parse_highlight_with(&emoji::Parser, input, self),
        }
    }
}
