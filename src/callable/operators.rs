use r_derive::*;

use super::core::*;
use crate::context::Context;
use crate::error::RError;
use crate::lang::{CallStack, EvalResult};
use crate::object::types::*;
use crate::object::*;

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "<-", kind = Infix)]
pub struct InfixAssign;
impl Callable for InfixAssign {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = args.unnamed_binary_args();
        stack.assign_lazy(lhs, rhs)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "+", kind = Infix)]
pub struct InfixAdd;
impl Callable for InfixAdd {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs + rhs
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "-", kind = Infix)]
pub struct InfixSub;
impl Callable for InfixSub {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs - rhs
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "-", kind = Prefix)]
pub struct PrefixSub;
impl Callable for PrefixSub {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let what = stack.eval(args.unnamed_unary_arg())?;
        -what
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "..", kind = Prefix)]
pub struct PrefixPack;
impl Callable for PrefixPack {
    fn call(&self, _args: ExprList, _stack: &mut CallStack) -> EvalResult {
        RError::IncorrectContext("..".to_string()).into()
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "*", kind = Infix)]
pub struct InfixMul;
impl Callable for InfixMul {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs * rhs
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "/", kind = Infix)]
pub struct InfixDiv;
impl Callable for InfixDiv {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs / rhs
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "^")]
pub struct InfixPow;
impl Callable for InfixPow {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs.power(rhs)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "%", kind = Infix)]
pub struct InfixMod;
impl Callable for InfixMod {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs % rhs
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "||", kind = Infix)]
pub struct InfixOr;
impl Callable for InfixOr {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        let res = match (lhs, rhs) {
            (Obj::Vector(l), Obj::Vector(r)) => {
                let Ok(lhs) = l.try_into() else { todo!() };
                let Ok(rhs) = r.try_into() else { todo!() };
                Obj::Vector(Vector::from(vec![OptionNA::Some(lhs || rhs)]))
            }
            _ => Obj::Null,
        };

        Ok(res)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "&&", kind = Infix)]
pub struct InfixAnd;
impl Callable for InfixAnd {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        let res = match (lhs, rhs) {
            (Obj::Vector(l), Obj::Vector(r)) => {
                let Ok(lhs) = l.try_into() else { todo!() };
                let Ok(rhs) = r.try_into() else { todo!() };
                Obj::Vector(Vector::from(vec![OptionNA::Some(lhs && rhs)]))
            }
            _ => Obj::Null,
        };

        Ok(res)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "|", kind = Infix)]
pub struct InfixVectorOr;
impl Callable for InfixVectorOr {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs | rhs
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "&", kind = Infix)]
pub struct InfixVectorAnd;
impl Callable for InfixVectorAnd {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs & rhs
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = ">", kind = Infix)]
pub struct InfixGreater;
impl Callable for InfixGreater {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs.vec_gt(rhs)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = ">=", kind = Infix)]
pub struct InfixGreaterEqual;
impl Callable for InfixGreaterEqual {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs.vec_gte(rhs)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "<", kind = Infix)]
pub struct InfixLess;
impl Callable for InfixLess {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs.vec_lt(rhs)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "<=", kind = Infix)]
pub struct InfixLessEqual;
impl Callable for InfixLessEqual {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs.vec_lte(rhs)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "==", kind = Infix)]
pub struct InfixEqual;
impl Callable for InfixEqual {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs.vec_eq(rhs)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "!=", kind = Infix)]
pub struct InfixNotEqual;
impl Callable for InfixNotEqual {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs.vec_neq(rhs)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "|>", kind = Infix)]
pub struct InfixPipe;
impl Callable for InfixPipe {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        // TODO: reduce call stack nesting here
        let (lhs, rhs) = args.unnamed_binary_args();

        use Expr::*;
        match rhs {
            Call(what, mut args) => {
                args.insert(0, lhs);
                let new_expr = Call(what, args);
                stack.eval(new_expr)
            }
            s @ Symbol(..) | s @ String(..) => {
                let args = ExprList::from(vec![(None, lhs)]);
                let new_expr = Call(Box::new(s), args);
                stack.eval(new_expr)
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = ":", kind = Infix)]
pub struct InfixColon;
impl Callable for InfixColon {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let mut argstream = args.into_iter();
        let arg1 = argstream.next().map(|(_, v)| v).unwrap_or(Expr::Null);
        let arg2 = argstream.next().map(|(_, v)| v).unwrap_or(Expr::Null);

