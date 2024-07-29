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
            (Obj::List(l), Obj::List(r)) => {
                let lb = l.values.borrow();
                let rb = r.values.borrow();
                let liter = lb.iter();
                let riter = rb.iter();
                liter
                    .zip(riter)
                    .all(|((lk, lv), (rk, rv))| lk == rk && lv == rv)
            }
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

pub trait CoercibleToNumeric {
    fn coercible_to_numeric(&self) -> bool;
}

pub trait InterpretableAsLogical {
    fn interpretable_as_logical(&self) -> bool;
}

pub trait Comparable {
    fn comparable(&self) -> bool;
}

pub trait Orderable {
    fn orderable(&self) -> bool;
}

impl Orderable for (&Obj, &Obj) {
    fn orderable(&self) -> bool {
        matches!(self, (Obj::Vector(_), Obj::Vector(_)))
    }
}

impl CoercibleToNumeric for (&Obj, &Obj) {
    fn coercible_to_numeric(&self) -> bool {
        self.0.coercible_to_numeric() & self.1.coercible_to_numeric()
    }
}

impl CoercibleToNumeric for &Obj {
    fn coercible_to_numeric(&self) -> bool {
        match self {
            Obj::Vector(v) => match *v {
                Vector::Character(_) => false,
                Vector::Logical(_) => true,
                Vector::Double(_) => true,
                Vector::Integer(_) => true,
            },
            _ => false,
        }
    }
}

impl Comparable for (&Obj, &Obj) {
    fn comparable(&self) -> bool {
        matches!(
            (self.0, self.1),
            (Obj::Environment(_), Obj::Environment(_))
                | (Obj::Vector(_), Obj::Vector(_))
                | (Obj::Expr(_), Obj::Expr(_))
                | (Obj::Promise(..), Obj::Promise(..))
                | (Obj::Function(..), Obj::Promise(..))
                | (Obj::List(..), Obj::List(..))
        )
    }
}

impl InterpretableAsLogical for &Obj {
    fn interpretable_as_logical(&self) -> bool {
        match self {
            Obj::Vector(v) => !matches!(v, Vector::Character(_)),
            _ => false,
        }
    }
}

impl InterpretableAsLogical for (&Obj, &Obj) {
    fn interpretable_as_logical(&self) -> bool {
        self.0.interpretable_as_logical() & self.1.interpretable_as_logical()
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
