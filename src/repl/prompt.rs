use std::borrow::Cow;

use reedline::{
    Prompt, PromptEditMode, PromptHistorySearch, PromptHistorySearchStatus, PromptViMode, Color
};

#[derive(Clone)]
pub struct RPrompt;
impl RPrompt {
    pub fn default() -> Self {
        RPrompt {}
    }
}

impl Prompt for RPrompt {
    fn render_prompt_left(&self) -> Cow<str> {
        Cow::Owned("".to_string())
    }

    fn render_prompt_right(&self) -> Cow<str> {
        Cow::Owned("".to_string())
    }

    fn render_prompt_indicator(&self, edit_mode: PromptEditMode) -> Cow<str> {
        match edit_mode {
            PromptEditMode::Default | PromptEditMode::Emacs => "> ".into(),
            PromptEditMode::Vi(vi_mode) => match vi_mode {
                PromptViMode::Normal => "n]".into(),
                PromptViMode::Insert => "i]".into(),
            },
            PromptEditMode::Custom(str) => format!("({})", str).into(),
        }
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<str> {
        Cow::Borrowed(": ")
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: PromptHistorySearch,
    ) -> Cow<str> {
        let prefix = match history_search.status {
            PromptHistorySearchStatus::Passing => "",
            PromptHistorySearchStatus::Failing => "failing ",
        };
        // NOTE: magic strings, given there is logic on how these compose I am not sure if it
        // is worth extracting in to static constant
        Cow::Owned(format!(
            "({}reverse-search: {}) ",
            prefix, history_search.term
        ))
    }

    /// Get the default prompt color
    fn get_prompt_color(&self) -> Color {
        Color::White
    }

    /// Get the default indicator color
    fn get_indicator_color(&self) -> Color {
        Color::White
    }

    /// Get the default right prompt color
    fn get_prompt_right_color(&self) -> Color {
        Color::White
    }
}
