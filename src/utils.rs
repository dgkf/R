#[macro_export]
macro_rules! assert_r_eq {
    (R{ $($l:tt)+ }, R{ $($r:tt)+ }) => {{
        let l = stringify!($($l)+);
        let r = stringify!($($r)+);
        assert_eq! (crate::repl::eval(l), crate::repl::eval(r));
    }};
    (R{ $($l:tt)+ }, $r:expr) => {{ {
        let l = stringify!($($l)+);
        assert_eq! (crate::repl::eval(l), $r);
    } }};
    ($l:expr, R{ $($r:tt)+ }) => {{ {
        let r = stringify!($($r)+);
        assert_eq! ($l, crate::repl::eval(r));
    } }};
}
