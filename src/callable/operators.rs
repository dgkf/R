use r_derive::*;

use super::core::*;
use crate::context::Context;
use crate::error::Error;
use crate::lang::{CallStack, EvalResult};
use crate::object::types::*;
use crate::object::*;

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "<-", kind = Infix)]
pub struct InfixAssign;
impl CallableFormals for InfixAssign {}
impl Callable for InfixAssign {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = args.unnamed_binary_args();
        stack.assign_lazy(lhs, rhs)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "+", kind = Infix)]
pub struct InfixAdd;
impl CallableFormals for InfixAdd {}
impl Callable for InfixAdd {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs + rhs
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "-", kind = Infix)]
pub struct InfixSub;
impl CallableFormals for InfixSub {}
impl Callable for InfixSub {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs - rhs
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "-", kind = Prefix)]
pub struct PrefixSub;
impl CallableFormals for PrefixSub {}
impl Callable for PrefixSub {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let what = stack.eval(args.unnamed_unary_arg())?;
        -what
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "!", kind = Prefix)]
pub struct PrefixNot;
impl CallableFormals for PrefixNot {}
impl Callable for PrefixNot {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let what = stack.eval(args.unnamed_unary_arg())?;
        !what
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "..", kind = Prefix)]
pub struct PrefixPack;
impl CallableFormals for PrefixPack {}
impl Callable for PrefixPack {
    fn call(&self, _args: ExprList, _stack: &mut CallStack) -> EvalResult {
        Error::IncorrectContext("..".to_string()).into()
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "*", kind = Infix)]
pub struct InfixMul;
impl CallableFormals for InfixMul {}
impl Callable for InfixMul {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs * rhs
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "/", kind = Infix)]
pub struct InfixDiv;
impl CallableFormals for InfixDiv {}
impl Callable for InfixDiv {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs / rhs
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "^")]
pub struct InfixPow;
impl CallableFormals for InfixPow {}
impl Callable for InfixPow {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs.power(rhs)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "%", kind = Infix)]
pub struct InfixMod;
impl CallableFormals for InfixMod {}
impl Callable for InfixMod {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs % rhs
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "||", kind = Infix)]
pub struct InfixOr;
impl CallableFormals for InfixOr {}
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
impl CallableFormals for InfixAnd {}
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
impl CallableFormals for InfixVectorOr {}
impl Callable for InfixVectorOr {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs | rhs
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "&", kind = Infix)]
pub struct InfixVectorAnd;
impl CallableFormals for InfixVectorAnd {}
impl Callable for InfixVectorAnd {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs & rhs
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = ">", kind = Infix)]
pub struct InfixGreater;
impl CallableFormals for InfixGreater {}
impl Callable for InfixGreater {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs.vec_gt(rhs)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = ">=", kind = Infix)]
pub struct InfixGreaterEqual;
impl CallableFormals for InfixGreaterEqual {}
impl Callable for InfixGreaterEqual {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs.vec_gte(rhs)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "<", kind = Infix)]
pub struct InfixLess;
impl CallableFormals for InfixLess {}
impl Callable for InfixLess {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs.vec_lt(rhs)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "<=", kind = Infix)]
pub struct InfixLessEqual;
impl CallableFormals for InfixLessEqual {}
impl Callable for InfixLessEqual {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs.vec_lte(rhs)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "==", kind = Infix)]
pub struct InfixEqual;
impl CallableFormals for InfixEqual {}
impl Callable for InfixEqual {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs.vec_eq(rhs)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "!=", kind = Infix)]
pub struct InfixNotEqual;
impl CallableFormals for InfixNotEqual {}
impl Callable for InfixNotEqual {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs.vec_neq(rhs)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "|>", kind = Infix)]
pub struct InfixPipe;
impl CallableFormals for InfixPipe {}
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
impl CallableFormals for InfixColon {}
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
            InfixColon.call(args, stack)

        // tertiary case
        } else if let Some((_, arg3)) = argstream.next() {
            // currently always returns double vector
            let start: f64 = stack.eval(arg1)?.try_into()?;
            let by: f64 = stack.eval(arg2)?.try_into()?;
            let end: f64 = stack.eval(arg3)?.try_into()?;

            if by == 0.0 {
                return Error::Other("Cannot increment by 0".to_string()).into();
            }

            let range = end - start;

            if range / by < 0.0 {
                return Ok(Obj::Vector(Vector::from(Vec::<Double>::new())));
            }

            let mut v = start;
            return Ok(Obj::Vector(Vector::from(
                vec![start]
                    .into_iter()
                    .chain(std::iter::repeat_with(|| {
                        v += by;
                        v
                    }))
                    .take_while(|x| if start <= end { x <= &end } else { x >= &end })
                    .collect::<Vec<f64>>(),
            )));

        // binary case
        } else {
            let start: i32 = stack.eval(arg1)?.as_integer()?.try_into()?;
            let end: i32 = stack.eval(arg2)?.as_integer()?.try_into()?;
            if start > end {
                return Error::InvalidRange.into();
            }
            return Ok(Obj::Vector(Vector::from(if start <= end {
                (start..=end).map(|i| i as f64).collect::<Vec<f64>>()
            } else {
                (end..=start).map(|i| i as f64).rev().collect::<Vec<f64>>()
            })));
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "$", kind = Infix)]
pub struct InfixDollar;
impl CallableFormals for InfixDollar {}
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
            Expr::String(s) | Expr::Symbol(s) => what.try_get_named(&s),
            _ => Ok(Obj::Null),
        }
    }

    fn call_mut(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let mut argstream = args.into_iter();

        let Some((_, what)) = argstream.next() else {
            unreachable!();
        };

        let Some((_, index)) = argstream.next() else {
            unreachable!();
        };

        let mut what = stack.eval_mut(what)?;

        match index {
            Expr::String(s) | Expr::Symbol(s) => what.try_get_named_mut(s.as_str()),
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
        let mut what = stack.eval_mut(what)?;

        match name {
            Expr::String(s) | Expr::Symbol(s) => what.try_set_named(&s, value),
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "..", kind = Postfix)]
pub struct PostfixPack;
impl CallableFormals for PostfixPack {}
impl Callable for PostfixPack {
    fn call(&self, _args: ExprList, _stack: &mut CallStack) -> EvalResult {
        Error::IncorrectContext("..".to_string()).into()
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "[[", kind = PostfixCall("[[", "]]"))]
pub struct PostfixIndex;
impl CallableFormals for PostfixIndex {}
impl Callable for PostfixIndex {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (what, index) = stack.eval_binary(args.unnamed_binary_args())?;
        what.try_get_inner(index)
    }

    fn call_mut(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let x = args.unnamed_binary_args();
        let what = stack.eval_mut(x.0)?;
        let index = stack.eval(x.1)?;
        what.try_get_inner_mut(index)
    }

    fn call_assign(&self, value: Expr, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let mut argstream = args.into_iter();

        let Some((_, what)) = argstream.next() else {
            unreachable!();
        };

        let Some((_, index)) = argstream.next() else {
            unreachable!();
        };

        let value = stack.eval(value)?;
        let what = stack.eval_mut(what)?;
        let index = stack.eval(index)?;

        let subset = index.try_into()?;

        Ok(match what {
            Obj::List(mut v) => v.set_subset(subset, value)?,
            Obj::Vector(mut v) => v.set_subset(subset, value).map(Obj::Vector)?,
            _ => unimplemented!(),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "[", kind = PostfixCall("[", "]"))]
pub struct PostfixVecIndex;
impl CallableFormals for PostfixVecIndex {}
impl Callable for PostfixVecIndex {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (what, index) = stack.eval_binary(args.unnamed_binary_args())?;
        what.try_get(index)
    }

    fn call_mut(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let x = args.unnamed_binary_args();
        let what = stack.eval_mut(x.0)?;
        let index = stack.eval(x.1)?;
        what.try_get(index)
    }
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::lang::{EvalResult, Signal};
    use crate::{r, r_expect};
    #[test]
    fn colon_operator() {
        assert_eq!(EvalResult::Err(Signal::Error(Error::InvalidRange)), r!(1:0));
        assert_eq!(r!([1, 2]), r!(1:2));
        assert_eq!(r!([1]), r!(1:1));
        assert_eq!(r!(1:-2:-3), r!([1, -1, -3]));
    }

    #[test]
    fn dollar_assign() {
        r_expect! {{"
            l = (a = 1, )
            x = (l$a = 2)
            l$a == 2 & x == 2
        "}}
    }
    #[test]
    fn dollar_assign_nested() {
        r_expect! {{"
            l = (a = (b = 1,),)
            x = (l$a$b = 2)
            l$a$b == 2 & x == 2
        "}}
    }

    #[test]
    fn dollar_access() {
        r_expect! {{"
            l = (a = 1, )
            l$a == 1
        "}}
    }
}
