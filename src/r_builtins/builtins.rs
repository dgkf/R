extern crate r_derive;
use r_derive::*;

use crate::ast::*;
use crate::lang::*;
use crate::r_vector::vectors::*;

use super::paste::*;
use std::rc::Rc;

fn match_args(
    mut formals: ExprList,
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
    fn call_assign(&self, _value: Expr, _args: ExprList, _env: &mut Environment) -> EvalResult {
        unimplemented!();
    }

    fn call(&self, args: ExprList, env: &mut Environment) -> EvalResult;
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
    fn rfmt_infix(s: &str, args: &ExprList) -> String
    where
        Self: Sized,
    {
        let state = FormatState::default();
        Self::rfmt_infix_with(s, state, args)
    }

    fn rfmt_infix_with(s: &str, _state: FormatState, args: &ExprList) -> String
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

    fn rfmt_call(&self, args: &ExprList) -> String {
        let state = FormatState::default();
        self.rfmt_call_with(state, args)
    }

    fn rfmt_call_with(&self, _state: FormatState, _args: &ExprList) -> String {
        "".to_string()
    }
}

pub trait Primitive: Callable + CallableClone + Format {}
pub trait Op {
    const SYM: &'static str;
}

impl<T> Format for T
where
    T: Op,
{
    fn rfmt_call_with(&self, _state: FormatState, args: &ExprList) -> String {
        let sym = Self::SYM;
        format!("{} {sym} {}", args.values[0], args.values[1])
    }
}

#[derive(Debug, Clone, Primitive)]
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
    fn call(&self, args: ExprList, env: &mut Environment) -> EvalResult {
        let mut args = args.values.into_iter();

        let cond = env.eval(args.next().unwrap())?;
        let cond: bool = cond.try_into()?;

        if cond {
            env.eval(args.next().unwrap())
        } else {
            env.eval(args.skip(1).next().unwrap_or(Expr::Null))
        }
    }
}

