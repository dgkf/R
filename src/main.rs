use r::r_repl::repl::repl;

fn main() -> rustyline::Result<()> {
    let history = "/tmp/history.txt".to_string();
    repl(Some(&history))
}
