const BLOCK_SIZE: usize = 32;

pub mod mpsc {
    pub use channel as async_channel;
    pub use channel as blocking_channel;
    use chenal::{backoff::NoBackoff, mpsc::Array, Channel, MTx, Rx};

    use super::BLOCK_SIZE;

    type Ch = Array<BLOCK_SIZE, usize, NoBackoff>;

    pub fn channel<T>(capacity: usize) -> (MTx<T, Ch>, Rx<T, Ch>) {
        Array::new(capacity).channel()
    }
}

pub mod spsc {
    pub use channel as async_channel;
    pub use channel as blocking_channel;
    use chenal::{spsc::Array, Channel, Rx, Tx};

    use super::BLOCK_SIZE;

    pub fn channel<T>(capacity: usize) -> (Tx<T, Array<BLOCK_SIZE>>, Rx<T, Array<BLOCK_SIZE>>) {
        Array::new(capacity).channel()
    }
}
