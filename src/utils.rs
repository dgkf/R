pub trait SameType<T>: Sized {
    fn is_same_type_as(&self, _other: &T) -> bool {
        false
    }
}

impl<T, U> SameType<T> for U {
    fn is_same_type_as(&self, _other: &T) -> bool {
        true
    }
}

#[macro_export]
macro_rules! r {
    // evaluate a single token directly
    {{ $expr:literal }} => {{
        {
            // test if token is a string literal and evaluate directly
            if let Some(s) = (&$expr as &dyn std::any::Any).downcast_ref::<&str>() {
                $crate::lang::CallStack::default().parse_and_eval(s)

            // otherwise stringify token before evaluating
            } else {
                let expr = stringify!($expr);
                $crate::lang::CallStack::default().parse_and_eval(expr)
            }
        }
    }};

    // evaluate a token stream by first stringifying. This can affect whitespace,
    // so consider using `r!{{" .. "}}` syntax when whitespace is meaningful
    { $($expr:tt)+ } => {{
        {
            let expr = stringify!($($expr)+);
            $crate::lang::CallStack::default().parse_and_eval(expr)
        }
    }};
}

#[macro_export]
macro_rules! r_expect {
    // evaluate a single token directly
    {{ $expr:tt }} => {{
        {
            // test if token is a string literal and evaluate directly
            if let Some(s) = (&$expr as &dyn std::any::Any).downcast_ref::<&str>() {
                let res  = $crate::lang::CallStack::default().parse_and_eval(s);
                assert_eq!(res, r!{ true })

            // otherwise stringify token before evaluating
            } else {
                let expr = stringify!($expr);
                let res  = $crate::lang::CallStack::default().parse_and_eval(expr);
                assert_eq!(res, r!{ true })
            }
        }
    }};

    // evaluate a token stream by first stringifying. This can affect whitespace,
    // so consider using `r!{{" .. "}}` syntax when whitespace is meaningful
    { $($expr:tt)+ } => {{
        {
            let expr = stringify!($($expr)+);
            let res  = $crate::lang::CallStack::default().parse_and_eval(expr);
            assert_eq!(res, r!{ true })
        }
    }};
}
