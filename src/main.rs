use r::r_repl::repl::repl;

fn main() -> Result<(), ()> {
    let history = "/tmp/history.txt".to_string();
    repl(Some(&history))
}
