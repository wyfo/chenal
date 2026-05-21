//! SPSC channel implementations.
mod array;

pub use array::Array;

use crate::{channel, channel::Channel};

/// Alias of `Tx<T, spsc::Array>`.
pub type Tx<T> = channel::Tx<T, Array>;
/// Alias of `Rx<T, spsc::Array>`.
pub type Rx<T> = channel::Rx<T, Array>;

/// Alias of `mpsc::Array::new(capacity).channel()`.
pub fn channel<T>(capacity: usize) -> (Tx<T>, Rx<T>) {
    Array::new(capacity).channel()
}
