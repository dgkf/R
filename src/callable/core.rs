extern crate r_derive;

use crate::callable::dyncompare::*;
use crate::cli::Experiment;
use crate::context::Context;
use crate::object::types::{Character, Integer};
use crate::object::{Expr, ExprList, List, Obj, Subset};
use crate::{internal_err, lang::*};

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

pub trait CallableFormals {
    fn formals(&self) -> ExprList {
        ExprList::default()
    }
}

pub trait Callable: CallableFormals {
    fn match_args(&self, args: List, stack: &mut CallStack) -> Result<(List, List), Signal> {
        let mut formals = self.formals();
        let ellipsis: List = List::new();
        let matched_args: List = List::new();

        // assign named args to corresponding formals

        let mut indices: Vec<i32> = Vec::new();

        for (i, (maybe_name, value)) in args.pairs_ref().iter().enumerate() {
            if let Character::Some(name) = maybe_name {
                if let Some((Some(_), _)) = formals.remove_named(name) {
                    matched_args.push_named(Character::Some(name.clone()), value.clone());
                    continue;
                }
            }
            indices.push(i as i32);
        }

        let indices: Vec<Integer> = indices.into_iter().map(Integer::Some).collect();
        let subset = Subset::Indices(indices.into());
        let args = args.subset(subset).materialize();

        // TODO(bug): need to evaluate trailing unassigned params that have
        // a default value before popping off remaining trailing params

        // remove any Ellipsis param, and any trailing unassigned params
        let remainder = formals.pop_trailing();

        // backfill unnamed args, populating ellipsis with overflow
        for (key, value) in args.iter_pairs() {
            match key {
                // named args go directly to ellipsis, they did not match a formal
                Character::Some(arg) => ellipsis.push_named(Character::Some(arg), value),

                // unnamed args populate next formal, or ellipsis if formals exhausted
                Character::NA => {
                    let next_unassigned_formal = formals.remove(0);
                    if let Some((Some(param), _)) = next_unassigned_formal {
                        matched_args.push_named(Character::Some(param), value);
                    } else {
                        ellipsis.push_named(Character::NA, value);
                    }
                }
            }
        }

        // add back in parameter defaults that weren't filled with args
        for (param, default) in formals.into_iter() {
            matched_args.push_named(
                param.into(),
                Obj::Promise(None, default, stack.last_frame().env().clone()),
            )
        }

        if let Some(Expr::Ellipsis(Some(name))) = remainder.get(0) {
            matched_args.push_named(Character::Some(name), Obj::List(ellipsis.clone()));
        } else if !remainder.is_empty() {
            matched_args.push_named(
                Character::Some("...".to_string()),
                Obj::List(ellipsis.clone()),
            );
        }

        Ok((matched_args, ellipsis))
    }

    fn match_arg_exprs(
        &self,
        args: ExprList,
        stack: &mut CallStack,
    ) -> Result<(List, List), Signal> {
        let args: List = stack.parent_frame().eval_list_lazy(args)?.try_into()?;
        let args = args.dedup_last();
        self.match_args(args, stack)
    }

    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let (args, ellipsis) = self.match_arg_exprs(args, stack)?;
        self.call_matched(args, ellipsis, stack)
    }

    fn call_mut(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        self.call(args, stack)
    }

    fn call_matched(
        &self,
        mut _args: List,
        mut _ellipsis: List,
        _stack: &mut CallStack,
    ) -> EvalResult {
        unimplemented!()
    }

    fn call_assign(&self, value: Expr, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let what = self.call_mut(args, stack)?;
        let value = stack.eval(value)?;
        what.assign(value)
    }
}

#[derive(Default)]
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

pub trait Builtin: Callable + CallableClone + Format + DynCompare + Sync + Send {
    fn is_transparent(&self) -> bool {
        false
    }

    fn is_infix(&self) -> bool {
        self.kind() == SymKind::Infix
    }

    fn kind(&self) -> SymKind {
        SymKind::Function
    }
}

pub trait Sym {
    const SYM: &'static str;
    const KIND: &'static SymKind;
}

