#[macro_export]
macro_rules! r {
    // evaluate a single token directly
    {{ $expr:tt }} => {{
        // test if token is a string literal and evaluate directly
        if let Some(s) = (&$expr as &dyn std::any::Any).downcast_ref::<&str>() {
            crate::repl::eval(&s)

        // otherwise stringify token before evaluating
        } else {
            let expr = stringify!($expr);
            crate::repl::eval(expr)           
        }
    }};

    // evaluate a token stream by first stringifying. This can affect whitespace,
    // so consider using `r!{{" .. "}}` syntax when whitespace is meaningful
    { $($expr:tt)+ } => {{
        let expr = stringify!($($expr)+);
        crate::repl::eval(expr)
    }};
}
