/// Do not edit directly!
///
/// The contents of this file are built by build.rs
///
use crate::callable::core::Builtin;
use crate::callable::operators::*;
use crate::callable::primitive::*;
use hashbrown::HashMap;
use std::sync::LazyLock;

#[rustfmt::skip]
pub static BUILTIN: LazyLock<HashMap<&'static str, Box<dyn Builtin>>> = LazyLock::new(|| {
    HashMap::from([
        // automatically populated on build. see build.rs // builtins start
        ("<-", Box::new(InfixAssign) as Box<dyn Builtin>),
        ("+", Box::new(InfixAdd) as Box<dyn Builtin>),
        ("-", Box::new(InfixSub) as Box<dyn Builtin>),
        ("-", Box::new(PrefixSub) as Box<dyn Builtin>),
        ("!", Box::new(PrefixNot) as Box<dyn Builtin>),
        ("..", Box::new(PrefixPack) as Box<dyn Builtin>),
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
        ("$", Box::new(InfixDollar) as Box<dyn Builtin>),
        ("..", Box::new(PostfixPack) as Box<dyn Builtin>),
        ("[[", Box::new(PostfixIndex) as Box<dyn Builtin>),
        ("[", Box::new(PostfixVecIndex) as Box<dyn Builtin>),
        ("c", Box::new(PrimitiveC) as Box<dyn Builtin>),
        ("callstack", Box::new(PrimitiveCallstack) as Box<dyn Builtin>),
        ("environment", Box::new(PrimitiveEnvironment) as Box<dyn Builtin>),
        ("eval", Box::new(PrimitiveEval) as Box<dyn Builtin>),
        ("is_null", Box::new(PrimitiveIsNull) as Box<dyn Builtin>),
        ("length", Box::new(PrimitiveLength) as Box<dyn Builtin>),
        ("list", Box::new(PrimitiveList) as Box<dyn Builtin>),
        ("names", Box::new(PrimitiveNames) as Box<dyn Builtin>),
        ("parent", Box::new(PrimitiveParent) as Box<dyn Builtin>),
        ("paste", Box::new(PrimitivePaste) as Box<dyn Builtin>),
        ("print", Box::new(PrimitivePrint) as Box<dyn Builtin>),
        ("q", Box::new(PrimitiveQ) as Box<dyn Builtin>),
        ("quote", Box::new(PrimitiveQuote) as Box<dyn Builtin>),
        ("rnorm", Box::new(PrimitiveRnorm) as Box<dyn Builtin>),
        ("runif", Box::new(PrimitiveRunif) as Box<dyn Builtin>),
        ("substitute", Box::new(PrimitiveSubstitute) as Box<dyn Builtin>),
        ("sum", Box::new(PrimitiveSum) as Box<dyn Builtin>),
        // builtins end
    ])
});
