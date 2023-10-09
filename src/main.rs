use r::lang::Signal;
use r::repl::repl;

fn main() -> Result<(), Signal> {
    let history = "/tmp/history.txt".to_string();
    repl(Some(&history))
}
