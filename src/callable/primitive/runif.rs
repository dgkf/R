use r_derive::builtin;
use rand::distributions::{Distribution, Uniform};
use rand::Rng;

use crate::callable::core::*;
use crate::error::RError;
use crate::lang::*;
use crate::object::*;

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "runif")]
pub struct PrimitiveRunif;
impl Callable for PrimitiveRunif {
    fn formals(&self) -> ExprList {
        ExprList::from(vec![
            (Some(String::from("n")), Expr::Null),
            (Some(String::from("min")), Expr::Number(0.0)),
            (Some(String::from("max")), Expr::Number(1.0)),
        ])
    }

    fn call(&self, args: ExprList, stack: &mut CallStack) -> EvalResult {
        use RError::ArgumentInvalid;

        let (vals, _) = self.match_arg_exprs(args, stack)?;
        let vals = force_closures(vals, stack);
        let mut vals = Obj::List(List::from(vals));

        let n: i32 = vals.try_get_named("n")?.try_into()?;
        let min: Vec<f64> = vals.try_get_named("min")?.try_into()?;
        let max: Vec<f64> = vals.try_get_named("max")?.try_into()?;
        let len = std::cmp::max(min.len(), max.len());
        let mut rng = rand::thread_rng();

        // TODO: perhaps these branches can be unified by creating a
        // run-length-encoding of the iterator?

        // special case when both min and max are length 1, sampling
        // all random numbers at once from the same distribution
        if len == 1 {
            let min = min
                .first()
                .map_or(ArgumentInvalid(String::from("min")).into(), Ok)?;
            let max = max
                .first()
                .map_or(ArgumentInvalid(String::from("max")).into(), Ok)?;
            let between = Uniform::try_from(*min..=*max).unwrap();

            Ok(Obj::Vector(Vector::from(
                (1..=n)
                    .map(|_| between.sample(&mut rng))
                    .collect::<Vec<f64>>(),
            )))

        // otherwise we need to zip through mins and maxes to get distributions
        } else {
            Ok(Obj::Vector(Vector::from(
                min.into_iter()
                    .cycle()
                    .zip(max.into_iter().cycle())
                    .take(len)
                    .map(|(min, max)| rng.gen_range(min..=max))
                    .collect::<Vec<f64>>(),
            )))
        }
    }
}
