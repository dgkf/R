/// This example is intended to be used with
/// 
/// ```
/// # cargo install cargo-expand
/// cargo expand --example builtin
/// ```
///
/// Expect errors due to missing traits in the current scope, though the output
/// can still be used to check macro expansion

use r_derive::*;

#[derive(Debug, Clone, CallableClone)]
#[builtin(sym = "(^.^)", kind = Infix)]
pub struct PrimitiveTest;

fn main() {}
