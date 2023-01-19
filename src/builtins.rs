extern crate r_derive;

use r_derive::Primitive;

use crate::ast::*;
use crate::lang::*;
use crate::r_vector::vectors::*;
use crate::utils::*;

use std::rc::Rc;

fn match_args(
    mut formals: RExprList,
    mut args: Vec<(Option<String>, R)>,
    env: &Environment,
) -> (Vec<(Option<String>, R)>, Vec<(Option<String>, R)>) {
    let mut ellipsis: Vec<(Option<String>, R)> = vec![];
    let mut matched_args: Vec<(Option<String>, R)> = vec![];

    // assign named args to corresponding formals
    let mut i: usize = 0;
    while i < args.len() {
        match &args[i].0 {
            Some(argname) => {
                if let Some((Some(_), _)) = formals.remove_named(&argname) {
                    matched_args.push(args.remove(i));
                    continue;
                }
            }
            _ => (),
        }
        i += 1;
    }

    // remove any Ellipsis param, and any trailing unassigned params
    formals.pop_trailing();

    // backfill unnamed args, populating ellipsis with overflow
    for (key, value) in args.into_iter() {
        match key {
            // named args go directly to ellipsis, they did not match a formal
            Some(arg) => {
                ellipsis.push((Some(arg), value));
            }

            // unnamed args populate next formal, or ellipsis if formals exhausted
            None => {
                let next_unassigned_formal = formals.remove(0);
                if let Some((Some(param), _)) = next_unassigned_formal {
                    matched_args.push((Some(param), value));
                } else {
                    ellipsis.push((None, value));
                }
            }
        }
    }

    // add back in parameter defaults that weren't filled with args
    for (param, default) in formals.into_iter() {
        matched_args.push((param, R::Closure(default, Rc::clone(env))));
    }

    (matched_args, ellipsis)
}

impl std::fmt::Debug for Box<dyn Callable> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<callable>")
    }
}

impl std::fmt::Debug for Box<dyn Primitive> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

impl Clone for Box<dyn Primitive> {
    fn clone(&self) -> Box<dyn Primitive> {
        self.callable_clone()
    }
}

pub trait CallableClone: Callable {
    fn callable_clone(&self) -> Box<dyn Primitive>;
}

pub trait Callable {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult;
}

pub struct FormatState {
    // // character width of indentation
    // indent_size: usize,
    // // current number of indentations
    // indent_count: usize,
    // // current column for format start
    // start: usize,
    // // width of desired formatted code
    // width: usize,
}

impl Default for FormatState {
    fn default() -> Self {
        FormatState {
            // indent_size: 2,
            // indent_count: 0,
            // start: 0,
            // width: 80,
        }
    }
}

pub trait Format {
    fn rfmt_infix(s: &str, args: &RExprList) -> String
    where
        Self: Sized,
    {
        let state = FormatState::default();
        Self::rfmt_infix_with(s, state, args)
    }

    fn rfmt_infix_with(s: &str, _state: FormatState, args: &RExprList) -> String
    where
        Self: Sized,
    {
        format!("{} {s} {}", args.values[0], args.values[1])
    }

    fn rfmt(&self) -> String {
        let state = FormatState::default();
        self.rfmt_with(state)
    }

    fn rfmt_with(&self, _state: FormatState) -> String {
        "".to_string()
    }

    fn rfmt_call(&self, args: &RExprList) -> String {
        let state = FormatState::default();
        self.rfmt_call_with(state, args)
    }

    fn rfmt_call_with(&self, _state: FormatState, _args: &RExprList) -> String {
        "".to_string()
    }
}

pub trait Primitive: Callable + CallableClone + Format {}
pub trait Op: Primitive {}

#[derive(Debug, Clone, Primitive)]
pub struct ExprIf;

