//! # Design Notes
//!
//! When vectors are subset, we can lazily evaluate subsets such that 
//! they may still be assigned to in place without producing intermediate 
//! values.
//!
//! For example, 
//!
//! ```r,no_run
//! x[1:1e6][3:10][[4]] <- 10
//! ```
//!
//! In R, this would first allocate 
//!
//! ```r,no_run
//! x[1:1e6], which we'll call y, then
//!        y[3:10], which we'll call z, then
//!              z[[4]], which we'll call w, then
//!                    w <- 3, which we'll assign back to z
//!              z[[4]] <- 4, which we'll assign back to y
//!        y[3:10] <- z[[4]], which we'll assign back to x
//! x[1:1e6] <- y
//! ```
//!     
//! This becomes a painfully inefficient method for heavily subset assigment,
//! which intuitively should be fast. Instead, we can pre-calculate exactly
//! which indices should be affected. Only when a subset is assigned to a value
//! do we need to materialize it into a new vector (and even then, possibly 
//! only on-write using Cow)
//!
//! # TODOs:
//! - [ ] Vector recycling
//! - [ ] Fail (or warn) when recycled elements don't have a common multiple
//! - [ ] Better bounds checking and defaults when assigning to new indices
//!

#![allow(dead_code)]

pub mod utils;
pub mod atomic;
pub mod types;
pub mod coercion;
pub mod vector;
pub mod subsets;