#[derive(Debug, Clone, Primitive)]
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
    fn call(&self, args: ExprList, env: &mut Environment) -> EvalResult {
        let mut args = args.into_iter();

        let (Some(var), iter_expr) = args.next().unwrap() else {
            unreachable!()
        };

        let (_, body) = args.next().unwrap();
        let iter = env.eval(iter_expr)?;

        let mut eval_result: EvalResult;
        let mut result = R::Null;
        let mut index = 0;

        while let Some(value) = iter.get(index) {
            index += 1;

            env.insert(var.clone(), value);
            eval_result = env.eval(body.clone());

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

#[derive(Debug, Clone, Primitive)]
pub struct PrimWhile;

impl Format for PrimWhile {
    fn rfmt_call_with(&self, _state: FormatState, args: &ExprList) -> String {
        format!("while ({}) {}", args.values[0], args.values[1])
    }
}

impl Callable for PrimWhile {
    fn call(&self, args: ExprList, env: &mut Environment) -> EvalResult {
        use Cond::*;
        use RSignal::*;

        let mut args = args.values.into_iter();
        let cond = args.next().unwrap();
        let body = args.next().unwrap();

        let mut eval_result: EvalResult;
        let mut result = R::Null;

        loop {
            // handle while condition
            let cond_result = env.eval(cond.clone())?;
            if cond_result.try_into()? {
                eval_result = env.eval(body.clone());
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
pub struct PrimRepeat;

impl Format for PrimRepeat {
    fn rfmt_call_with(&self, _state: FormatState, args: &ExprList) -> String {
        format!("repeat {}", args.values[1])
    }
}

impl Callable for PrimRepeat {
    fn call(&self, args: ExprList, env: &mut Environment) -> EvalResult {
        let mut args = args.values.into_iter();
        let body = args.next().unwrap();

        let mut eval_result: EvalResult;
        let mut result = R::Null;

        loop {
            eval_result = env.eval(body.clone());

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
    fn call(&self, args: ExprList, env: &mut Environment) -> EvalResult {
        let mut value = Ok(R::Null);
        for expr in args.values {
            let result = env.eval(expr);
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

impl Op for InfixAssign {
    const SYM: &'static str = "<-";
}

impl Callable for InfixAssign {
    fn call(&self, args: ExprList, env: &mut Environment) -> EvalResult {
        let (lhs, rhs) = args.unnamed_binary_args();

        use Expr::*;
        match lhs {
            String(s) | Symbol(s) => {
                let value = env.eval(rhs)?;
                env.insert(s, value.clone());
                Ok(value)
            }
            Call(what, mut args) => match *what {
                Primitive(prim) => prim.call_assign(rhs, args, env),
                String(s) | Symbol(s) => {
                    args.insert(0, rhs);
                    let s = format!("{}<-", s);
                    env.eval(Call(Box::new(Symbol(s)), args))
                }
                _ => unreachable!(),
            },
            _ => unimplemented!("cannot assign to that!"),
        }
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct InfixAdd;

impl Op for InfixAdd {
    const SYM: &'static str = "+";
}

impl Callable for InfixAdd {
    fn call(&self, args: ExprList, env: &mut Environment) -> EvalResult {
        let (lhs, rhs) = env.eval_binary(args.unnamed_binary_args())?;
        lhs + rhs
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct InfixSub;

impl Op for InfixSub {
    const SYM: &'static str = "-";
}

impl Callable for InfixSub {
    fn call(&self, args: ExprList, env: &mut Environment) -> EvalResult {
        let (lhs, rhs) = env.eval_binary(args.unnamed_binary_args())?;
        lhs - rhs
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct PrefixSub;

impl Format for PrefixSub {
    fn rfmt_call_with(&self, _state: FormatState, args: &ExprList) -> String {
        format!("-{}", args.values[0])
    }
}

impl Callable for PrefixSub {
    fn call(&self, args: ExprList, env: &mut Environment) -> EvalResult {
        let what = env.eval(args.unnamed_unary_arg())?;
        -what
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct InfixMul;

impl Op for InfixMul {
    const SYM: &'static str = "*";
}

impl Callable for InfixMul {
    fn call(&self, args: ExprList, env: &mut Environment) -> EvalResult {
        let (lhs, rhs) = env.eval_binary(args.unnamed_binary_args())?;
        lhs * rhs
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct InfixDiv;

impl Op for InfixDiv {
    const SYM: &'static str = "/";
}

impl Callable for InfixDiv {
    fn call(&self, args: ExprList, env: &mut Environment) -> EvalResult {
        let (lhs, rhs) = env.eval_binary(args.unnamed_binary_args())?;
        lhs / rhs
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct InfixPow;

impl Op for InfixPow {
    const SYM: &'static str = "*";
}

impl Callable for InfixPow {
    fn call(&self, args: ExprList, env: &mut Environment) -> EvalResult {
        let (lhs, rhs) = env.eval_binary(args.unnamed_binary_args())?;
        lhs.power(rhs)
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct InfixMod;

impl Op for InfixMod {
    const SYM: &'static str = "%";
}

impl Callable for InfixMod {
    fn call(&self, args: ExprList, env: &mut Environment) -> EvalResult {
        let (lhs, rhs) = env.eval_binary(args.unnamed_binary_args())?;
        lhs % rhs
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct InfixOr;

impl Op for InfixOr {
    const SYM: &'static str = "||";
}

impl Callable for InfixOr {
    fn call(&self, args: ExprList, env: &mut Environment) -> EvalResult {
        let (lhs, rhs) = env.eval_binary(args.unnamed_binary_args())?;
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

#[derive(Debug, Clone, Primitive)]
pub struct InfixAnd;

impl Op for InfixAnd {
    const SYM: &'static str = "&&";
}

impl Callable for InfixAnd {
    fn call(&self, args: ExprList, env: &mut Environment) -> EvalResult {
        let (lhs, rhs) = env.eval_binary(args.unnamed_binary_args())?;
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

#[derive(Debug, Clone, Primitive)]
pub struct InfixVectorOr;

impl Op for InfixVectorOr {
    const SYM: &'static str = "|";
}

impl Callable for InfixVectorOr {
    fn call(&self, args: ExprList, env: &mut Environment) -> EvalResult {
        let (lhs, rhs) = env.eval_binary(args.unnamed_binary_args())?;
        lhs | rhs
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct InfixVectorAnd;

impl Op for InfixVectorAnd {
    const SYM: &'static str = "&";
}

impl Callable for InfixVectorAnd {
    fn call(&self, args: ExprList, env: &mut Environment) -> EvalResult {
        let (lhs, rhs) = env.eval_binary(args.unnamed_binary_args())?;
        lhs & rhs
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct InfixGreater;

impl Op for InfixGreater {
    const SYM: &'static str = ">";
}

impl Callable for InfixGreater {
    fn call(&self, args: ExprList, env: &mut Environment) -> EvalResult {
        let (lhs, rhs) = env.eval_binary(args.unnamed_binary_args())?;
        lhs.vec_gt(rhs)
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct InfixGreaterEqual;

impl Op for InfixGreaterEqual {
    const SYM: &'static str = ">=";
}

impl Callable for InfixGreaterEqual {
    fn call(&self, args: ExprList, env: &mut Environment) -> EvalResult {
        let (lhs, rhs) = env.eval_binary(args.unnamed_binary_args())?;
        lhs.vec_gte(rhs)
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct InfixLess;

impl Op for InfixLess {
    const SYM: &'static str = "<";
}

impl Callable for InfixLess {
    fn call(&self, args: ExprList, env: &mut Environment) -> EvalResult {
        let (lhs, rhs) = env.eval_binary(args.unnamed_binary_args())?;
        lhs.vec_lt(rhs)
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct InfixLessEqual;

impl Op for InfixLessEqual {
    const SYM: &'static str = "<=";
}

impl Callable for InfixLessEqual {
    fn call(&self, args: ExprList, env: &mut Environment) -> EvalResult {
        let (lhs, rhs) = env.eval_binary(args.unnamed_binary_args())?;
        lhs.vec_lte(rhs)
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct InfixEqual;

impl Op for InfixEqual {
    const SYM: &'static str = "==";
}

impl Callable for InfixEqual {
    fn call(&self, args: ExprList, env: &mut Environment) -> EvalResult {
        let (lhs, rhs) = env.eval_binary(args.unnamed_binary_args())?;
        lhs.vec_eq(rhs)
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct InfixNotEqual;

impl Op for InfixNotEqual {
    const SYM: &'static str = "!=";
}

impl Callable for InfixNotEqual {
    fn call(&self, args: ExprList, env: &mut Environment) -> EvalResult {
        let (lhs, rhs) = env.eval_binary(args.unnamed_binary_args())?;
        lhs.vec_neq(rhs)
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct InfixPipe;

impl Op for InfixPipe {
    const SYM: &'static str = "|>";
}

impl Callable for InfixPipe {
    fn call(&self, args: ExprList, env: &mut Environment) -> EvalResult {
        // TODO: reduce call stack nesting here
        let (lhs, rhs) = args.unnamed_binary_args();

        use Expr::*;
        match rhs {
            Call(what, mut args) => {
                args.insert(0, lhs);
                let new_expr = Call(what, args);
                env.eval(new_expr)
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Primitive)]
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
    fn call(&self, args: ExprList, env: &mut Environment) -> EvalResult {
        let (what, index) = env.eval_binary(args.unnamed_binary_args())?;
        what.try_get(index)
    }

    fn call_assign(&self, value: Expr, args: ExprList, env: &mut Environment) -> EvalResult {
        let value = env.eval(value)?;
        let (what, index) = env.eval_binary(args.unnamed_binary_args())?;

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

#[derive(Debug, Clone, Primitive)]
pub struct PostfixVecIndex;

impl Format for PostfixVecIndex {
    fn rfmt_call_with(&self, _state: FormatState, args: &ExprList) -> String {
        format!("{}[{}]", args.values[0], args.values[1])
    }
}

impl Callable for PostfixVecIndex {
    fn call(&self, args: ExprList, env: &mut Environment) -> EvalResult {
        let (what, index) = env.eval_binary(args.unnamed_binary_args())?;
        what.try_get(index)
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct PrimVec;

impl Format for PrimVec {
    fn rfmt_call_with(&self, _state: FormatState, args: &ExprList) -> String {
        format!("[{}]", args)
    }
}

impl Callable for PrimVec {
    fn call(&self, args: ExprList, env: &mut Environment) -> EvalResult {
        // for now just use c()
        primitive_c(args, env)
    }
}

#[derive(Debug, Clone, Primitive)]
pub struct PrimList;

impl Format for PrimList {
    fn rfmt_call_with(&self, _state: FormatState, args: &ExprList) -> String {
        format!("({})", args)
    }
}

impl Callable for PrimList {
    fn call(&self, args: ExprList, env: &mut Environment) -> EvalResult {
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

pub fn primitive(name: &str) -> Option<Box<dyn Fn(ExprList, &mut Environment) -> EvalResult>> {
    match name {
        "c" => Some(Box::new(primitive_c)),
        "list" => Some(Box::new(primitive_list)),
        "paste" => Some(Box::new(primitive_paste)),
        "q" => Some(Box::new(primitive_q)),
        _ => None,
    }
}

pub fn primitive_q(_args: ExprList, _env: &mut Environment) -> EvalResult {
    Err(RSignal::Condition(Cond::Terminate))
}

pub fn primitive_list(args: ExprList, env: &mut Environment) -> EvalResult {
    PrimList::call(&PrimList, args, env)
}

pub fn force_closures(vals: Vec<(Option<String>, R)>) -> Vec<(Option<String>, R)> {
    // Force any closures that were created during call. This helps with using
    // variables as argument for sep and collapse parameters.
    vals.into_iter()
        .map(|(k, v)| (k, v.clone().force().unwrap_or(R::Null))) // TODO: raise this error
        .collect()
}

pub fn primitive_c(args: ExprList, env: &mut Environment) -> EvalResult {
    // this can be cleaned up quite a bit, but I just need it working with
    // more types for now to test vectorized operators using different types

    let R::List(vals) = env.eval_list(args)? else {
        unreachable!()
    };

    let vals = force_closures(vals);

    // until there's a better way of handling type hierarchy, this will do
    let t: u8 = vals
        .iter()
        .map(|(_, v)| match v {
            R::Null => 0,
            R::Vector(vec) => match vec {
                Vector::Logical(_) => 1,
                Vector::Integer(_) => 2,
                Vector::Numeric(_) => 3,
                Vector::Character(_) => 4,
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
                    R::Vector(Vector::Logical(mut v)) => output.append(&mut v),
                    _ => unimplemented!(),
                }
            }
            Ok(R::Vector(Vector::Logical(output)))
        }
        // Coerce everything into integer
        2 => {
            let mut output = vec![OptionNA::Some(0); 0];
            for (_, val) in vals {
                match val {
                    R::Null => continue,
                    R::Vector(Vector::Integer(mut v)) => output.append(&mut v),
                    R::Vector(Vector::Logical(v)) => {
                        output.append(&mut Vector::vec_coerce::<bool, i32>(&v))
                    }
                    _ => unimplemented!(),
                }
            }
            Ok(R::Vector(Vector::Integer(output)))
        }
        // Coerce everything into numeric
        3 => {
            let mut output = vec![OptionNA::Some(0.0); 0];
            for (_, val) in vals {
                match val {
                    R::Null => continue,
                    R::Vector(Vector::Numeric(mut v)) => output.append(&mut v),
                    R::Vector(Vector::Integer(v)) => {
                        output.append(&mut Vector::vec_coerce::<i32, f64>(&v))
                    }
                    R::Vector(Vector::Logical(v)) => {
                        output.append(&mut Vector::vec_coerce::<bool, f64>(&v))
                    }
                    _ => {
                        println!("{:#?}", val);
                        unimplemented!()
                    }
                }
            }
            Ok(R::Vector(Vector::Numeric(output)))
        }
        // coerce everything into strings
        4 => {
            let mut output = vec![OptionNA::Some("".to_string()); 0];
            for (_, val) in vals {
                match val {
                    R::Null => continue,
                    R::Vector(Vector::Numeric(v)) => {
                        output.append(&mut Vector::vec_coerce::<f64, String>(&v))
                    }
                    R::Vector(Vector::Integer(v)) => {
                        output.append(&mut Vector::vec_coerce::<i32, String>(&v))
                    }
                    R::Vector(Vector::Logical(v)) => {
                        output.append(&mut Vector::vec_coerce::<bool, String>(&v))
                    }
                    R::Vector(Vector::Character(mut v)) => output.append(&mut v),
                    _ => {
                        println!("{:#?}", val);
                        unimplemented!()
                    }
                }
            }
            Ok(R::Vector(Vector::Character(output)))
        }
        _ => Ok(R::Null),
    }
}

impl Format for String {
    fn rfmt_call_with(&self, _state: FormatState, args: &ExprList) -> String {
        format!("{}({})", self, args)
    }
}

impl Callable for String {
    fn call(&self, args: ExprList, env: &mut Environment) -> EvalResult {
        if let Some(f) = primitive(self) {
            return f(args, env);
        }

        (env.get(self.clone())?).call(args, env)
    }
}

impl Format for R {}

impl Callable for R {
    fn call(&self, args: ExprList, env: &mut Environment) -> EvalResult {
        let R::Function(formals, body, fn_env) = self else {
            unimplemented!("can't call non-function")
        };

        // set up our local scope, a child environment of the function environment
        let mut local_scope = Environment::new(Env {
            parent: Some(Rc::clone(fn_env)),
            ..Default::default()
        });

        // evaluate arguments in calling environment
        let R::List(args) = env.eval_list(args)? else {
            unreachable!();
        };

        // match arguments against function signature
        let (args, ellipsis) = match_args(formals.clone(), args, env);

        // add closures to local scope
        local_scope.insert("...".to_string(), R::List(ellipsis));
        local_scope.append(R::List(args));

        // evaluate body in local scope
        local_scope.eval(body.clone())
    }
}
