//! Same as [`crate::chenal_32`] but with exponential unbounded backoff,
//! independently of the target platform. Only the MPSC channel has an unbounded
//! backoff parameter; the SPSC channel is reused from [`crate::chenal_32`].

const BLOCK_SIZE: usize = 32;

pub use crate::chenal_32::spsc;

pub mod mpsc {
    pub use channel as async_channel;
    pub use channel as blocking_channel;
    use chenal::{backoff::ExponentialBackoff, mpsc::Array, Channel, MTx, Rx};

    use super::BLOCK_SIZE;

    type Ch = Array<BLOCK_SIZE, usize, ExponentialBackoff<6, true>>;

    pub fn channel<T>(capacity: usize) -> (MTx<T, Ch>, Rx<T, Ch>) {
        Array::new(capacity).channel()
    }
}
