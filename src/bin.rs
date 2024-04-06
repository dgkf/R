use r::cli::Cli;
use r::lang::Signal;
use r::repl::repl;
use r::session::Session;

#[cfg(target_family = "wasm")]
fn main() {}

#[cfg(not(target_family = "wasm"))]
fn main() -> Result<(), Signal> {
    use clap::Parser;
    let session: Session = Session::from(Cli::parse()).with_history_file(
        std::env::temp_dir()
            .join("history.txt")
            .into_os_string()
            .into_string()
            .unwrap_or_default(),
    );

    repl(session)
}
