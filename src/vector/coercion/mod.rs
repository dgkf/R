//! Tools for coercing between internal vector types
//!
//! The primary workhorse of this implementation is [CoerceInto], which 
//! provides unilateral mappings between types that might be used for internal
//! representations. 
//!
//! For binary operations, two additional mappings are provided for pairs
//! of inputs, [CommonNum] (for numeric computations) and [CommonLog] (for 
//! logical computations).
//!
//! Provided two inputs, these mappings indicate what the common numeric
//! represenation of the two inputs should be before performing computation,
//! it does not require that the output is of that type (though that is often 
//! the case). If required, this can be customized via specific implementations
//! of the operators that are used.
//!

mod macros;

mod coerce_into;
pub use coerce_into::*;

mod common_num;
pub use common_num::*;

mod common_log;
pub use common_log::*;

mod map_common;
pub use map_common::*;