impl Format for ExprIf {
    fn rfmt_call_with(&self, _state: FormatState, args: &RExprList) -> String {
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

impl Callable for ExprIf {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
        let mut args = args.values.into_iter();

        let cond = eval(args.next().unwrap(), env)?;
        let cond: bool = cond.try_into()?;

        if cond {
            eval(args.next().unwrap(), env)
        } else {
            eval(args.skip(1).next().unwrap_or(RExpr::Null), env)
        }
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct ExprFor;

impl Format for ExprFor {
    fn rfmt_call_with(&self, _state: FormatState, args: &RExprList) -> String {
        let Some(sym) = &args.keys[0] else {
            unreachable!()
        };

        format!("for ({} in {}) {}", sym, args.values[0], args.values[1])
    }
}

impl Callable for ExprFor {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
        let mut args = args.into_iter();

        let (Some(var), iter_expr) = args.next().unwrap() else {
            unreachable!()
        };

        let (_, body) = args.next().unwrap();
        let iter = eval(iter_expr, env)?;

        let mut eval_result: EvalResult;
        let mut result = R::Null;
        let mut index = 0;

        while let Some(value) = iter.get(index) {
            index += 1;

            env.insert(var.clone(), value);
            eval_result = eval(body.clone(), env);

            // TODO: use std::ops::ControlFlow?
            match eval_result {
                Err(RSignal::Condition(Cond::Break)) => break,
                Err(RSignal::Condition(Cond::Continue)) => continue,
                Err(RSignal::Condition(Cond::Return(_))) => return eval_result,
                Err(RSignal::Error(_)) => return eval_result,
                _ => (),
            }

            result = eval_result.expect("unhandled eval err");
        }

        Ok(result)
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct ExprWhile;

impl Format for ExprWhile {
    fn rfmt_call_with(&self, _state: FormatState, args: &RExprList) -> String {
        format!("while ({}) {}", args.values[0], args.values[1])
    }
}

impl Callable for ExprWhile {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
        use Cond::*;
        use RSignal::*;

        let mut args = args.values.into_iter();
        let cond = args.next().unwrap();
        let body = args.next().unwrap();

        let mut eval_result: EvalResult;
        let mut result = R::Null;

        loop {
            // handle while condition
            let cond_result = eval(cond.clone(), env)?;
            if cond_result.try_into()? {
                eval_result = eval(body.clone(), env);
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

#[derive(Debug, Clone, Primitive)]
pub struct ExprRepeat;

impl Format for ExprRepeat {
    fn rfmt_call_with(&self, _state: FormatState, args: &RExprList) -> String {
        format!("repeat {}", args.values[1])
    }
}

impl Callable for ExprRepeat {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
        let mut args = args.values.into_iter();
        let body = args.next().unwrap();

        let mut eval_result: EvalResult;
        let mut result = R::Null;

        loop {
            eval_result = eval(body.clone(), env);

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

#[derive(Debug, Clone, Primitive)]
pub struct ExprBlock;

impl Format for ExprBlock {
    fn rfmt_call_with(&self, _state: FormatState, args: &RExprList) -> String {
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

impl Callable for ExprBlock {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
        let mut value = Ok(R::Null);
        for expr in args.values {
            let result = eval(expr, env);
            match result {
                Ok(_) => value = result,
                _ => return result,
            }
        }
        value
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct InfixAssign;

impl Format for InfixAssign {
    fn rfmt_call_with(&self, _state: FormatState, args: &RExprList) -> String {
        format!("{} <- {}", args.values[0], args.values[1])
    }
}

impl Callable for InfixAssign {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
        let (RExpr::Symbol(s), value) = args.unnamed_binary_args() else {
            unreachable!()
        };

        let value = eval(value, env)?;
        env.insert(s, value.clone());
        Ok(value)
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct InfixAdd;

impl Format for InfixAdd {
    fn rfmt_call_with(&self, state: FormatState, args: &RExprList) -> String {
        Self::rfmt_infix_with("+", state, args)
    }
}

impl Callable for InfixAdd {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
        let (lhs, rhs) = env.eval_binary(args.unnamed_binary_args())?;
        lhs + rhs
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct InfixSub;

impl Format for InfixSub {
    fn rfmt_call_with(&self, state: FormatState, args: &RExprList) -> String {
        Self::rfmt_infix_with("-", state, args)
    }
}

impl Callable for InfixSub {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
        let (lhs, rhs) = env.eval_binary(args.unnamed_binary_args())?;
        lhs - rhs
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct PrefixSub;

impl Format for PrefixSub {
    fn rfmt_call_with(&self, _state: FormatState, args: &RExprList) -> String {
        format!("-{}", args.values[0])
    }
}

impl Callable for PrefixSub {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
        let what = env.eval(args.unnamed_unary_arg())?;
        -what
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct InfixMul;

impl Format for InfixMul {
    fn rfmt_call_with(&self, _state: FormatState, args: &RExprList) -> String {
        format!("{} * {}", args.values[0], args.values[1])
    }
}

impl Callable for InfixMul {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
        let (lhs, rhs) = env.eval_binary(args.unnamed_binary_args())?;
        lhs * rhs
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct InfixDiv;

impl Format for InfixDiv {
    fn rfmt_call_with(&self, _state: FormatState, args: &RExprList) -> String {
        format!("{} / {}", args.values[0], args.values[1])
    }
}
impl Callable for InfixDiv {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
        let (lhs, rhs) = env.eval_binary(args.unnamed_binary_args())?;
        lhs / rhs
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct InfixPow;

impl Format for InfixPow {
    fn rfmt_call_with(&self, _state: FormatState, args: &RExprList) -> String {
        format!("{}^{}", args.values[0], args.values[1])
    }
}
impl Callable for InfixPow {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
        let (lhs, rhs) = env.eval_binary(args.unnamed_binary_args())?;
        lhs.power(rhs)
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct InfixMod;

impl Format for InfixMod {
    fn rfmt_call_with(&self, _state: FormatState, args: &RExprList) -> String {
        format!("{} % {}", args.values[0], args.values[1])
    }
}

impl Callable for InfixMod {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
        let (lhs, rhs) = env.eval_binary(args.unnamed_binary_args())?;
        lhs % rhs
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct InfixOr;

impl Format for InfixOr {
    fn rfmt_call_with(&self, _state: FormatState, args: &RExprList) -> String {
        format!("{} || {}", args.values[0], args.values[1])
    }
}

impl Callable for InfixOr {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
        let (lhs, rhs) = env.eval_binary(args.unnamed_binary_args())?;
        let res = match (lhs, rhs) {
            (R::Vector(l), R::Vector(r)) => {
                let Ok(lhs) = l.try_into() else { todo!() };
                let Ok(rhs) = r.try_into() else { todo!() };
                R::Vector(RVector::Logical(vec![OptionNA::Some(lhs || rhs)]))
            }
            _ => R::Null,
        };

        Ok(res)
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct InfixAnd;

impl Format for InfixAnd {
    fn rfmt_call_with(&self, _state: FormatState, args: &RExprList) -> String {
        format!("{} && {}", args.values[0], args.values[1])
    }
}

impl Callable for InfixAnd {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
        let (lhs, rhs) = env.eval_binary(args.unnamed_binary_args())?;
        let res = match (lhs, rhs) {
            (R::Vector(l), R::Vector(r)) => {
                let Ok(lhs) = l.try_into() else { todo!() };
                let Ok(rhs) = r.try_into() else { todo!() };
                R::Vector(RVector::Logical(vec![OptionNA::Some(lhs && rhs)]))
            }
            _ => R::Null,
        };

        Ok(res)
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct InfixVectorOr;

impl Format for InfixVectorOr {
    fn rfmt_call_with(&self, _state: FormatState, args: &RExprList) -> String {
        format!("{} | {}", args.values[0], args.values[1])
    }
}

impl Callable for InfixVectorOr {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
        let (lhs, rhs) = env.eval_binary(args.unnamed_binary_args())?;
        lhs | rhs
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct InfixVectorAnd;

impl Format for InfixVectorAnd {
    fn rfmt_call_with(&self, _state: FormatState, args: &RExprList) -> String {
        format!("{} & {}", args.values[0], args.values[1])
    }
}

impl Callable for InfixVectorAnd {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
        let (lhs, rhs) = env.eval_binary(args.unnamed_binary_args())?;
        lhs & rhs
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct InfixGreater;

impl Format for InfixGreater {
    fn rfmt_call_with(&self, _state: FormatState, args: &RExprList) -> String {
        format!("{} > {}", args.values[0], args.values[1])
    }
}

impl Callable for InfixGreater {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
        let (lhs, rhs) = env.eval_binary(args.unnamed_binary_args())?;
        lhs.vec_gt(rhs)
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct InfixGreaterEqual;

impl Format for InfixGreaterEqual {
    fn rfmt_call_with(&self, _state: FormatState, args: &RExprList) -> String {
        format!("{} >= {}", args.values[0], args.values[1])
    }
}

impl Callable for InfixGreaterEqual {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
        let (lhs, rhs) = env.eval_binary(args.unnamed_binary_args())?;
        lhs.vec_gte(rhs)
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct InfixLess;

impl Format for InfixLess {
    fn rfmt_call_with(&self, _state: FormatState, args: &RExprList) -> String {
        format!("{} < {}", args.values[0], args.values[1])
    }
}

impl Callable for InfixLess {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
        let (lhs, rhs) = env.eval_binary(args.unnamed_binary_args())?;
        lhs.vec_lt(rhs)
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct InfixLessEqual;

impl Format for InfixLessEqual {
    fn rfmt_call_with(&self, _state: FormatState, args: &RExprList) -> String {
        format!("{} > {}", args.values[0], args.values[1])
    }
}

impl Callable for InfixLessEqual {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
        let (lhs, rhs) = env.eval_binary(args.unnamed_binary_args())?;
        lhs.vec_lte(rhs)
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct InfixEqual;

impl Format for InfixEqual {
    fn rfmt_call_with(&self, _state: FormatState, args: &RExprList) -> String {
        format!("{} > {}", args.values[0], args.values[1])
    }
}

impl Callable for InfixEqual {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
        let (lhs, rhs) = env.eval_binary(args.unnamed_binary_args())?;
        lhs.vec_eq(rhs)
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct InfixNotEqual;

impl Format for InfixNotEqual {
    fn rfmt_call_with(&self, _state: FormatState, args: &RExprList) -> String {
        format!("{} > {}", args.values[0], args.values[1])
    }
}

impl Callable for InfixNotEqual {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
        let (lhs, rhs) = env.eval_binary(args.unnamed_binary_args())?;
        lhs.vec_neq(rhs)
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct PostfixIndex;

impl Format for PostfixIndex {
    fn rfmt_call_with(&self, _state: FormatState, args: &RExprList) -> String {
        let what = &args.values[0];

        let args = RExprList {
            keys: args.keys[1..].to_owned(),
            values: args.values[1..].to_owned(),
        };

        format!("{}[[{}]]", what, args)
    }
}

impl Callable for PostfixIndex {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
        let (what, index) = env.eval_binary(args.unnamed_binary_args())?;
        what.try_get(index)
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct PostfixVecIndex;

impl Format for PostfixVecIndex {
    fn rfmt_call_with(&self, _state: FormatState, args: &RExprList) -> String {
        format!("{}[{}]", args.values[0], args.values[1])
    }
}

impl Callable for PostfixVecIndex {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
        let (what, index) = env.eval_binary(args.unnamed_binary_args())?;
        what.try_get(index)
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct ExprVec;

impl Format for ExprVec {
    fn rfmt_call_with(&self, _state: FormatState, args: &RExprList) -> String {
        format!("({})", args)
    }
}

impl Callable for ExprVec {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
        // for now just use c()
        primitive_c(args, env)
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct ExprList;

impl Format for ExprList {
    fn rfmt_call_with(&self, _state: FormatState, args: &RExprList) -> String {
        format!("[{}]", args)
    }
}

impl Callable for ExprList {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
        let vals: Result<Vec<_>, _> = args
            .into_iter()
            .map(|(n, v)| match env.eval(v) {
                Ok(val) => Ok((n, val)),
                Err(err) => Err(err),
            })
            .collect();

        Ok(R::List(vals?))
    }
}

#[derive(Debug, Clone)]
pub struct Name(String);

pub fn primitive(name: &str) -> Option<Box<dyn Fn(RExprList, &mut Environment) -> EvalResult>> {
    match name {
        "c" => Some(Box::new(primitive_c)),
        "list" => Some(Box::new(primitive_list)),
        _ => None,
    }
}

pub fn primitive_list(args: RExprList, env: &mut Environment) -> EvalResult {
    ExprList::call(&ExprList, args, env)
}

pub fn primitive_c(args: RExprList, env: &mut Environment) -> EvalResult {
    // this can be cleaned up quite a bit, but I just need it working with
    // more types for now to test vectorized operators using different types

    let R::List(vals) = eval_rexprlist(args, env)? else {
        unreachable!()
    };

    // force any closures that were created during call
    let vals: Vec<_> = vals
        .into_iter()
        .map(|(k, v)| (k, force(v).unwrap_or(R::Null))) // TODO: raise this error
        .collect();

    // until there's a better way of handling type hierarchy, this will do
    let t: u8 = vals
        .iter()
        .map(|(_, v)| match v {
            R::Null => 0,
            R::Vector(vec) => match vec {
                RVector::Logical(_) => 1,
                RVector::Integer(_) => 2,
                RVector::Numeric(_) => 3,
                RVector::Character(_) => 4,
            },
            R::List(_) => 5,
            _ => 0,
        })
        .fold(0, std::cmp::max);

    match t {
        0 => Ok(R::Null),
        // Coerce everything into logical
        1 => {
            let mut output = vec![OptionNA::Some(true); 0];
            for (_, val) in vals {
                match val {
                    R::Null => continue,
                    R::Vector(RVector::Logical(mut v)) => output.append(&mut v),
                    _ => unimplemented!(),
                }
            }
            Ok(R::Vector(RVector::Logical(output)))
        }
        // Coerce everything into integer
        2 => {
            let mut output = vec![OptionNA::Some(0); 0];
            for (_, val) in vals {
                match val {
                    R::Null => continue,
                    R::Vector(RVector::Integer(mut v)) => output.append(&mut v),
                    R::Vector(RVector::Logical(v)) => {
                        output.append(&mut RVector::vec_coerce::<bool, i32>(&v))
                    }
                    _ => unimplemented!(),
                }
            }
            Ok(R::Vector(RVector::Integer(output)))
        }
        // Coerce everything into numeric
        3 => {
            let mut output = vec![OptionNA::Some(0.0); 0];
            for (_, val) in vals {
                match val {
                    R::Null => continue,
                    R::Vector(RVector::Numeric(mut v)) => output.append(&mut v),
                    R::Vector(RVector::Integer(v)) => {
                        output.append(&mut RVector::vec_coerce::<i32, f64>(&v))
                    }
                    R::Vector(RVector::Logical(v)) => {
                        output.append(&mut RVector::vec_coerce::<bool, f64>(&v))
                    }
                    _ => {
                        println!("{:#?}", val);
                        unimplemented!()
                    }
                }
            }
            Ok(R::Vector(RVector::Numeric(output)))
        }
        // coerce everything into strings
        4 => {
            let mut output = vec![OptionNA::Some("".to_string()); 0];
            for (_, val) in vals {
                match val {
                    R::Null => continue,
                    R::Vector(RVector::Numeric(v)) => {
                        output.append(&mut RVector::vec_coerce::<f64, String>(&v))
                    }
                    R::Vector(RVector::Integer(v)) => {
                        output.append(&mut RVector::vec_coerce::<i32, String>(&v))
                    }
                    R::Vector(RVector::Logical(v)) => {
                        output.append(&mut RVector::vec_coerce::<bool, String>(&v))
                    }
                    R::Vector(RVector::Character(mut v)) => output.append(&mut v),
                    _ => {
                        println!("{:#?}", val);
                        unimplemented!()
                    }
                }
            }
            Ok(R::Vector(RVector::Character(output)))
        }
        _ => Ok(R::Null),
    }
}

impl Format for String {
    fn rfmt_call_with(&self, _state: FormatState, args: &RExprList) -> String {
        format!("{}({})", self, args)
    }
}

impl Callable for String {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
        if let Some(f) = primitive(self) {
            return f(args, env);
        }

        (env.get(self.clone())?).call(args, env)
    }
}

impl Format for R {}

impl Callable for R {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
        let R::Function(formals, body, fn_env) = self else {
            unimplemented!("can't call non-function")
        };

        // set up our local scope, a child environment of the function environment
        let local_scope = Environment::new(Env {
            parent: Some(Rc::clone(fn_env)),
            ..Default::default()
        });

        // evaluate arguments in calling environment
        let R::List(args) = eval_rexprlist(args, env)? else {
            unreachable!();
        };

        // match arguments against function signature
        let (args, ellipsis) = match_args(formals.clone(), args, env);

        // add closures to local scope
        local_scope.insert("...".to_string(), R::List(ellipsis));
        local_scope.append(R::List(args));

        // evaluate body in local scope
        eval(body.clone(), &mut Rc::clone(&local_scope))
    }
}
