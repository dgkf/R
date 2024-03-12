use r::cli::Cli;
use r::lang::Signal;
use r::repl::repl;
use r::session::Session;

#[cfg(feature = "wasm")]
fn main() {}

#[cfg(not(feature = "wasm"))]
fn main() -> Result<(), Signal> {
    use clap::Parser;
    let session: Session = Cli::parse().into();
    repl(session)
}
