use std::fmt::Debug;
use std::fmt::Display;

use crate::error::RError;
use crate::lang::RSignal;
use crate::r_vector::coercion::CoercibleInto;

use super::coercion::*;
use super::iterators::*;

#[derive(Clone, PartialEq)]
pub enum OptionNA<T> {
    Some(T),
    NA,
}

#[derive(Debug, Clone)]
pub enum Vector {
    Numeric(Vec<OptionNA<f64>>),
    Integer(Vec<OptionNA<i32>>),
    Logical(Vec<OptionNA<bool>>),
    Character(Vec<OptionNA<String>>),
    // Complex(Complex),
    // Raw(Raw),
}

impl Vector {
    pub fn get(&self, index: usize) -> Option<Vector> {
        use Vector::*;

        fn f<T>(v: &Vec<T>, index: usize) -> Option<Vector>
        where
            T: Clone,
            Vector: From<Vec<T>>,
        {
            if let Some(value) = v.get(index - 1) {
                Some(Vector::from(vec![value.clone()]))
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

    pub fn set_from_vec(&mut self, index: Vector, values: Vector) -> Result<(), RSignal> {
        let index = index.as_integer();

        let i = match index.as_integer() {
            Vector::Integer(ivec) => match ivec[..] {
                [OptionNA::Some(i)] => i,
                _ => {
                    return Err(RSignal::Error(RError::Other(
                        "can only index into vector with integer indices".to_string(),
                    )))
                }
            },
            _ => unreachable!(),
        };

        if values.len() != 1 {
            return Err(RSignal::Error(RError::Other(
                "cannot assign multiple values to single index".to_string(),
            )));
        };

        use Vector::*;
        match self {
            Numeric(l) => match values.as_numeric() {
                Numeric(r) => l[(i - 1) as usize] = r[0].clone(),
                _ => unreachable!(),
            },
            Integer(l) => match values.as_integer() {
                Integer(r) => l[(i - 1) as usize] = r[0].clone(),
                _ => unreachable!(),
            },
            Logical(l) => match values.as_logical() {
                Logical(r) => l[(i - 1) as usize] = r[0].clone(),
                _ => unreachable!(),
            },
            Character(l) => match values.as_character() {
                Character(r) => l[(i - 1) as usize] = r[0].clone(),
                _ => unreachable!(),
            },
        }

        Ok(())
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

    pub fn as_integer(self) -> Vector {
        use Vector::*;
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

    pub fn as_numeric(self) -> Vector {
        use Vector::*;
        match self {
            Numeric(_) => self,
            Integer(v) => Numeric(Self::vec_coerce::<i32, f64>(&v)),
            Logical(v) => Numeric(Self::vec_coerce::<bool, f64>(&v)),
            Character(v) => {
                let (_any_new_nas, v) = Self::vec_parse::<f64>(&v);
                Numeric(v)
            }
        }
    }

    pub fn as_logical(self) -> Vector {
        use Vector::*;
        match self {
            Numeric(v) => Logical(v.into_iter().map(|i| i.as_logical()).collect::<Vec<_>>()),
            Integer(v) => Logical(v.into_iter().map(|i| i.as_logical()).collect::<Vec<_>>()),
            Logical(_) => self,
            Character(v) => {
                let (_any_new_nas, v) = Self::vec_parse::<bool>(&v);
                Logical(v)
            }
        }
    }

    pub fn as_character(self) -> Vector {
        use Vector::*;
        match self {
            Numeric(v) => Character(Self::vec_coerce::<f64, String>(&v)),
            Integer(v) => Character(Self::vec_coerce::<i32, String>(&v)),
            Logical(v) => Character(Self::vec_coerce::<bool, String>(&v)),
            Character(_) => self,
        }
    }

    pub fn len(&self) -> usize {
        use Vector::*;
        match self {
            Numeric(v) => v.len(),
            Integer(v) => v.len(),
            Logical(v) => v.len(),
            Character(v) => v.len(),
        }
    }

    pub fn reserve(self, additional: usize) {
        use Vector::*;
        match self {
            Numeric(mut v) => v.reserve(additional),
            Integer(mut v) => v.reserve(additional),
            Logical(mut v) => v.reserve(additional),
            Character(mut v) => v.reserve(additional),
        }
    }
}

impl TryInto<bool> for Vector {
    type Error = ();
    fn try_into(self) -> Result<bool, Self::Error> {
        use OptionNA::*;
        use Vector::*;
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

impl From<Vec<f64>> for Vector {
    fn from(x: Vec<f64>) -> Self {
        Vector::Numeric(x.into_iter().map(|i| OptionNA::Some(i)).collect())
    }
}

impl From<Vec<OptionNA<f64>>> for Vector {
    fn from(x: Vec<OptionNA<f64>>) -> Self {
        Vector::Numeric(x)
    }
}

impl From<Vec<i32>> for Vector {
    fn from(x: Vec<i32>) -> Self {
        Vector::Integer(x.into_iter().map(|i| OptionNA::Some(i)).collect())
    }
}

impl From<Vec<OptionNA<i32>>> for Vector {
    fn from(x: Vec<OptionNA<i32>>) -> Self {
        Vector::Integer(x)
    }
}

impl From<Vec<bool>> for Vector {
    fn from(x: Vec<bool>) -> Self {
        Vector::Logical(x.into_iter().map(|i| OptionNA::Some(i)).collect())
    }
}

impl From<Vec<OptionNA<bool>>> for Vector {
    fn from(x: Vec<OptionNA<bool>>) -> Self {
        Vector::Logical(x)
    }
}

impl From<Vec<String>> for Vector {
    fn from(x: Vec<String>) -> Self {
        Vector::Character(x.into_iter().map(|i| OptionNA::Some(i)).collect())
    }
}

impl Into<Vec<String>> for Vector {
    fn into(self) -> Vec<String> {
        match self {
            Vector::Character(v) => v
                .iter()
                .map(|x| match x {
                    OptionNA::Some(val) => val.clone(),
                    OptionNA::NA => "NA".to_string(),
                })
                .collect(),
            _ => todo!(),
        }
    }
}

impl From<Vec<OptionNA<String>>> for Vector {
    fn from(x: Vec<OptionNA<String>>) -> Self {
        Vector::Character(x)
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

impl Display for Vector {
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

        fn fmt_strs(x: &Vec<OptionNA<String>>) -> Vec<String> {
            use OptionNA::*;
            x.into_iter()
                .map(|i| match i {
                    Some(x) => format!("\"{}\"", x),
                    NA => "NA".to_string(),
                })
                .collect()
        }

        match self {
            Vector::Numeric(x) => fmt_vec(x, f),
            Vector::Integer(x) => fmt_vec(x, f),
            Vector::Logical(x) => fmt_vec(x, f),
            Vector::Character(x) => fmt_vec(&fmt_strs(x), f),
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

impl<T> std::ops::Neg for OptionNA<T>
where
    T: std::ops::Neg<Output = T>,
{
    type Output = OptionNA<T>;
    fn neg(self) -> Self::Output {
        use OptionNA::*;
        match self {
            Some(x) => Some(x.neg()),
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

pub trait Pow {
    type Output;
    /// raise self to the rhs power
    fn power(self, rhs: Self) -> Self::Output;
}

impl Pow for i32 {
    type Output = i32;
    fn power(self, rhs: Self) -> Self::Output {
        i32::pow(self, rhs as u32)
    }
}

impl Pow for f64 {
    type Output = f64;
    fn power(self, rhs: Self) -> Self::Output {
        f64::powf(self, rhs)
    }
}

impl<T> Pow for OptionNA<T>
where
    T: Pow,
{
    type Output = OptionNA<<T as Pow>::Output>;
    fn power(self, rhs: Self) -> Self::Output {
        use OptionNA::*;
        match (self, rhs) {
            (Some(l), Some(r)) => Some(T::power(l, r)),
            _ => NA,
        }
    }
}

impl<T> std::ops::Rem for OptionNA<T>
where
    T: std::ops::Rem,
{
    type Output = OptionNA<<T as std::ops::Rem>::Output>;
    fn rem(self, rhs: Self) -> Self::Output {
        use OptionNA::*;
        match (self, rhs) {
            (Some(l), Some(r)) => Some(l.rem(r)),
            _ => NA,
        }
    }
}

impl std::ops::Add for Vector {
    type Output = Vector;
    fn add(self, rhs: Self) -> Self::Output {
        use Vector::*;

        fn f<L, R, LNum, RNum, LRNum>(l: Vec<L>, r: Vec<R>) -> Vector
        where
            L: IntoNumeric<LNum> + Clone,
            R: IntoNumeric<RNum> + Clone,
            (LNum, RNum): CommonNum<LRNum>,
            Vector: From<Vec<<LRNum as std::ops::Add>::Output>>,
            LRNum: std::ops::Add,
        {
            Vector::from(
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

impl std::ops::Sub for Vector {
    type Output = Vector;
    fn sub(self, rhs: Self) -> Self::Output {
        use Vector::*;

        fn f<L, R, LNum, RNum, LRNum>(l: Vec<L>, r: Vec<R>) -> Vector
        where
            L: IntoNumeric<LNum> + Clone,
            R: IntoNumeric<RNum> + Clone,
            (LNum, RNum): CommonNum<LRNum>,
            Vector: From<Vec<<LRNum as std::ops::Sub>::Output>>,
            LRNum: std::ops::Sub,
        {
            Vector::from(
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

impl std::ops::Neg for Vector {
    type Output = Vector;
    fn neg(self) -> Self::Output {
        use Vector::*;

        fn f<T, TNum>(x: Vec<T>) -> Vector
        where
            T: IntoNumeric<TNum>,
            Vector: From<Vec<<TNum as std::ops::Neg>::Output>>,
            TNum: std::ops::Neg,
        {
            Vector::from(
                x.into_iter()
                    .map(|i| i.as_numeric().neg())
                    .collect::<Vec<_>>(),
            )
        }

        match self {
            Numeric(x) => f(x),
            Integer(x) => f(x),
            Logical(x) => f(x),
            _ => todo!(),
        }
    }
}

impl std::ops::Mul for Vector {
    type Output = Vector;
    fn mul(self, rhs: Self) -> Self::Output {
        use Vector::*;

        fn f<L, R, LNum, RNum, LRNum>(l: Vec<L>, r: Vec<R>) -> Vector
        where
            L: IntoNumeric<LNum> + Clone,
            R: IntoNumeric<RNum> + Clone,
            (LNum, RNum): CommonNum<LRNum>,
            Vector: From<Vec<<LRNum as std::ops::Mul>::Output>>,
            LRNum: std::ops::Mul,
        {
            Vector::from(
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

impl std::ops::Div for Vector {
    type Output = Vector;
    fn div(self, rhs: Self) -> Self::Output {
        use Vector::*;

        fn f<L, R, LNum, RNum, LRNum>(l: Vec<L>, r: Vec<R>) -> Vector
        where
            L: IntoNumeric<LNum> + Clone,
            R: IntoNumeric<RNum> + Clone,
            (LNum, RNum): CommonNum<LRNum>,
            Vector: From<Vec<<LRNum as std::ops::Div>::Output>>,
            LRNum: std::ops::Div,
        {
            Vector::from(
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

impl Pow for Vector {
    type Output = Vector;
    fn power(self, rhs: Self) -> Self::Output {
        use Vector::*;

        fn f<L, R, LNum, RNum, LRNum>(l: Vec<L>, r: Vec<R>) -> Vector
        where
            L: IntoNumeric<LNum> + Clone,
            R: IntoNumeric<RNum> + Clone,
            (LNum, RNum): CommonNum<LRNum>,
            Vector: From<Vec<<LRNum as Pow>::Output>>,
            LRNum: Pow,
        {
            Vector::from(
                map_common_mul_numeric(zip_recycle(l, r))
                    .map(|(l, r)| LRNum::power(l, r))
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

pub trait VecPartialCmp {
    type CmpOutput;
    type Output;
    fn vec_partial_cmp(self, rhs: Self) -> Self::CmpOutput;
    fn vec_gt(self, rhs: Self) -> Self::Output;
    fn vec_gte(self, rhs: Self) -> Self::Output;
    fn vec_lt(self, rhs: Self) -> Self::Output;
    fn vec_lte(self, rhs: Self) -> Self::Output;
    fn vec_eq(self, rhs: Self) -> Self::Output;
    fn vec_neq(self, rhs: Self) -> Self::Output;
}

impl VecPartialCmp for Vector {
    type CmpOutput = Vec<Option<std::cmp::Ordering>>;
    type Output = Vector;

    fn vec_partial_cmp(self, rhs: Self) -> Self::CmpOutput {
        use Vector::*;

        fn f<L, R, LR>(l: Vec<OptionNA<L>>, r: Vec<OptionNA<R>>) -> Vec<Option<std::cmp::Ordering>>
        where
            L: CoercibleInto<LR> + Clone,
            R: CoercibleInto<LR> + Clone,
            (L, R): CommonCmp<LR>,
            LR: PartialOrd,
        {
            zip_recycle(l.into_iter(), r.into_iter())
                .map(|pair| match pair {
                    (OptionNA::Some(l), OptionNA::Some(r)) => {
                        let l = CoercibleInto::<LR>::coerce_into(l);
                        let r = CoercibleInto::<LR>::coerce_into(r);
                        l.partial_cmp(&r)
                    }
                    _ => None,
                })
                .collect()
        }

        match (self, rhs) {
            (Numeric(l), Numeric(r)) => f(l, r),
            (Numeric(l), Integer(r)) => f(l, r),
            (Numeric(l), Logical(r)) => f(l, r),
            (Numeric(l), Character(r)) => f(l, r),
            (Integer(l), Numeric(r)) => f(l, r),
            (Integer(l), Integer(r)) => f(l, r),
            (Integer(l), Logical(r)) => f(l, r),
            (Integer(l), Character(r)) => f(l, r),
            (Logical(l), Numeric(r)) => f(l, r),
            (Logical(l), Integer(r)) => f(l, r),
            (Logical(l), Logical(r)) => f(l, r),
            (Logical(l), Character(r)) => f(l, r),
            (Character(l), Numeric(r)) => f(l, r),
            (Character(l), Integer(r)) => f(l, r),
            (Character(l), Logical(r)) => f(l, r),
            (Character(l), Character(r)) => f(l, r),
        }
    }

    fn vec_gt(self, rhs: Self) -> Self::Output {
        use std::cmp::Ordering::*;
        Vector::Logical(
            self.vec_partial_cmp(rhs)
                .into_iter()
                .map(|i| match i {
                    Some(Greater) => OptionNA::Some(true),
                    Some(_) => OptionNA::Some(false),
                    None => OptionNA::NA,
                })
                .collect(),
        )
    }

    fn vec_gte(self, rhs: Self) -> Self::Output {
        use std::cmp::Ordering::*;
        Vector::Logical(
            self.vec_partial_cmp(rhs)
                .into_iter()
                .map(|i| match i {
                    Some(Greater | Equal) => OptionNA::Some(true),
                    Some(_) => OptionNA::Some(false),
                    None => OptionNA::NA,
                })
                .collect(),
        )
    }

    fn vec_lt(self, rhs: Self) -> Self::Output {
        use std::cmp::Ordering::*;
        Vector::Logical(
            self.vec_partial_cmp(rhs)
                .into_iter()
                .map(|i| match i {
                    Some(Less) => OptionNA::Some(true),
                    Some(_) => OptionNA::Some(false),
                    None => OptionNA::NA,
                })
                .collect(),
        )
    }

    fn vec_lte(self, rhs: Self) -> Self::Output {
        use std::cmp::Ordering::*;
        Vector::Logical(
            self.vec_partial_cmp(rhs)
                .into_iter()
                .map(|i| match i {
                    Some(Less | Equal) => OptionNA::Some(true),
                    Some(_) => OptionNA::Some(false),
                    None => OptionNA::NA,
                })
                .collect(),
        )
    }

    fn vec_eq(self, rhs: Self) -> Self::Output {
        use std::cmp::Ordering::*;
        Vector::Logical(
            self.vec_partial_cmp(rhs)
                .into_iter()
                .map(|i| match i {
                    Some(Equal) => OptionNA::Some(true),
                    Some(_) => OptionNA::Some(false),
                    None => OptionNA::NA,
                })
                .collect(),
        )
    }

    fn vec_neq(self, rhs: Self) -> Self::Output {
        use std::cmp::Ordering::*;
        Vector::Logical(
            self.vec_partial_cmp(rhs)
                .into_iter()
                .map(|i| match i {
                    Some(Equal) => OptionNA::Some(false),
                    Some(_) => OptionNA::Some(true),
                    None => OptionNA::NA,
                })
                .collect(),
        )
    }
}

impl std::ops::Rem for Vector {
    type Output = Vector;
    fn rem(self, rhs: Self) -> Self::Output {
        use Vector::*;

        fn f<L, R, LNum, RNum, LRNum>(l: Vec<L>, r: Vec<R>) -> Vector
        where
            L: IntoNumeric<LNum> + Clone,
            R: IntoNumeric<RNum> + Clone,
            (LNum, RNum): CommonNum<LRNum>,
            Vector: From<Vec<<LRNum as std::ops::Rem>::Output>>,
            LRNum: std::ops::Rem,
        {
            Vector::from(
                map_common_mul_numeric(zip_recycle(l, r))
                    .map(|(l, r)| l.rem(r))
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

impl std::ops::BitOr for Vector {
    type Output = Vector;
    fn bitor(self, rhs: Self) -> Self::Output {
        use OptionNA::*;
        use Vector::*;

        match (self.as_logical(), rhs.as_logical()) {
            (Logical(l), Logical(r)) => Vector::Logical(
                zip_recycle(l.into_iter(), r.into_iter())
                    .map(|(l, r)| match (l, r) {
                        (Some(l), Some(r)) => Some(l || r),
                        _ => NA,
                    })
                    .collect::<Vec<_>>(),
            ),
            _ => todo!(),
        }
    }
}

impl std::ops::BitAnd for Vector {
    type Output = Vector;
    fn bitand(self, rhs: Self) -> Self::Output {
        use OptionNA::*;
        use Vector::*;

        match (self.as_logical(), rhs.as_logical()) {
            (Logical(l), Logical(r)) => Vector::Logical(
                zip_recycle(l.into_iter(), r.into_iter())
                    .map(|(l, r)| match (l, r) {
                        (Some(l), Some(r)) => Some(l && r),
                        _ => NA,
                    })
                    .collect::<Vec<_>>(),
            ),
            _ => todo!(),
        }
    }
}
