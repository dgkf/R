use r_derive::Primitive;
use rand::distributions::{Distribution, Uniform};

use crate::ast::*;
use crate::error::RError;
use crate::lang::*;
use crate::callable::core::*;
use crate::vector::vectors::{Vector, OptionNA};

#[derive(Debug, Clone, Primitive, PartialEq)]
pub struct PrimitiveRunif;

impl PrimitiveSYM for PrimitiveRunif {
    const SYM: &'static str = "runif";
}

impl Callable for PrimitiveRunif {
    fn formals(&self) -> ExprList {
        ExprList::from(vec![
            (Some(String::from("n")),   Expr::Null),
            (Some(String::from("min")), Expr::Number(0.0)),
            (Some(String::from("max")), Expr::Number(1.0)),
        ])
    }

    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        let R::List(vals) = stack.parent_frame().eval_list_lazy(args)? else {
            unreachable!()
        };

        let (vals, _) = match_args(self.formals(), vals, &stack.env());
        let vals = force_closures(vals, stack);
        let mut vals = R::List(vals);

        let Some(n) = vals.get_named("n") else {
            return Err(RSignal::Error(RError::ArgumentMissing(String::from("n"))));
        };

        let R::Vector(Vector::Integer(n)) = n.as_integer()? else {
            return Err(RSignal::Error(RError::CannotBeCoercedToNumeric));
        };

        let Some(min) = vals.get_named("min") else {
            return Err(RSignal::Error(RError::ArgumentMissing(String::from("min"))));
        };

        let R::Vector(Vector::Numeric(min)) = min.as_numeric()? else {
            return Err(RSignal::Error(RError::CannotBeCoercedToNumeric));
        };

        let Some(max) = vals.get_named("max") else {
            return Err(RSignal::Error(RError::ArgumentMissing(String::from("max"))));
        };

        let R::Vector(Vector::Numeric(max)) = max.as_numeric()? else {
            return Err(RSignal::Error(RError::CannotBeCoercedToNumeric));
        };

        let Some(&OptionNA::Some(n)) = n.get(0) else {
            todo!();
        };

        let Some(&OptionNA::Some(min)) = min.get(0) else {
            todo!();
        };

        let Some(&OptionNA::Some(max)) = max.get(0) else {
            todo!();
        };

        let between = Uniform::try_from(min..=max).unwrap();
        let mut rng = rand::thread_rng();
        Ok(R::Vector(Vector::from((1..=n).into_iter().map(|_| between.sample(&mut rng)).collect::<Vec<f64>>())))
    }
}