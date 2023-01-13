use crate::parser::*;
use pest::Parser;
use reedline::{ValidationResult, Validator};

pub struct RValidator;

impl Validator for RValidator {
    fn validate(&self, line: &str) -> ValidationResult {
        let res = RParser::parse(Rule::repl, line);
        match res {
            Ok(_) => ValidationResult::Complete,
            Err(_) => ValidationResult::Incomplete,
        }
    }
}
