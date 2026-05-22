//! MPMC channel implementations.
mod array;

pub use array::Array;

use crate::{channel, channel::Channel};

/// Alias of `MTx<T, mpmc::Array>`.
pub type MTx<T> = channel::MTx<T, Array>;
/// Alias of `MRx<T, mpmc::Array>`.
pub type MRx<T> = channel::MRx<T, Array>;

/// Alias of `mpmc::Array::new(capacity).channel()`.
pub fn channel<T>(capacity: usize) -> (MTx<T>, MRx<T>) {
    Array::new(capacity).channel()
}
