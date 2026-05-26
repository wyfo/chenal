//! MPSC channel implementations.
mod array;

pub use array::Array;

use crate::{backoff::NoBackoff, channel, channel::Channel};

/// Alias of `MTx<T, mpsc::Array>`.
pub type MTx<T, B = NoBackoff> = channel::MTx<T, Array, B>;
/// Alias of `Rx<T, mpsc::Array>`.
pub type Rx<T> = channel::Rx<T, Array>;

/// Alias of `mpsc::Array::new(capacity).channel()`.
pub fn channel<T>(capacity: usize) -> (MTx<T>, Rx<T>) {
    Array::new(capacity).channel()
}
