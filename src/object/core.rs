use std::rc::Rc;

use crate::error::Error;
use crate::internal_err;
use crate::lang::Signal;

use super::*;

#[derive(Default, Clone, Debug)]
pub enum Obj {
    // Data structures
    #[default]
    Null,
    Vector(Vector),
    List(List),

    // Metaprogramming structures
    Expr(Expr),
    Promise(Option<Box<Obj>>, Expr, Rc<Environment>),
    Function(ExprList, Expr, Rc<Environment>),
    Environment(Rc<Environment>),
}

impl PartialEq for Obj {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Obj::Null, Obj::Null) => true,
            (Obj::List(l), Obj::List(r)) => l
                .pairs_ref()
                .iter()
                .zip(r.pairs_ref().iter())
                .all(|((lk, lv), (rk, rv))| lk == rk && lv == rv),
            (Obj::Expr(l), Obj::Expr(r)) => l == r,
            (Obj::Promise(None, lc, lenv), Obj::Promise(None, rc, renv)) => {
                lc == rc && lenv == renv
            }
            (Obj::Promise(Some(a), ..), Obj::Promise(Some(b), ..)) => a == b,
            (Obj::Promise(..), Obj::Promise(..)) => false,
            (Obj::Function(largs, lbody, lenv), Obj::Function(rargs, rbody, renv)) => {
                largs == rargs
                    && lbody == rbody
                    && Obj::Environment(lenv.clone()) == Obj::Environment(renv.clone())
            }
            (Obj::Environment(l), Obj::Environment(r)) => {
                l.values.as_ptr() == r.values.as_ptr()
                    && (match (&l.parent, &r.parent) {
                        (None, None) => true,
                        (Some(lp), Some(rp)) => {
                            Rc::<Environment>::as_ptr(lp) == Rc::<Environment>::as_ptr(rp)
                        }
                        _ => false,
                    })
            }
            (Obj::Vector(lv), Obj::Vector(rv)) => match (lv, rv) {
                (Vector::Double(l), Vector::Double(r)) => l == r,
                (Vector::Integer(l), Vector::Integer(r)) => l == r,
                (Vector::Logical(l), Vector::Logical(r)) => l == r,
                (Vector::Character(l), Vector::Character(r)) => l == r,
                _ => false,
            },
            _ => false,
        }
    }
}

impl TryInto<i32> for Obj {
    type Error = Signal;
    fn try_into(self) -> Result<i32, Self::Error> {
        use Error::CannotBeCoercedToInteger;

        let Obj::Vector(Vector::Integer(v)) = self.as_integer()? else {
            return internal_err!();
        };

        match v.inner().clone().borrow()[..] {
            [OptionNA::Some(i), ..] => Ok(i),
            _ => Err(CannotBeCoercedToInteger.into()),
        }
    }
}

impl<T> From<T> for Obj
where
    Vector: From<T>,
{
    fn from(value: T) -> Self {
        Obj::Vector(Vector::from(value))
    }
}

impl TryInto<f64> for Obj {
    type Error = Signal;
    fn try_into(self) -> Result<f64, Self::Error> {
        use Error::CannotBeCoercedToDouble;

        let Obj::Vector(Vector::Double(v)) = self.as_double()? else {
            return internal_err!();
        };

        match v.inner().clone().borrow()[..] {
            [OptionNA::Some(i), ..] => Ok(i),
            _ => Err(CannotBeCoercedToDouble.into()),
        }
    }
}

impl TryInto<Vec<f64>> for Obj {
    type Error = Signal;
    fn try_into(self) -> Result<Vec<f64>, Self::Error> {
        let Obj::Vector(Vector::Double(v)) = self.as_double()? else {
            return internal_err!();
        };

        Ok(v.inner()
            .clone()
            .borrow()
            .iter()
            .map(|vi| match vi {
                OptionNA::Some(i) => *i,
                OptionNA::NA => f64::NAN,
            })
            .collect())
    }
}
