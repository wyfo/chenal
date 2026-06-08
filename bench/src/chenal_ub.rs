//! Same as [`crate::chenal`] but with exponential unbounded backoff, independently
//! of the target platform. Only the MPSC and MPMC channels have an unbounded
//! backoff parameter; the SPMC/SPSC channels are reused from [`crate::chenal`].
//! The blanket trait impls in [`crate::chenal`] cover the channel types
//! constructed below.

pub use crate::chenal::{spmc, spsc};

pub mod mpsc {
    pub use channel as async_channel;
    pub use channel as blocking_channel;
    use chenal::{Channel, MTx, Rx, backoff::ExponentialBackoff, mpsc::Array};

    type Ch = Array<1, usize, ExponentialBackoff<6, true>>;

    pub fn channel<T>(capacity: usize) -> (MTx<T, Ch>, Rx<T, Ch>) {
        Array::new(capacity).channel()
    }
}

pub mod mpmc {
    pub use channel as async_channel;
    pub use channel as blocking_channel;
    use chenal::{Channel, MRx, MTx, backoff::ExponentialBackoff, mpmc::Array};

    type Ch = Array<usize, ExponentialBackoff<6, true>>;

    pub fn channel<T>(capacity: usize) -> (MTx<T, Ch>, MRx<T, Ch>) {
        Array::new(capacity).channel()
    }
}
