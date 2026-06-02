//! MPMC channel implementations.
mod array;
#[cfg(feature = "racy")]
mod racy_array;

pub use array::Array;
#[cfg(feature = "racy")]
pub use racy_array::RacyArray;

use crate::{backoff::NoBackoff, channel, channel::Channel};

/// Alias of `MTx<T, mpmc::Array>`.
pub type MTx<T, B = NoBackoff> = channel::MTx<T, Array, B>;
/// Alias of `MRx<T, mpmc::Array>`.
pub type MRx<T, B = NoBackoff> = channel::MRx<T, Array, B>;

/// Alias of `mpmc::Array::new(capacity).channel()`.
pub fn channel<T>(capacity: usize) -> (MTx<T>, MRx<T>) {
    Array::new(capacity).channel()
}
