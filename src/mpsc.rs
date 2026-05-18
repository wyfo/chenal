//! MPSC channel implementations.
mod array;
mod block;

pub use array::Array;
pub use block::BlockArray;

use crate::{channel, channel::Channel};

/// Alias of `MTx<T, mpsc::Array>`.
pub type MTx<T> = channel::MTx<T, Array>;
/// Alias of `Rx<T, mpsc::Array>`.
pub type Rx<T> = channel::Rx<T, Array>;

/// Alias of `mpsc::Array::new(capacity).channel()`.
pub fn channel<T>(capacity: usize) -> (MTx<T>, Rx<T>) {
    Array::new(capacity).channel()
}
