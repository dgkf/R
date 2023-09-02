pub const GIT_HASH: &'static str = env!("GIT_HASH");

pub fn session_header() -> String {
    let dev = if GIT_HASH.len() > 0 {
        format!(" (dev {:.8})", GIT_HASH)
    } else {
        String::from("")
    };

    format!("R version 0.2.0 -- \"In Bloom\"{dev}")
}
