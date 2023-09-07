use super::modes::AsMinimallyNumeric;
use super::atomic::AtomicMode;

impl AsMinimallyNumeric for String { type As = f64; }
impl AtomicMode for String { 
    fn is_character() -> bool { 
        true 
    } 
}
