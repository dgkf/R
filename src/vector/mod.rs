//! Vectors
//!
//! This module implements a generic vector interface for operating on data
//! in vector representations. Most notably, it provides implicit 
//! "duck-typing" for vector operations and implementations for all mathematical,
//! logical and comparison operators.
//! 
//! Internally, vectors can take different representations, allowing more
//! efficient ways of representing or generating their contents. 
//!
mod vector;
pub use vector::*;

pub mod utils;
pub mod types;
pub mod coercion;
pub mod rep;
pub mod vecops;
