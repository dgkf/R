use std::collections::HashMap;

use::lazy_static::lazy_static;

use crate::callable::core::Primitive;
use crate::callable::primitive::*;

lazy_static! {
    static ref BUILTIN: HashMap<&'static str, Box<dyn Primitive>> = {
        HashMap::from([
            //// register_builtin start 
            ("paste", Box::new(PrimitivePaste) as Box<dyn Primitive>)
            //// register_builtin end
        ])
    };
}