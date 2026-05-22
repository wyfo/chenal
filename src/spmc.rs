//! SPMC channel implementations.
mod array;

pub use array::Array;

use crate::{channel, channel::Channel};

/// Alias of `MTx<T, spmc::Array>`.
pub type Tx<T> = channel::Tx<T, Array>;
/// Alias of `Rx<T, spmc::Array>`.
pub type MRx<T> = channel::MRx<T, Array>;

/// Alias of `spmc::Array::new(capacity).channel()`.
pub fn channel<T>(capacity: usize) -> (Tx<T>, MRx<T>) {
    Array::new(capacity).channel()
}
