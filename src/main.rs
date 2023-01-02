use std::io::{self, stdout, BufRead, Write};

use r::lang::Environment;
use r::parser::parse;
use r::utils::eval;

fn main() -> io::Result<()> {
    let mut stdin = io::stdin().lock().lines();
    let mut global_env = Environment::default();

    // startup message
    println!("R version 0.0.1 -- \"Why Not?\"");

    loop {
        print!("> ");
        stdout().flush().unwrap();

        let line = stdin.next().unwrap()?;
        let parse_res = parse(&line);

        match parse_res {
            Ok(expr) => {
                // println!("{:?}", expr);
                let res = eval(expr, &mut global_env);
                match res {
                    Ok(val) => println!("{}", val),
                    Err(e) => println!("{}", e),
                }
            }
            Err(e) => eprintln!("{}", e),
        }
    }
}