        fn colon_args(arg: &Expr) -> Option<(Expr, Expr)> {
            if let Expr::Call(what, largs) = arg.clone() {
                if let Expr::Primitive(p) = *what {
                    if p == (Box::new(InfixColon) as Box<dyn Builtin>) {
                        return Some(largs.clone().unnamed_binary_args());
                    }
                }
            }

            None
        }

        // handle special case of chained colon ops: `x:y:z`
        if let Some((llhs, lrhs)) = colon_args(&arg1) {
            // since we're rearranging calls here, we might need to modify the call stack
            let args = ExprList::from(vec![(None, llhs), (None, lrhs), (None, arg2)]);
            return InfixColon.call(args, stack);

        // tertiary case
        } else if let Some((_, arg3)) = argstream.next() {
            // currently always returns numeric vector
            let start: f64 = stack.eval(arg1)?.try_into()?;
            let by: f64 = stack.eval(arg2)?.try_into()?;
            let end: f64 = stack.eval(arg3)?.try_into()?;

            if by == 0.0 {
                return RError::Other("Cannot increment by 0".to_string()).into();
            }

            if (end - start) / by < 0.0 {
                return Ok(Obj::Vector(Vector::from(Vec::<Numeric>::new())));
            }

            let mut v = start;
            return Ok(Obj::Vector(Vector::from(
                vec![start]
                    .into_iter()
                    .chain(std::iter::repeat_with(|| {
                        v += by;
                        v
                    }))
                    .take_while(|x| if &start <= &end { x <= &end } else { x >= &end })
                    .collect::<Vec<f64>>(),
            )));

        // binary case
        } else {
            let start: i32 = stack.eval(arg1)?.as_integer()?.try_into()?;
            let end: i32 = stack.eval(arg2)?.as_integer()?.try_into()?;
            return Ok(Obj::Vector(Vector::from(if start <= end {
                (start..=end)
                    .map(|i| i as f64)
                    .into_iter()
                    .collect::<Vec<f64>>()
            } else {
                (end..=start)
                    .map(|i| i as f64)
                    .into_iter()
                    .rev()
                    .collect::<Vec<f64>>()
            })));
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "$", kind = Infix)]
pub struct InfixDollar;
impl Callable for InfixDollar {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let mut argstream = args.into_iter();

        let Some((_, what)) = argstream.next() else {
            unreachable!();
        };

        let Some((_, index)) = argstream.next() else {
            unreachable!();
        };

        let mut what = stack.eval(what)?;

        match index {
            Expr::String(s) | Expr::Symbol(s) => what.try_get_named(s.as_str()),
            _ => Ok(Obj::Null),
        }
    }

    fn call_assign(&self, value: Expr, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let mut argstream = args.into_iter();

        let Some((_, what)) = argstream.next() else {
            unreachable!();
        };

        let Some((_, name)) = argstream.next() else {
            unreachable!();
        };

        let value = stack.eval(value)?;
        let mut what = stack.eval(what)?;

        match name {
            Expr::String(s) | Expr::Symbol(s) => {
                what.set_named(s.as_str(), value)?;
                Ok(what)
            }
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "..", kind = Postfix)]
pub struct PostfixPack;
impl Callable for PostfixPack {
    fn call(&self, _args: ExprList, _stack: &mut CallStack) -> EvalResult {
        RError::IncorrectContext("..".to_string()).into()
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "[[", kind = PostfixCall("[[", "]]"))]
pub struct PostfixIndex;
impl Callable for PostfixIndex {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (what, index) = stack.eval_binary(args.unnamed_binary_args())?;
        what.try_get_inner(index)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "[", kind = PostfixCall("[", "]"))]
pub struct PostfixVecIndex;
impl Callable for PostfixVecIndex {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (what, index) = stack.eval_binary(args.unnamed_binary_args())?;
        what.try_get(index)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin]
pub struct PrimVec;

impl Format for PrimVec {
    fn rfmt_call_with(&self, _state: FormatState, args: &ExprList) -> String {
        format!("[{}]", args)
    }
}

impl Callable for PrimVec {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        // for now just use c()
        super::primitive::PrimitiveC.call(args, stack)
    }
}
