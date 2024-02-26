use std::sync::OnceLock;

pub fn use_tail_calls(x: Option<bool>) -> bool {
    static USE_TAIL_CALLS: OnceLock<bool> = OnceLock::new();
    *USE_TAIL_CALLS.get_or_init(|| x.unwrap_or_default())
}

pub fn use_rest_args(x: Option<bool>) -> bool {
    static USE_REST_ARGS: OnceLock<bool> = OnceLock::new();
    *USE_REST_ARGS.get_or_init(|| x.unwrap_or_default())
}
