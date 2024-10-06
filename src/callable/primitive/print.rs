use r_derive::*;
use std::io::Write;

use crate::callable::core::*;
use crate::formals;
use crate::lang::*;
use crate::object::*;

/// Print to the Console
///
/// # In-Language
///
/// ## Usage
///
/// ```custom,{class=r}
/// print(x, ...)
/// ```
///
/// ## Arguments
///
/// * `x`: An object to print.
///
/// ## Examples
///
/// ```custom,{class=r-repl}
/// print("Hello, World!")
/// ```
///
#[doc(alias = "print")]
#[builtin(sym = "print")]
#[derive(Debug, Clone, PartialEq)]
pub struct PrimitivePrint;

formals!(PrimitivePrint, "(x, ...)");

impl Callable for PrimitivePrint {
    fn call_matched(&self, args: List, _ellipsis: List, stack: &mut CallStack) -> EvalResult {
        let mut args = Obj::List(args);
        let x = args.try_get_named("x")?.force(stack)?;
        writeln!(stack.session.output, "{x}").ok();
        Ok(x)
    }
}
