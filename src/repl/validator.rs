use crate::parser::*;
use reedline::{ValidationResult, Validator};

impl Validator for Localization {
    fn validate(&self, line: &str) -> ValidationResult {
        let res = self.parse_input(line);
        match res {
            Ok(_) => ValidationResult::Complete,
            Err(_) => ValidationResult::Incomplete,
        }
    }
}
