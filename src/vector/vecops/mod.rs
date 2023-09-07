//! Vector Operations
//! 
//! This module implements all the vector-wise primitives for algebraic
//! operations, logical operations and comparisons.
//! 

mod math;
pub use math::*;

mod logical;
pub use logical::*;

mod cmp;
pub use cmp::*;