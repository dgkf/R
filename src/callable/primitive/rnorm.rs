use r_derive::builtin;
use rand_distr::{Distribution, Normal};

use crate::callable::core::*;
use crate::error::RError;
use crate::lang::*;
use crate::object::*;

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "rnorm")]
pub struct PrimitiveRnorm;
impl Callable for PrimitiveRnorm {
    fn formals(&self) -> ExprList {
        ExprList::from(vec![
            (Some(String::from("n")), Expr::Null),
            (Some(String::from("mean")), Expr::Number(0.0)),
            (Some(String::from("std")), Expr::Number(1.0)),
        ])
    }

    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        use RError::ArgumentInvalid;
        let (vals, _) = self.match_arg_exprs(args, stack)?;
        let vals = force_closures(vals, stack);
        let mut vals = Obj::List(List::from(vals));

        let n: i32 = vals.try_get_named("n")?.try_into()?;
        let mean: Vec<f64> = vals.try_get_named("mean")?.try_into()?;
        let std: Vec<f64> = vals.try_get_named("std")?.try_into()?;
        let len = std::cmp::max(mean.len(), std.len());
        let mut rng = rand::thread_rng();

        // TODO: perhaps these branches can be unified by creating a
        // run-length-encoding of the iterator?

        // special case when both min and max are length 1, sampling
        // all random numbers at once from the same distribution
        if len == 1 {
            let mean = mean
                .first()
                .map_or(ArgumentInvalid(String::from("mean")).into(), Ok)?;
            let std = std
                .first()
                .map_or(ArgumentInvalid(String::from("std")).into(), Ok)?;

            let normal = Normal::new(*mean, *std).unwrap();

            Ok(Obj::Vector(Vector::from(
                (1..=n)
                    .map(|_| normal.sample(&mut rng))
                    .collect::<Vec<f64>>(),
            )))

        // otherwise we need to zip through mins and maxes to get distributions
        } else {
            Ok(Obj::Vector(Vector::from(
                mean.into_iter()
                    .cycle()
                    .zip(std.into_iter().cycle())
                    .take(len)
                    .map(|(mean, std)| Normal::new(mean, std).unwrap().sample(&mut rng))
                    .collect::<Vec<f64>>(),
            )))
        }
    }
}
