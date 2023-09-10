extern crate r_derive;

use crate::ast::*;
use crate::callable::dyncompare::*;
use crate::callable::builtins::BUILTIN;
use crate::lang::*;

pub fn match_args(mut formals: ExprList, mut args: List, stack: &CallStack) -> (List, List) {
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

impl std::fmt::Debug for Box<dyn Builtin> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

impl Clone for Box<dyn Builtin> {
    fn clone(&self) -> Box<dyn Builtin> {
        self.callable_clone()
    }
}

pub trait CallableClone: Callable {
    fn callable_clone(&self) -> Box<dyn Builtin>;
}

pub trait Callable {
    fn formals(&self) -> ExprList {
        ExprList::new()
    }

    fn match_args(&self, args: ExprList, stack: &mut CallStack) -> Result<(List, List), RSignal> {
        let mut formals = self.formals();
        let mut ellipsis: Vec<(Option<String>, R)> = vec![];
        let mut matched_args: Vec<(Option<String>, R)> = vec![];

        // extract iterable list of arguments
        let mut args: List = stack.parent_frame().eval_list_lazy(args)?.try_into()?;

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

        Ok((matched_args, ellipsis))
    }

    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (args, ellipsis) = self.match_args(args, stack)?;
        self.call_matched(R::List(args), R::List(ellipsis), stack)
    }

    fn call_matched(&self, mut _args: R, mut _ellipsis: R, _stack: &mut CallStack) -> EvalResult {
        unimplemented!()
    }

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

pub trait Builtin: Callable + CallableClone + Format + DynCompare + Sync {}

pub trait Sym {
    const SYM: &'static str;
    const KIND: &'static SymKind;
}

pub enum SymKind {
    Function,
    Infix,
    Prefix,
    PostfixCall(&'static str, &'static str)
}

impl PartialEq<dyn Builtin> for dyn Builtin {
    fn eq(&self, other: &dyn Builtin) -> bool {
        self.as_dyn_compare() == other.as_dyn_compare()
    }
}

impl<T> Format for T
where
    T: Sym,
{
    fn rfmt_call_with(&self, _state: FormatState, args: &ExprList) -> String {
        use SymKind::*;
        let sym = Self::SYM;
        match Self::KIND {
            Function => format!("{sym}({})", args),
            Infix => format!("{} {sym} {}", args.values[0], args.values[1]),
            Prefix => format!("{sym}{}", args.values[0]),
            PostfixCall(l, r) => format!("{sym}{l}{}{r}", args),
        }
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
    fn callable_clone(&self) -> Box<dyn Builtin> {
        Box::new(self.clone())
    }
}

impl Builtin for String {}

pub fn builtin(s: &str) -> Result<Box<dyn Builtin>, ()> {
    <Box<dyn Builtin>>::try_from(s)
}

impl TryFrom<&str> for Box<dyn Builtin> {
    type Error = ();
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        BUILTIN.get(s).map_or(Err(()), |b| Ok(b.clone()))
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
    use crate::r;

    #[test]
    fn calls_find_symbols_in_parent_envs() {
        assert_eq!(
            r!{ f <- function(a) { a + b }; b <- 3; f(2) }, 
            r!{ 5 }
        );

        assert_eq!(
            r!{{"
                x <- function(a) { 
                    a + b 
                }

                b <- 3

                y <- function(c, b) { 
                    x(c) * 2 + b 
                }

                y(10, 100)
            "}},
            r!{ 126 }
        );
    }

    #[test]
    fn lazy_argument_evaluation() {
        assert_eq!(
            r!{ f <- function(a, b = a) { b }; f(3) }, 
            r!{ 3 }
        );

        assert_eq!(
            r!{ f <- function(a, b = a) { b }; f(a = 3) }, 
            r!{ 3 }
        );
    }
}