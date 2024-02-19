use clap::Parser;
use r::lang::Signal;
use r::parser::Localization;
use r::repl::repl;

#[derive(Parser, Debug)]
struct Cli {
    #[arg(short, long, default_value_t = Localization::En)]
    locale: Localization,

    #[arg(long)]
    warranty: bool,
}

fn main() -> Result<(), Signal> {
    let cli = Cli::parse();
    let history = "/tmp/history.txt".to_string();
    repl(cli.locale, Some(&history), cli.warranty)
}
