use r_derive::Primitive;

use crate::ast::*;
use crate::lang::*;
use crate::vector::vectors::*;
use super::core::*;

#[derive(Debug, Clone, Primitive, PartialEq)]
pub struct InfixAssign;

impl PrimitiveSYM for InfixAssign {
    const SYM: &'static str = "<-";
}

impl Callable for InfixAssign {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = args.unnamed_binary_args();

        use Expr::*;
        match lhs {
            String(s) | Symbol(s) => {
                let value = stack.eval(rhs)?;
                stack.last_frame().env.insert(s, value.clone());
                Ok(value)
            }
            Call(what, mut args) => match *what {
                Primitive(prim) => prim.call_assign(rhs, args, stack),
                String(s) | Symbol(s) => {
                    args.insert(0, rhs);
                    let s = format!("{}<-", s);
                    stack.eval(Call(Box::new(Symbol(s)), args))
                }
                _ => unreachable!(),
            },
            _ => unimplemented!("cannot assign to that!"),
        }
    }
}

#[derive(Debug, Clone, Primitive, PartialEq)]
pub struct InfixAdd;

impl PrimitiveSYM for InfixAdd {
    const SYM: &'static str = "+";
}

impl Callable for InfixAdd {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs + rhs
    }
}

#[derive(Debug, Clone, Primitive, PartialEq)]
pub struct InfixSub;

impl PrimitiveSYM for InfixSub {
    const SYM: &'static str = "-";
}

impl Callable for InfixSub {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs - rhs
    }
}

#[derive(Debug, Clone, Primitive, PartialEq)]
pub struct PrefixSub;

impl Format for PrefixSub {
    fn rfmt_call_with(&self, _state: FormatState, args: &ExprList) -> String {
        format!("-{}", args.values[0])
    }
}

impl Callable for PrefixSub {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let what = stack.eval(args.unnamed_unary_arg())?;
        -what
    }
}

#[derive(Debug, Clone, Primitive, PartialEq)]
pub struct InfixMul;

impl PrimitiveSYM for InfixMul {
    const SYM: &'static str = "*";
}

impl Callable for InfixMul {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs * rhs
    }
}

#[derive(Debug, Clone, Primitive, PartialEq)]
pub struct InfixDiv;

impl PrimitiveSYM for InfixDiv {
    const SYM: &'static str = "/";
}

impl Callable for InfixDiv {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs / rhs
    }
}

#[derive(Debug, Clone, Primitive, PartialEq)]
pub struct InfixPow;

impl PrimitiveSYM for InfixPow {
    const SYM: &'static str = "*";
}

impl Callable for InfixPow {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs.power(rhs)
    }
}

#[derive(Debug, Clone, Primitive, PartialEq)]
pub struct InfixMod;

impl PrimitiveSYM for InfixMod {
    const SYM: &'static str = "%";
}

impl Callable for InfixMod {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs % rhs
    }
}

#[derive(Debug, Clone, Primitive, PartialEq)]
pub struct InfixOr;

impl PrimitiveSYM for InfixOr {
    const SYM: &'static str = "||";
}

impl Callable for InfixOr {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        let res = match (lhs, rhs) {
            (R::Vector(l), R::Vector(r)) => {
                let Ok(lhs) = l.try_into() else { todo!() };
                let Ok(rhs) = r.try_into() else { todo!() };
                R::Vector(Vector::Logical(vec![OptionNA::Some(lhs || rhs)]))
            }
            _ => R::Null,
        };

        Ok(res)
    }
}

#[derive(Debug, Clone, Primitive, PartialEq)]
pub struct InfixAnd;

impl PrimitiveSYM for InfixAnd {
    const SYM: &'static str = "&&";
}

impl Callable for InfixAnd {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        let res = match (lhs, rhs) {
            (R::Vector(l), R::Vector(r)) => {
                let Ok(lhs) = l.try_into() else { todo!() };
                let Ok(rhs) = r.try_into() else { todo!() };
                R::Vector(Vector::Logical(vec![OptionNA::Some(lhs && rhs)]))
            }
            _ => R::Null,
        };

        Ok(res)
    }
}

#[derive(Debug, Clone, Primitive, PartialEq)]
pub struct InfixVectorOr;

impl PrimitiveSYM for InfixVectorOr {
    const SYM: &'static str = "|";
}

impl Callable for InfixVectorOr {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs | rhs
    }
}

#[derive(Debug, Clone, Primitive, PartialEq)]
pub struct InfixVectorAnd;

impl PrimitiveSYM for InfixVectorAnd {
    const SYM: &'static str = "&";
}

impl Callable for InfixVectorAnd {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs & rhs
    }
}

