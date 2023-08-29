mod release;

#[cfg(feature = "repl")]
mod repl;

#[cfg(feature = "repl")]
pub use repl::*;

#[cfg(feature = "repl")]
pub mod highlight;

#[cfg(feature = "repl")]
pub mod prompt;

#[cfg(feature = "repl")]
pub mod validator;

#[cfg(feature = "wasm")]
pub mod headless;
