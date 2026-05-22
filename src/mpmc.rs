//! MPMC channel implementations.
mod array;

pub use array::Array;

use crate::{channel, channel::Channel};

/// Alias of `MTx<T, mpsc::Array>`.
pub type MTx<T> = channel::MTx<T, Array>;
/// Alias of `MRx<T, mpsc::Array>`.
pub type MRx<T> = channel::MRx<T, Array>;

/// Alias of `mpsc::Array::new(capacity).channel()`.
pub fn channel<T>(capacity: usize) -> (MTx<T>, MRx<T>) {
    Array::new(capacity).channel()
}
