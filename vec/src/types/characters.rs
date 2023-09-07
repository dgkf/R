use crate::atomic::AtomicMode;

// impl IntoNaAble for String { type Output = OptionNa<String>; }
// impl Integer for String { type As = OptionNa<i32>; }
impl AtomicMode for String { 
    fn is_character() -> bool { 
        true 
    } 
}

