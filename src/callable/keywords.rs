use r_derive::*;

use super::core::*;
use crate::context::Context;
use crate::internal_err;
use crate::lang::Signal::*;
use crate::lang::*;
use crate::object::{ExprList, Obj};

#[derive(Debug, Clone, PartialEq)]
#[builtin]
pub struct KeywordReturn;

impl Format for KeywordReturn {
    fn rfmt_call_with(&self, _state: FormatState, args: &ExprList) -> String {
        format!("return {}", args.values[0])
    }

    fn rfmt_with(&self, _state: FormatState) -> String {
        "if".to_string()
    }
}

impl CallableFormals for KeywordReturn {}

impl Callable for KeywordReturn {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let mut args = args.values.into_iter();
        let value = stack.eval(args.next().unwrap())?;
        Ok(value)
    }
}

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

    fn rfmt_with(&self, _state: FormatState) -> String {
        "if".to_string()
    }
}

impl CallableFormals for KeywordIf {}

impl Callable for KeywordIf {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let mut args = args.values.into_iter();
        let cond = stack.eval(args.next().unwrap())?;
        let cond: bool = cond.try_into()?;

        if cond {
            let ifblock = args.next().ok_or::<Signal>(internal_err!())?;
            Tail(ifblock, true).into()
        } else {
            let elseblock = args.nth(1).ok_or::<Signal>(internal_err!())?;
            Tail(elseblock, true).into()
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin]
pub struct KeywordFor;

impl CallableFormals for KeywordFor {}

impl Format for KeywordFor {
    fn rfmt_call_with(&self, _state: FormatState, args: &ExprList) -> String {
        let Some(sym) = &args.keys[0] else {
            unreachable!()
        };

        format!("for ({} in {}) {}", sym, args.values[0], args.values[1])
    }

    fn rfmt_with(&self, _state: FormatState) -> String {
        "for".to_string()
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
        let mut result = Obj::Null;
        let mut index = 0;

        while let Some(value) = iter.get(index) {
            index += 1;

            stack.last_frame().env().insert(var.clone(), value);
            eval_result = stack.eval_and_finalize(body.clone());

            use Cond::*;
            use Signal::*;
            match eval_result {
                Err(Condition(Break)) => break,
                Err(Condition(Continue)) => continue,
                Err(Return(..)) => return eval_result,
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

impl CallableFormals for KeywordWhile {}

impl Format for KeywordWhile {
    fn rfmt_call_with(&self, _state: FormatState, args: &ExprList) -> String {
        format!("while ({}) {}", args.values[0], args.values[1])
    }
}

impl Callable for KeywordWhile {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        use Cond::*;
        use Signal::*;

        let mut args = args.values.into_iter();

        let cond = args.next().unwrap();
        let body = args.next().unwrap();

        let mut eval_result: EvalResult;
        let mut result = Obj::Null;

        loop {
            // handle while condition
            let cond_result = stack.eval_and_finalize(cond.clone())?;
            if cond_result.try_into()? {
                eval_result = stack.eval_and_finalize(body.clone());
            } else {
                break;
            }

            // handle control flow signals during execution
            match eval_result {
                Err(Condition(Break)) => break,
                Err(Condition(Continue)) => continue,
                Err(Return(..)) => return eval_result,
                Err(Error(_)) => return eval_result,
                _ => (),
            }

            // update result
            result = eval_result?;
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

impl CallableFormals for KeywordRepeat {}

impl Callable for KeywordRepeat {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let mut args = args.values.into_iter();
        let body = args.next().unwrap();

        let mut eval_result: EvalResult;
        let mut result = Obj::Null;

        loop {
            eval_result = stack.eval_and_finalize(body.clone());

            // handle control flow signals during execution
            match eval_result {
                Err(Signal::Condition(Cond::Break)) => break,
                Err(Signal::Condition(Cond::Continue)) => continue,
                Err(Signal::Return(..)) => return eval_result,
                Err(Signal::Error(_)) => return eval_result,
                _ => (),
            }

            // update result
            result = eval_result?;
        }

        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin]
pub struct KeywordParen;

impl CallableFormals for KeywordParen {}

impl Format for KeywordParen {
    fn rfmt_call_with(&self, _state: FormatState, args: &ExprList) -> String {
        format!("({})", args.values.first().unwrap())
    }

    fn rfmt_with(&self, _: FormatState) -> String {
        "(".to_string()
    }
}

impl Callable for KeywordParen {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let expr = args.values.into_iter().next().unwrap();
        stack.eval_and_finalize(expr)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin]
pub struct KeywordBlock;

impl CallableFormals for KeywordBlock {}

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

    fn rfmt_with(&self, _: FormatState) -> String {
        "{".to_string()
    }
}

impl Callable for KeywordBlock {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let mut value = Obj::Null;
        let n = args.values.len().saturating_sub(1);

        for (i, expr) in args.values.into_iter().enumerate() {
            value = match i {
                i if i == n => return Tail(expr, true).into(),
                _ => stack.eval_and_finalize(expr)?,
            };
        }

        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin]
pub struct KeywordVec;

impl Format for KeywordVec {
    fn rfmt_call_with(&self, _state: FormatState, args: &ExprList) -> String {
        format!("[{}]", args)
    }
}

impl CallableFormals for KeywordVec {}

impl Callable for KeywordVec {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        // for now just use c()
        super::primitive::PrimitiveC.call(args, stack)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin]
pub struct KeywordList;

impl Format for KeywordList {
    fn rfmt_call_with(&self, _state: FormatState, args: &ExprList) -> String {
        let trailing_comma = if args.len() > 1 { "" } else { "," };
        format!("({}{})", args, trailing_comma)
    }
}

impl CallableFormals for KeywordList {}

impl Callable for KeywordList {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        super::primitive::PrimitiveList.call(args, stack)
    }
}

#[cfg(test)]
mod test {
    use crate::r;

    #[test]
    fn repeat_with_break() {
        assert_eq!(
            r! {{"
               sum <- 0
               repeat {
                   if (sum >= 3) break
                   sum <- sum + 1
               }
            "}},
            r! { 3 }
        );
    }

    #[test]
    fn while_simple() {
        assert_eq!(
            r! {{"
               sum <- 0
               while (sum < 3) {
                   sum <- sum + 1
               }
            "}},
            r! { 3 }
        );
    }

    #[test]
    fn for_simple() {
        assert_eq!(
            r! {{"
               sum <- 0
               for (i in 1:3) {
                   sum <- sum + i
               }
            "}},
            r! { 6 }
        );
    }
}
