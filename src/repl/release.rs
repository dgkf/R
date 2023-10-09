pub const GIT_HASH: &str = env!("GIT_HASH");

pub fn session_header() -> String {
    let dev = if !GIT_HASH.is_empty() {
        format!(" (dev {:.8})", GIT_HASH)
    } else {
        String::from("")
    };

    format!("R version 0.3.1 -- \"Art Smock\"{dev}")
}
