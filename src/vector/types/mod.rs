//! Internal types
//!
//! Internal types are considered to fall into one of four modes, which 
//! map to R's vector modes. This is modeled for familarity, but is not
//! required for the vector implementation as even within modes there are
//! multiple valid representations - for example, [i8] and [i16] are both
//! valid integer types.
//!
//! A central feature of atomic representations is that they must have a 
//! representation for `NA`. For many `rust` atomic types, this is not the case,
//! and therefore these types are wrapped in [option_na::OptionNa], which is 
//! a simple `enum` indicating that there is some valid value or that the value
//! is `NA`.
//!
//! All internal types implement [atomic::Atomic], and the `rust` primitives
//! that form their basis implement [atomic::IntoAtomic]. Simply put,
//! this guarantees that any primitive data has an `NA` representation and 
//! claims to be of any `mode` (preferrably only one, though the current 
//! implementation certainly allows for data to claim to be of more than one 
//! mode).
//!

pub mod atomic;
pub mod modes;

mod option_na;
pub use option_na::*;

mod na_able;
pub use na_able::*;

mod numerics;
pub use numerics::*;

mod integers;
pub use integers::*;

mod logicals;
pub use logicals::*;

mod characters;
pub use characters::*;
