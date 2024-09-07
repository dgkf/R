// run using `cargo run --example parsing`

use r::parser::*;
use r::r_parse;
use r::session::SessionParserConfig;

fn main() {
    // Define how we want to parse.
    //
    // This will configure things like experiments that should enable
    // non-standard parsing rules or localization settings to determine which
    // localized keywords to use.
    //
    // Defaults to no experiments enabled and parsing English language keywords.
    //
    let parser = SessionParserConfig::default();

    let ast = parser.parse_input(
        "
          x <- 3  
          f <- function(a, b, c) {
            a + b + c
          }
        ",
    );

    println!(
        "Example 1:\nparsed syntax tree using `SessionParserConfig::parse_input()`:\n\n{ast:?}\n\n"
    );

    // for valid rust-parsible syntax:
    // r_parse! { <expr> }
    // can be used with naked expressions
    let ast = r_parse! {
        1 + 2 + 3
    };

    println!(
        "Example 2:\nfor simple rust-valid syntax we can use `r_parse! {{ <expr> }}`:\n\n{ast:?}\n\n"
    );

    // when we need to allow non-rust-parsible syntax:
    // r_parse! {{ "<expr>" }}
    // can be used to indicate that the expression is in a string
    let ast = r_parse! {{
        "
          x <- 3  
          f <- function(a, b, c) {
            a + b + c
          }
        "
    }};

    println!(
        "Example 3:\nthe same parse tree, using `r_parse! {{{{ \"<expr>\" }}}}`:\n\n{ast:?}\n\n"
    );
}
