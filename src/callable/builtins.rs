/// Do not edit directly!
/// 
/// The contents of this file are built by build.rs
///

use std::collections::HashMap;

use::lazy_static::lazy_static;

use crate::callable::core::Builtin;
use crate::callable::primitive::*;
use crate::callable::operators::*;

lazy_static! {
    pub static ref BUILTIN: HashMap<&'static str, Box<dyn Builtin>> = {
        HashMap::from([
            // automatically populated on build. see build.rs // builtins start
            ("<-", Box::new(InfixAssign) as Box<dyn Builtin>),
            ("+", Box::new(InfixAdd) as Box<dyn Builtin>),
            ("-", Box::new(InfixSub) as Box<dyn Builtin>),
            ("-", Box::new(PrefixSub) as Box<dyn Builtin>),
            ("*", Box::new(InfixMul) as Box<dyn Builtin>),
            ("/", Box::new(InfixDiv) as Box<dyn Builtin>),
            ("^", Box::new(InfixPow) as Box<dyn Builtin>),
            ("%", Box::new(InfixMod) as Box<dyn Builtin>),
            ("||", Box::new(InfixOr) as Box<dyn Builtin>),
            ("&&", Box::new(InfixAnd) as Box<dyn Builtin>),
            ("|", Box::new(InfixVectorOr) as Box<dyn Builtin>),
            ("&", Box::new(InfixVectorAnd) as Box<dyn Builtin>),
            (">", Box::new(InfixGreater) as Box<dyn Builtin>),
            (">=", Box::new(InfixGreaterEqual) as Box<dyn Builtin>),
            ("<", Box::new(InfixLess) as Box<dyn Builtin>),
            ("<=", Box::new(InfixLessEqual) as Box<dyn Builtin>),
            ("==", Box::new(InfixEqual) as Box<dyn Builtin>),
            ("!=", Box::new(InfixNotEqual) as Box<dyn Builtin>),
            ("|>", Box::new(InfixPipe) as Box<dyn Builtin>),
            (":", Box::new(InfixColon) as Box<dyn Builtin>),
            ("[[", Box::new(PostfixIndex) as Box<dyn Builtin>),
            ("[", Box::new(PostfixVecIndex) as Box<dyn Builtin>),
            ("q", Box::new(PrimitiveQ) as Box<dyn Builtin>),
            ("c", Box::new(PrimitiveC) as Box<dyn Builtin>),
            ("callstack", Box::new(PrimitiveCallstack) as Box<dyn Builtin>),
            ("ls", Box::new(PrimitiveLs) as Box<dyn Builtin>),
            ("rnorm", Box::new(PrimitiveRnorm) as Box<dyn Builtin>),
            ("list", Box::new(PrimitiveList) as Box<dyn Builtin>),
            ("runif", Box::new(PrimitiveRunif) as Box<dyn Builtin>),
            ("paste", Box::new(PrimitivePaste) as Box<dyn Builtin>),
            // builtins end
        ])
    };
}