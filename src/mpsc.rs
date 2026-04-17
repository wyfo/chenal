mod array;
mod block;

pub use array::Array;
pub use block::BlockArray;

use crate::{channel, channel::Channel};

pub type MTx<T> = channel::MTx<T, Array>;
pub type Rx<T> = channel::Rx<T, Array>;

pub fn channel<T>(capacity: usize) -> (MTx<T>, Rx<T>) {
    Array::new(capacity).channel()
}
