use r::repl::repl;

fn main() -> rustyline::Result<()> {
    let history = "history.txt".to_string();
    repl(Some(&history))
}
