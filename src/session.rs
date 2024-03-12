use crate::cli::{Cli, Experiment};
use crate::parser::{Localization, LocalizedParser};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Session {
    pub locale: Localization,
    pub warranty: bool,
    pub experiments: Vec<Experiment>,
    pub history: String,
}

impl From<Cli> for Session {
    fn from(value: Cli) -> Self {
        Session {
            locale: value.locale,
            warranty: value.warranty,
            experiments: value.experiments,
            history: std::env::temp_dir()
                .join("history.txt")
                .into_os_string()
                .into_string()
                .unwrap_or_default(),
        }
    }
}

impl LocalizedParser for Session {
    fn parse_input(&self, input: &str) -> crate::parser::ParseResult {
        self.locale.parse_input_with(input, self)
    }

    fn parse_highlight(&self, input: &str) -> crate::parser::HighlightResult {
        self.locale.parse_highlight_with(input, self)
    }

    fn parse_input_with(&self, input: &str, session: &Session) -> crate::parser::ParseResult {
        self.locale.parse_input_with(input, session)
    }

    fn parse_highlight_with(
        &self,
        input: &str,
        session: &Session,
    ) -> crate::parser::HighlightResult {
        self.locale.parse_highlight_with(input, session)
    }
}
