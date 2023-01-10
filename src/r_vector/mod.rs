/// R Vectors module
///
/// This module is for handling of vectorized operators in R. All the numeric
/// coercion is handled within rust's type system. It is pretty faithful to
/// R's vector types, but there is room for improvement.
///
pub mod coercion;
pub mod iterators;
pub mod vectors;
