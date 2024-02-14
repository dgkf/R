use crate::parser::*;
use pest::Parser;
use reedline::{ValidationResult, Validator};

pub struct ExprValidator {}

impl ExprValidator {
    pub fn new() -> ExprValidator {
        ExprValidator {}
    }
}

impl Validator for ExprValidator {
    fn validate(&self, line: &str) -> ValidationResult {
        use es::*;
        let res = ExprParser::parse(Rule::repl, line);
        match res {
            Ok(_) => ValidationResult::Complete,
            Err(_) => ValidationResult::Incomplete,
        }
    }
}
