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
macro_rules! r_macro_stringify {
    // evaluate a single token directly
    {{ $expr:literal }} => {{
        {
            // test if token is a string literal and evaluate directly
            if let Some(s) = (&$expr as &dyn std::any::Any).downcast_ref::<&str>() {
                s

            // otherwise stringify token before evaluating
            } else {
                stringify!($expr)
            }
        }
    }};

    // evaluate a token stream by first stringifying. This can affect whitespace,
    // so consider using `r!{{" .. "}}` syntax when whitespace is meaningful
    { $($expr:tt)+ } => {{
        {
            stringify!($($expr)+)
        }
    }};
}

#[macro_export]
macro_rules! formals {
    ( $what:ident, $args:literal ) => {
        impl CallableFormals for $what {
            fn formals(&self) -> ExprList {
                static FORMALS: std::sync::LazyLock<ExprList> = std::sync::LazyLock::new(|| {
                    use $crate::object::{Expr, ExprList};

                    let signature = $crate::r_parse! {{ $args }};
                    let Ok($crate::object::Expr::Call(_, signature)) = signature else {
                        panic!("unexpected formal definition")
                    };

                    signature
                        .into_iter()
                        .map(|pair| match pair {
                            (None, Expr::Symbol(name)) => (Some(name), Expr::Missing),
                            pair => pair,
                        })
                        .collect::<ExprList>()
                });

                (*FORMALS).clone()
            }
        }
    };
}

#[macro_export]
macro_rules! r_parse {
    { $($expr:tt)+ } => {{
        let expr = $crate::r_macro_stringify!($($expr)+);
        $crate::lang::CallStack::default().parse(expr)
    }};
}

#[macro_export]
macro_rules! r {
    { $($expr:tt)+ } => {{
        use $crate::context::Context;
        ($crate::r_parse! { $($expr)+ })
            .and_then(|x| $crate::lang::CallStack::default().eval_and_finalize(x))

    }};
}

#[macro_export]
macro_rules! r_expect {
    { $($expr:tt)+ } => {{
        let res = $crate::r! { $($expr)+ };
        assert_eq!(res, r! { true })
    }};
}
