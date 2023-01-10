use std::fmt::Debug;
use std::fmt::Display;

use super::coercion::*;
use super::iterators::*;

#[derive(Clone, PartialEq)]
pub enum OptionNA<T> {
    Some(T),
    NA,
}

#[derive(Debug, Clone)]
pub enum RVector {
    Numeric(Vec<OptionNA<f64>>),
    Integer(Vec<OptionNA<i32>>),
    Logical(Vec<OptionNA<bool>>),
    Character(Vec<OptionNA<String>>),
    // Complex(Complex),
    // Raw(Raw),
}
impl RVector {
    pub fn get(&self, index: usize) -> Option<RVector> {
        use RVector::*;

        fn f<T>(v: &Vec<T>, index: usize) -> Option<RVector>
        where
            T: Clone,
            RVector: From<Vec<T>>,
        {
            if let Some(value) = v.get(index) {
                Some(RVector::from(vec![value.clone()]))
            } else {
                None
            }
        }

        match self {
            Numeric(x) => f(x, index),
            Integer(x) => f(x, index),
            Logical(x) => f(x, index),
            Character(x) => f(x, index),
        }
    }

    pub fn vec_coerce<T, U>(v: &Vec<OptionNA<T>>) -> Vec<OptionNA<U>>
    where
        T: CoercibleInto<U> + Clone,
    {
        use OptionNA::*;
        v.into_iter()
            .map(|i| match i {
                Some(x) => Some(CoercibleInto::<U>::coerce_into(x.clone())),
                NA => NA,
            })
            .collect()
    }

    pub fn vec_parse<U>(v: &Vec<OptionNA<String>>) -> (bool, Vec<OptionNA<U>>)
    where
        U: std::str::FromStr,
    {
        use OptionNA::*;
        let mut any_new_nas = false;
        let result = v
            .into_iter()
            .map(|i| match i {
                Some(s) => match s.parse::<U>() {
                    Ok(value) => Some(value),
                    Err(_) => {
                        any_new_nas = true;
                        NA
                    }
                },
                NA => NA,
            })
            .collect();

        (any_new_nas, result)
    }

    pub fn as_integer(self) -> RVector {
        use RVector::*;
        match self {
            Numeric(v) => Integer(Self::vec_coerce::<f64, i32>(&v)),
            Integer(_) => self,
            Logical(v) => Integer(Self::vec_coerce::<bool, i32>(&v)),
            Character(v) => {
                let (_any_new_nas, v) = Self::vec_parse::<i32>(&v);
                Integer(v)
            }
        }
    }

    pub fn as_numeric(self) -> RVector {
        use RVector::*;
        match self {
            Numeric(v) => Numeric(v),
            Integer(v) => Numeric(Self::vec_coerce::<i32, f64>(&v)),
            Logical(v) => Numeric(Self::vec_coerce::<bool, f64>(&v)),
            Character(v) => {
                let (_any_new_nas, v) = Self::vec_parse::<f64>(&v);
                Numeric(v)
            }
        }
    }
}

impl TryInto<bool> for RVector {
    type Error = ();
    fn try_into(self) -> Result<bool, Self::Error> {
        use OptionNA::*;
        use RVector::*;
        match self {
            Numeric(v) => match v[..] {
                [Some(vi)] if vi == 0_f64 => Ok(false),
                [Some(_)] => Ok(true),
                [NA] => Err(()), // missing value where TRUE/FALSE needed
                [] => Err(()),   // argument is of length zero
                _ => Err(()),    // condition has length > 1
            },
            Integer(v) => match v[..] {
                [Some(0)] => Ok(false),
                [Some(_)] => Ok(true),
                [NA] => Err(()), // missing value where TRUE/FALSE needed
                [] => Err(()),   // argument is of length zero
                _ => Err(()),    // condition has length > 1
            },
            Logical(v) => match v[..] {
                [Some(true)] => Ok(true),
                [Some(false)] => Ok(false),
                [NA] => Err(()), // missing value where TRUE/FALSE needed
                [] => Err(()),   // argument is of length zero
                _ => Err(()),    // condition has length > 1
            },
            Character(v) => match &v[..] {
                [Some(x)] if x.as_str() == "TRUE" => Ok(true),
                [Some(x)] if x.as_str() == "FALSE" => Ok(false),
                [Some(_)] => Err(()), // argument is not interpretable as logical
                [NA] => Err(()),      // missing value where TRUE/FALSE needed
                [] => Err(()),        // argument is of length zero
                _ => Err(()),         // condition has length > 1
            },
        }
    }
}

impl From<Vec<f64>> for RVector {
    fn from(x: Vec<f64>) -> Self {
        RVector::Numeric(x.into_iter().map(|i| OptionNA::Some(i)).collect())
    }
}

impl From<Vec<OptionNA<f64>>> for RVector {
    fn from(x: Vec<OptionNA<f64>>) -> Self {
        RVector::Numeric(x)
    }
}

