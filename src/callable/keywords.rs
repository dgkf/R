use r_derive::Primitive;

use crate::ast::*;
use crate::lang::*;
use super::core::*;

#[derive(Debug, Clone, Primitive, PartialEq)]
pub struct PrimIf;

impl Format for PrimIf {
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

impl Callable for PrimIf {
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

#[derive(Debug, Clone, Primitive, PartialEq)]
pub struct PrimFor;

impl Format for PrimFor {
    fn rfmt_call_with(&self, _state: FormatState, args: &ExprList) -> String {
        let Some(sym) = &args.keys[0] else {
            unreachable!()
        };

        format!("for ({} in {}) {}", sym, args.values[0], args.values[1])
    }
}

impl Callable for PrimFor {
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

#[derive(Debug, Clone, Primitive, PartialEq)]
pub struct PrimWhile;

impl Format for PrimWhile {
    fn rfmt_call_with(&self, _state: FormatState, args: &ExprList) -> String {
        format!("while ({}) {}", args.values[0], args.values[1])
    }
}

impl Callable for PrimWhile {
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

#[derive(Debug, Clone, Primitive, PartialEq)]
pub struct PrimRepeat;

impl Format for PrimRepeat {
    fn rfmt_call_with(&self, _state: FormatState, args: &ExprList) -> String {
        format!("repeat {}", args.values[1])
    }
}

impl Callable for PrimRepeat {
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

#[derive(Debug, Clone, Primitive, PartialEq)]
pub struct PrimBlock;

impl Format for PrimBlock {
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

impl Callable for PrimBlock {
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
