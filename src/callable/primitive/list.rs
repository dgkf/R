use r_derive::*;

use crate::callable::core::*;
use crate::context::Context;
use crate::formals;
use crate::lang::*;
use crate::object::*;

/// Construct a `list`
///
/// A central data structure constructor for a `list`, a heterogeneous
/// collection of objects. A `list` may have names, corresponding
/// to each element. Constructed using `list()`, or the syntactic
/// shorthand, `(,)`.
///
/// # In-Language
///
/// ## Usage
///
/// ```custom,{class=r}
/// list(...)
/// (...,)
/// ```
///
/// ## Arguments
///
/// `...`: Arguments to collect into a `list`.
///
/// ## Differences to the R implementation
///
/// Setting a list value to `null` does not remove the element but
/// sets it to the value `null`.
///
/// ## Examples
///
/// ```custom,{class=r-repl}
/// list(one = 1, two = "two", three = 3.0)
/// ```
///
/// or using the syntactic sugar, `( ,)`
///
/// ```custom,{class=r-repl}
/// (1, "two", 3.0)
/// ```
///
/// To construct a `list` with only one element, a trailing comma is
/// required to disambiguate it from parentheses used for establishing
/// an order of operations.
///
/// ```custom,{class=r-repl}
/// (1,)
/// ```
#[doc(alias = "list")]
#[builtin(sym = "list")]
#[derive(Debug, Clone, PartialEq)]
pub struct PrimitiveList;

formals!(PrimitiveList, "(...)");

impl Callable for PrimitiveList {
    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        stack.eval_list_eager(args)
    }

    fn call_assign(&self, _value: Expr, _args: ExprList, _stack: &mut CallStack) -> EvalResult {
        // unpacking here!
        todo!();
    }
}