impl From<Vec<i32>> for RVector {
    fn from(x: Vec<i32>) -> Self {
        RVector::Integer(x.into_iter().map(|i| OptionNA::Some(i)).collect())
    }
}

impl From<Vec<OptionNA<i32>>> for RVector {
    fn from(x: Vec<OptionNA<i32>>) -> Self {
        RVector::Integer(x)
    }
}

impl From<Vec<bool>> for RVector {
    fn from(x: Vec<bool>) -> Self {
        RVector::Logical(x.into_iter().map(|i| OptionNA::Some(i)).collect())
    }
}

impl From<Vec<OptionNA<bool>>> for RVector {
    fn from(x: Vec<OptionNA<bool>>) -> Self {
        RVector::Logical(x)
    }
}

impl From<Vec<String>> for RVector {
    fn from(x: Vec<String>) -> Self {
        RVector::Character(x.into_iter().map(|i| OptionNA::Some(i)).collect())
    }
}

impl From<Vec<OptionNA<String>>> for RVector {
    fn from(x: Vec<OptionNA<String>>) -> Self {
        RVector::Character(x)
    }
}

impl<T> Debug for OptionNA<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptionNA::Some(x) => write!(f, "{:?}", x),
            OptionNA::NA => write!(f, "NA"),
        }
    }
}

impl<T> Display for OptionNA<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptionNA::Some(x) => write!(f, "{}", x),
            OptionNA::NA => write!(f, "NA"),
        }
    }
}

impl Display for RVector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn fmt_vec<T>(x: &Vec<T>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
        where
            T: Display,
        {
            let n = x.len();
            let nlen = format!("{}", n).len();

            if n == 0 {
                return write!(f, "numeric(0)");
            }

            let x_strs = x.iter().map(|xi| format!("{}", xi));
            let max_len = x_strs
                .clone()
                .fold(0, |max_len, xi| std::cmp::max(max_len, xi.len()));

            let mut col = 0;
            x_strs.enumerate().try_for_each(|(i, x_str)| {
                col += max_len + 1;
                if i == 0 {
                    write!(f, "{:>3$}[{}] {:>4$} ", "", i + 1, x_str, nlen - 1, max_len)
                } else if col > 80 - nlen - 3 {
                    col = 0;
                    let i_str = format!("{}", i + 1);
                    let gutter = nlen - i_str.len();
                    write!(f, "\n{:>3$}[{}] {:>4$} ", "", i_str, x_str, gutter, max_len)
                } else {
                    write!(f, "{:>1$} ", x_str, max_len)
                }
            })
        }

        match self {
            RVector::Numeric(x) => fmt_vec(x, f),
            RVector::Integer(x) => fmt_vec(x, f),
            RVector::Logical(x) => fmt_vec(x, f),
            RVector::Character(x) => fmt_vec(x, f),
        }
    }
}

impl<T> std::ops::Add for OptionNA<T>
where
    T: std::ops::Add<Output = T>,
{
    type Output = OptionNA<T>;
    fn add(self, rhs: Self) -> Self::Output {
        use OptionNA::*;
        match (self, rhs) {
            (Some(l), Some(r)) => Some(l + r),
            _ => NA,
        }
    }
}

impl<T> std::ops::Sub for OptionNA<T>
where
    T: std::ops::Sub<Output = T>,
{
    type Output = OptionNA<T>;
    fn sub(self, rhs: Self) -> Self::Output {
        use OptionNA::*;
        match (self, rhs) {
            (Some(l), Some(r)) => Some(l - r),
            _ => NA,
        }
    }
}

impl<T> std::ops::Mul for OptionNA<T>
where
    T: std::ops::Mul<Output = T>,
{
    type Output = OptionNA<T>;
    fn mul(self, rhs: Self) -> Self::Output {
        use OptionNA::*;
        match (self, rhs) {
            (Some(l), Some(r)) => Some(l * r),
            _ => NA,
        }
    }
}

impl<T> std::ops::Div for OptionNA<T>
where
    T: std::ops::Div<Output = T>,
{
    type Output = OptionNA<T>;
    fn div(self, rhs: Self) -> Self::Output {
        use OptionNA::*;
        match (self, rhs) {
            (Some(l), Some(r)) => Some(l / r),
            _ => NA,
        }
    }
}

impl std::ops::Add for RVector {
    type Output = RVector;

