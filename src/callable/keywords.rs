use r_derive::*;

use crate::ast::*;
use crate::lang::*;
use super::core::*;

#[derive(Debug, Clone, PartialEq)]
#[builtin]
pub struct KeywordIf;

impl Format for KeywordIf {
    fn rfmt_call_with(&self, _state: FormatState, args: &ExprList) -> String {
        if let Some(else_expr) = args.values.get(2) {
            format!(
                "if ({}) {} else {}",
                args.values[0], args.values[1], else_expr
            )
        } else {
            format!("if ({}) {}", args.values[0], args.values[1])
        }
    }
}

impl Callable for KeywordIf {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let mut args = args.values.into_iter();
        let cond = stack.eval(args.next().unwrap())?;
        let cond: bool = cond.try_into()?;

        if cond {
            stack.eval(args.next().unwrap())
        } else {
            stack.eval(args.skip(1).next().unwrap_or(Expr::Null))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin]
pub struct KeywordFor;

impl Format for KeywordFor {
    fn rfmt_call_with(&self, _state: FormatState, args: &ExprList) -> String {
        let Some(sym) = &args.keys[0] else {
            unreachable!()
        };

        format!("for ({} in {}) {}", sym, args.values[0], args.values[1])
    }
}

impl Callable for KeywordFor {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let mut args = args.into_iter();

        let (Some(var), iter_expr) = args.next().unwrap() else {
            unreachable!()
        };

        let (_, body) = args.next().unwrap();
        let iter = stack.eval(iter_expr)?;

        let mut eval_result: EvalResult;
        let mut result = R::Null;
        let mut index = 1;

        while let Some(value) = iter.get(index) {
            index += 1;

            stack.last_frame().env.insert(var.clone(), value);
            eval_result = stack.eval(body.clone());

            use Cond::*;
            use RSignal::*;
            match eval_result {
                Err(Condition(Break)) => break,
                Err(Condition(Continue)) => continue,
                Err(Condition(Return(_))) => return eval_result,
                Err(Error(_)) => return eval_result,
                _ => (),
            }

            result = eval_result.expect("unhandled eval err");
        }

        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin]
pub struct KeywordWhile;

impl Format for KeywordWhile {
    fn rfmt_call_with(&self, _state: FormatState, args: &ExprList) -> String {
        format!("while ({}) {}", args.values[0], args.values[1])
    }
}

impl Callable for KeywordWhile {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        use Cond::*;
        use RSignal::*;

        let mut args = args.values.into_iter();

        let cond = args.next().unwrap();
        let body = args.next().unwrap();

        let mut eval_result: EvalResult;
        let mut result = R::Null;

        loop {
            // handle while condition
            let cond_result = stack.eval(cond.clone())?;
            if cond_result.try_into()? {
                eval_result = stack.eval(body.clone());
            } else {
                break;
            }

            // handle control flow signals during execution
            match eval_result {
                Err(Condition(Break)) => break,
                Err(Condition(Continue)) => continue,
                Err(Condition(Return(_))) => return eval_result,
                Err(Error(_)) => return eval_result,
                _ => (),
            }

            // update result
            result = eval_result.expect("unhandled eval err");
        }

        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin]
pub struct KeywordRepeat;

impl Format for KeywordRepeat {
    fn rfmt_call_with(&self, _state: FormatState, args: &ExprList) -> String {
        format!("repeat {}", args.values[1])
    }
}

impl Callable for KeywordRepeat {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let mut args = args.values.into_iter();
        let body = args.next().unwrap();

        let mut eval_result: EvalResult;
        let mut result = R::Null;

        loop {
            eval_result = stack.eval(body.clone());

            // handle control flow signals during execution
            match eval_result {
                Err(RSignal::Condition(Cond::Break)) => break,
                Err(RSignal::Condition(Cond::Continue)) => continue,
                Err(RSignal::Condition(Cond::Return(_))) => return eval_result,
                Err(RSignal::Error(_)) => return eval_result,
                _ => (),
            }

            // update result
            result = eval_result.expect("unhandled eval err");
        }

        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin]
pub struct KeywordBlock;

impl Format for KeywordBlock {
    fn rfmt_call_with(&self, _state: FormatState, args: &ExprList) -> String {
        format!(
            "{{\n{}\n}}",
            args.clone()
                .into_iter()
                .map(|(_, v)| format!("  {}", v))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

impl Callable for KeywordBlock {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let mut value = Ok(R::Null);
        for expr in args.values {
            let result = stack.eval(expr);
            match result {
                Ok(_) => value = result,
                _ => return result,
            }
        }
        value
    }
}