#[derive(PartialEq)]
pub enum SymKind {
    Keyword,
    Function,
    Infix,
    Prefix,
    Postfix,
    PostfixCall(&'static str, &'static str),
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
            Postfix => format!("{}{sym}", args.values[0]),
            PostfixCall(l, r) => {
                let mut args = args.clone().into_iter();
                let first = args.next().unwrap_or((None, Expr::Null));
                let rest = args.collect::<ExprList>();
                format!("{}{l}{}{r}", first.1, rest)
            }
            Keyword => sym.to_string(), // keywords generally implement their own formatter
        }
    }

    fn rfmt_with(&self, _: FormatState) -> String {
        Self::SYM.to_string()
    }
}

// Allow Strings to be used via Expr::Primitive(String)

impl CallableClone for String
where
    Self: Callable,
{
    fn callable_clone(&self) -> Box<dyn Builtin> {
        Box::new(self.clone())
    }
}

impl CallableFormals for String {}
impl Builtin for String {}

pub fn force_promises(vals: List, stack: &mut CallStack) -> Result<Vec<(Character, Obj)>, Signal> {
    // Force any closures that were created during call. This helps with using
    // variables as argument for sep and collapse parameters.
    vals.iter_pairs()
        .map(|(k, v)| match (k, v.force(stack)) {
            (k, Ok(v)) => Ok((k, v)),
            (_, Err(e)) => Err(e),
        })
        .collect()
}

impl Format for String {
    fn rfmt_call_with(&self, _state: FormatState, args: &ExprList) -> String {
        format!("{}({})", self, args)
    }
}

impl Callable for String {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        (stack.last_frame().env().clone().get(self.clone())?).call(args, stack)
    }
}

impl Format for Obj {}

impl CallableFormals for Obj {
    fn formals(&self) -> ExprList {
        match self {
            Obj::Function(formals, _, _) => formals.clone(),
            _ => ExprList::new(),
        }
    }
}

impl Callable for Obj {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let Obj::Function(_, body, _) = self else {
            return internal_err!();
        };

        // body is a primitive, call directly
        if let Expr::Primitive(f) = body {
            return f.call(args, stack);
        };

        // match arguments against function signature
        let (args, ellipsis) = self.match_arg_exprs(args, stack)?;
        self.call_matched(args, ellipsis, stack)
    }

    fn call_matched(&self, args: List, ellipsis: List, stack: &mut CallStack) -> EvalResult {
        let Obj::Function(_, body, _) = self else {
            return internal_err!();
        };

        if !stack.session.experiments.contains(&Experiment::RestArgs) {
            stack.env().insert("...".to_string(), Obj::List(ellipsis));
        }

        stack.env().append(args);
        stack.eval(body.clone())
    }
}

#[cfg(test)]
mod test {
    use crate::r;

    #[test]
    fn recursion_works_as_expected() {
        assert_eq!(r! { f <- function(a) { a + b }; b <- 3; f(2) }, r! { 5 });

        assert_eq!(
            r! {{"
                x <- function(n) {
                    if (n > 0) x(n - 1) else 'done'
                }

                x(10)
            "}},
            r! { "done" }
        );
    }

    #[test]
    fn calls_find_symbols_in_parent_envs() {
        assert_eq!(r! { f <- function(a) { a + b }; b <- 3; f(2) }, r! { 5 });

        assert_eq!(
            r! {{"
                x <- function(a) {
                    a + b
                }

                b <- 3

                y <- function(c, b) {
                    x(c) * 2 + b
                }

                y(10, 100)
            "}},
            r! { 126 }
        );
    }

    #[test]
    fn calls_appropriately_scope_parameter_defaults() {
        assert_eq!(
            r! {{"
                f <- function(x = a) {
                    a <- 3
                    x
                }
                f(10)
            "}},
            r! { 10 }
        );

        assert_eq!(
            r! {{"
                f <- function(x = a) {
                    a <- 3
                    x
                }
                f()
            "}},
            r! { 3 }
        );
    }

    #[test]
    fn lazy_argument_evaluation() {
        assert_eq!(r! { f <- function(a, b = a) { b }; f(3) }, r! { 3 });

        assert_eq!(r! { f <- function(a, b = a) { b }; f(a = 3) }, r! { 3 });
    }
}