    fn add(self, rhs: Self) -> Self::Output {
        use RVector::*;

        fn f<L, R, LNum, RNum, LRNum>(l: Vec<L>, r: Vec<R>) -> RVector
        where
            L: IntoNumeric<LNum> + Clone,
            R: IntoNumeric<RNum> + Clone,
            (LNum, RNum): CommonNum<LRNum>,
            RVector: From<Vec<<LRNum as std::ops::Add>::Output>>,
            LRNum: std::ops::Add,
        {
            RVector::from(
                map_common_add_numeric(zip_recycle(l, r))
                    .map(|(l, r)| l + r)
                    .collect::<Vec<_>>(),
            )
        }

        match (self, rhs) {
            (Numeric(l), Numeric(r)) => f(l, r),
            (Numeric(l), Integer(r)) => f(l, r),
            (Numeric(l), Logical(r)) => f(l, r),
            (Integer(l), Numeric(r)) => f(l, r),
            (Integer(l), Integer(r)) => f(l, r),
            (Integer(l), Logical(r)) => f(l, r),
            (Logical(l), Numeric(r)) => f(l, r),
            (Logical(l), Integer(r)) => f(l, r),
            (Logical(l), Logical(r)) => f(l, r),
            _ => todo!(),
        }
    }
}

impl std::ops::Sub for RVector {
    type Output = RVector;

    fn sub(self, rhs: Self) -> Self::Output {
        use RVector::*;

        fn f<L, R, LNum, RNum, LRNum>(l: Vec<L>, r: Vec<R>) -> RVector
        where
            L: IntoNumeric<LNum> + Clone,
            R: IntoNumeric<RNum> + Clone,
            (LNum, RNum): CommonNum<LRNum>,
            RVector: From<Vec<<LRNum as std::ops::Sub>::Output>>,
            LRNum: std::ops::Sub,
        {
            RVector::from(
                map_common_add_numeric(zip_recycle(l, r))
                    .map(|(l, r)| l - r)
                    .collect::<Vec<_>>(),
            )
        }

        match (self, rhs) {
            (Numeric(l), Numeric(r)) => f(l, r),
            (Numeric(l), Integer(r)) => f(l, r),
            (Numeric(l), Logical(r)) => f(l, r),
            (Integer(l), Numeric(r)) => f(l, r),
            (Integer(l), Integer(r)) => f(l, r),
            (Integer(l), Logical(r)) => f(l, r),
            (Logical(l), Numeric(r)) => f(l, r),
            (Logical(l), Integer(r)) => f(l, r),
            (Logical(l), Logical(r)) => f(l, r),
            _ => todo!(),
        }
    }
}

impl std::ops::Mul for RVector {
    type Output = RVector;

    fn mul(self, rhs: Self) -> Self::Output {
        use RVector::*;

        fn f<L, R, LNum, RNum, LRNum>(l: Vec<L>, r: Vec<R>) -> RVector
        where
            L: IntoNumeric<LNum> + Clone,
            R: IntoNumeric<RNum> + Clone,
            (LNum, RNum): CommonNum<LRNum>,
            RVector: From<Vec<<LRNum as std::ops::Mul>::Output>>,
            LRNum: std::ops::Mul,
        {
            RVector::from(
                map_common_mul_numeric(zip_recycle(l, r))
                    .map(|(l, r)| l * r)
                    .collect::<Vec<_>>(),
            )
        }

        match (self, rhs) {
            (Numeric(l), Numeric(r)) => f(l, r),
            (Numeric(l), Integer(r)) => f(l, r),
            (Numeric(l), Logical(r)) => f(l, r),
            (Integer(l), Numeric(r)) => f(l, r),
            (Integer(l), Integer(r)) => f(l, r),
            (Integer(l), Logical(r)) => f(l, r),
            (Logical(l), Numeric(r)) => f(l, r),
            (Logical(l), Integer(r)) => f(l, r),
            (Logical(l), Logical(r)) => f(l, r),
            _ => todo!(),
        }
    }
}

impl std::ops::Div for RVector {
    type Output = RVector;

    fn div(self, rhs: Self) -> Self::Output {
        use RVector::*;

        fn f<L, R, LNum, RNum, LRNum>(l: Vec<L>, r: Vec<R>) -> RVector
        where
            L: IntoNumeric<LNum> + Clone,
            R: IntoNumeric<RNum> + Clone,
            (LNum, RNum): CommonNum<LRNum>,
            RVector: From<Vec<<LRNum as std::ops::Div>::Output>>,
            LRNum: std::ops::Div,
        {
            RVector::from(
                map_common_mul_numeric(zip_recycle(l, r))
                    .map(|(l, r)| l / r)
                    .collect::<Vec<_>>(),
            )
        }

        match (self, rhs) {
            (Numeric(l), Numeric(r)) => f(l, r),
            (Numeric(l), Integer(r)) => f(l, r),
            (Numeric(l), Logical(r)) => f(l, r),
            (Integer(l), Numeric(r)) => f(l, r),
            (Integer(l), Integer(r)) => f(l, r),
            (Integer(l), Logical(r)) => f(l, r),
            (Logical(l), Numeric(r)) => f(l, r),
            (Logical(l), Integer(r)) => f(l, r),
            (Logical(l), Logical(r)) => f(l, r),
            _ => todo!(),
        }
    }
}
