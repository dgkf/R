extern crate r_derive;

use crate::ast::*;
use crate::callable::operators::*;
use crate::callable::dyncompare::*;
use crate::lang::*;

pub fn match_args(
    mut formals: ExprList,
    mut args: Vec<(Option<String>, R)>,
    stack: &CallStack,
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
        matched_args.push((param, R::Closure(default, stack.env().clone())));
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
    fn formals(&self) -> ExprList {
        ExprList::new()
    }

    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult;

    fn call_assign(&self, _value: Expr, _args: ExprList, _stack: &mut CallStack) -> EvalResult {
        unimplemented!();
    }
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

pub trait Primitive: Callable + CallableClone + Format + DynCompare {
}

impl PartialEq<dyn Primitive> for dyn Primitive {
    fn eq(&self, other: &dyn Primitive) -> bool {
        self.as_dyn_compare() == other.as_dyn_compare()
    }
}

pub trait PrimitiveSYM {
    const SYM: &'static str;
}

impl<T> Format for T
where
    T: PrimitiveSYM,
{
    fn rfmt_call_with(&self, _state: FormatState, args: &ExprList) -> String {
        let sym = Self::SYM;
        format!("{} {sym} {}", args.values[0], args.values[1])
    }

    fn rfmt_with(&self, _: FormatState) -> String {
        Self::SYM.to_string()
    }
}

// Allow Strings to be used via Expr::Primitive(String)

impl CallableClone for String
where
    Self: Callable
{
    fn callable_clone(&self) -> Box<dyn Primitive> {
        Box::new(self.clone())
    }
}

impl Primitive for String {}

pub fn string_as_primitive(s: &str) -> Result<Box<dyn Primitive>, ()> {
    use crate::callable::primitive::*;
    match s {
        "|>" => Ok(Box::new(InfixPipe)),
        "c" => Ok(Box::new(PrimitiveC)),
        "callstack" => Ok(Box::new(PrimitiveCallstack)),
        "list" => Ok(Box::new(PrimitiveList)),
        "paste" => Ok(Box::new(PrimitivePaste)),
        "q" => Ok(Box::new(PrimitiveQ)),
        "rnorm" => Ok(Box::new(PrimitiveRnorm)),
        "runif" => Ok(Box::new(PrimitiveRunif)),
        _ => Err(()),
    }
}

impl TryFrom<&str> for Box<dyn Primitive> {
    type Error = ();
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        string_as_primitive(s)
    }
}

pub fn force_closures(vals: Vec<(Option<String>, R)>, stack: &mut CallStack) -> Vec<(Option<String>, R)> {
    // Force any closures that were created during call. This helps with using
    // variables as argument for sep and collapse parameters.
    vals.into_iter()
        .map(|(k, v)| (k, v.clone().force(stack).unwrap_or(R::Null))) // TODO: raise this error
        .collect()
}

impl Format for String {
    fn rfmt_call_with(&self, _state: FormatState, args: &ExprList) -> String {
        format!("{}({})", self, args)
    }
}

impl Callable for String {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        (stack.last_frame().env.clone().get(self.clone())?).call(args, stack)
    }
}

impl Format for R {}

impl Callable for R {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let R::Function(formals, body, _) = self else {
            unimplemented!("can't call non-function")
        };

        // body is a primitive
        if let Expr::Primitive(f) = body {
            return f.call(args, stack)
        };

        // fetch the calling frame from the stack
        let Some(mut calling_frame) = stack.frame(-1) else {
            unreachable!();
        };

        // evaluate args in the calling frame environment
        let R::List(args) = calling_frame.eval_list_lazy(args)? else {
            unreachable!();
        };

        // match arguments against function signature
        let (args, ellipsis) = match_args(formals.clone(), args, stack);

        // add closures to local scope
        stack.last_frame().env.insert("...".to_string(), R::List(ellipsis));
        stack.last_frame().env.append(R::List(args));

        // evaluate body in local scope
        stack.eval(body.clone())
    }
}

#[cfg(test)]
mod test {
    use crate::assert_r_eq;

    #[test]
    fn calls_find_symbols_in_parent_envs() {
        assert_r_eq!(
            R{ f <- function(a) { a + b }; b <- 3; f(2) }, 
            R{ 5 }
        );

        assert_r_eq!(
            R{ x <- function(a) { a + b }; b <- 3; y <- function(c, b) { x(c) * 2 + b }; y(10, 100) }, 
            R{ 126 }
        );
    }

    #[test]
    fn lazy_argument_evaluation() {
        assert_r_eq!(
            R{ f <- function(a, b = a) { b }; f(3) }, 
            R{ 3 }
        );

        assert_r_eq!(
            R{ f <- function(a, b = a) { b }; f(a = 3) }, 
            R{ 3 }
        );
    }
}
