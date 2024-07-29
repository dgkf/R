use crate::object::Obj;
use crate::object::Vector;

pub trait CoercibleToNumeric {
    fn coercible_to_numeric(&self) -> bool;
}

pub trait InterpretableAsLogical {
    fn interpretable_as_logical(&self) -> bool;
}

pub trait Comparable {
    fn comparable(&self) -> bool;
}
impl CoercibleToNumeric for (&Obj, &Obj) {
    fn coercible_to_numeric(&self) -> bool {
        match self {
            (Obj::Vector(l), Obj::Vector(r)) => match (l, r) {
                (Vector::Character(_), _) | (_, Vector::Character(_)) => false,
                _ => true,
            },
            _ => false,
        }
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
        match self {
            (Obj::Vector(_), Obj::Vector(_)) => true,
            _ => false,
        }
    }
}

impl InterpretableAsLogical for &Obj {
    fn interpretable_as_logical(&self) -> bool {
        match self {
            Obj::Vector(v) => match v {
                Vector::Character(_) => false,
                _ => true,
            },
            _ => false,
        }
    }
}

impl InterpretableAsLogical for (&Obj, &Obj) {
    fn interpretable_as_logical(&self) -> bool {
        self.0.interpretable_as_logical() & self.1.interpretable_as_logical()
    }
}