#[derive(Debug, Clone, Primitive, PartialEq)]
pub struct InfixGreater;

impl PrimitiveSYM for InfixGreater {
    const SYM: &'static str = ">";
}

impl Callable for InfixGreater {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs.vec_gt(rhs)
    }
}

#[derive(Debug, Clone, Primitive, PartialEq)]
pub struct InfixGreaterEqual;

impl PrimitiveSYM for InfixGreaterEqual {
    const SYM: &'static str = ">=";
}

impl Callable for InfixGreaterEqual {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs.vec_gte(rhs)
    }
}

#[derive(Debug, Clone, Primitive, PartialEq)]
pub struct InfixLess;

impl PrimitiveSYM for InfixLess {
    const SYM: &'static str = "<";
}

impl Callable for InfixLess {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs.vec_lt(rhs)
    }
}

#[derive(Debug, Clone, Primitive, PartialEq)]
pub struct InfixLessEqual;

impl PrimitiveSYM for InfixLessEqual {
    const SYM: &'static str = "<=";
}

impl Callable for InfixLessEqual {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs.vec_lte(rhs)
    }
}

#[derive(Debug, Clone, Primitive, PartialEq)]
pub struct InfixEqual;

impl PrimitiveSYM for InfixEqual {
    const SYM: &'static str = "==";
}

impl Callable for InfixEqual {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs.vec_eq(rhs)
    }
}

#[derive(Debug, Clone, Primitive, PartialEq)]
pub struct InfixNotEqual;

impl PrimitiveSYM for InfixNotEqual {
    const SYM: &'static str = "!=";
}

impl Callable for InfixNotEqual {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (lhs, rhs) = stack.eval_binary(args.unnamed_binary_args())?;
        lhs.vec_neq(rhs)
    }
}

#[derive(Debug, Clone, Primitive, PartialEq)]
pub struct InfixPipe;

impl PrimitiveSYM for InfixPipe {
    const SYM: &'static str = "|>";
}

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
            },
            s @ Symbol(..) | s @ String(..) => {
                let args = ExprList::from(vec![(None, lhs)]);
                let new_expr = Call(Box::new(s), args);
                stack.eval(new_expr)
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Primitive, PartialEq)]
pub struct InfixColon;

impl PrimitiveSYM for InfixColon {
    const SYM: &'static str = ":";
}

impl Callable for InfixColon {
    fn call(&self, args: ExprList, _stack: &mut CallStack) -> EvalResult {
        // TODO: reduce call stack nesting here
        let (lhs, rhs) = args.unnamed_binary_args();

        use Expr::*;
        match (lhs, rhs) {
            (Number(l), Number(r)) => Ok(R::Vector(Vector::from(((l as i32)..=(r as i32)).into_iter().collect::<Vec<i32>>()))),
            (Number(l), Integer(r)) => Ok(R::Vector(Vector::from(((l as i32)..=(r as i32)).into_iter().collect::<Vec<i32>>()))),
            (Integer(l), Number(r)) => Ok(R::Vector(Vector::from(((l as i32)..=(r as i32)).into_iter().collect::<Vec<i32>>()))),
            (Integer(l), Integer(r)) => Ok(R::Vector(Vector::from(((l as i32)..=(r as i32)).into_iter().collect::<Vec<i32>>()))),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Primitive, PartialEq)]
pub struct PostfixIndex;

impl Format for PostfixIndex {
    fn rfmt_call_with(&self, _state: FormatState, args: &ExprList) -> String {
        let what = &args.values[0];

        let args = ExprList {
            keys: args.keys[1..].to_owned(),
            values: args.values[1..].to_owned(),
        };

        format!("{}[[{}]]", what, args)
    }
}

impl Callable for PostfixIndex {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (what, index) = stack.eval_binary(args.unnamed_binary_args())?;
        what.try_get(index)
    }

    fn call_assign(&self, value: Expr, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let value = stack.eval(value)?;
        let (what, index) = stack.eval_binary(args.unnamed_binary_args())?;

        use R::*;
        match (what, value, index.as_integer()?) {
            (Vector(mut lrvec), Vector(rrvec), Vector(i)) => {
                lrvec.set_from_vec(i, rrvec)?;
                Ok(Vector(lrvec))
            }
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone, Primitive, PartialEq)]
pub struct PostfixVecIndex;

impl Format for PostfixVecIndex {
    fn rfmt_call_with(&self, _state: FormatState, args: &ExprList) -> String {
        format!("{}[{}]", args.values[0], args.values[1])
    }
}

impl Callable for PostfixVecIndex {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (what, index) = stack.eval_binary(args.unnamed_binary_args())?;
        what.try_get(index)
    }
}

#[derive(Debug, Clone, Primitive, PartialEq)]
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
