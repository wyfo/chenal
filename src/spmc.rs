//! SPMC channel implementations.
use crate::{backoff::NoBackoff, channel, channel::Channel};

mod array;
#[cfg(feature = "racy")]
mod racy_array;

pub use array::Array;
#[cfg(feature = "racy")]
pub use racy_array::RacyArray;

/// Alias of `MTx<T, spmc::Array>`.
pub type Tx<T> = channel::Tx<T, Array>;
/// Alias of `Rx<T, spmc::Array>`.
pub type MRx<T, B = NoBackoff> = channel::MRx<T, Array, B>;

/// Alias of `spmc::Array::new(capacity).channel()`.
pub fn channel<T>(capacity: usize) -> (Tx<T>, MRx<T>) {
    Array::new(capacity).channel()
}
