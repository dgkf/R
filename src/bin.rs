use r::cli::Cli;
use r::lang::Signal;
use r::repl::repl;

#[cfg(feature = "wasm")]
fn main() {}

#[cfg(not(feature = "wasm"))]
fn main() -> Result<(), Signal> {
    use clap::Parser;
    let cli = Cli::parse();
    let history = "/tmp/history.txt".to_string();
    repl(cli.locale, Some(&history), cli.warranty)
}
