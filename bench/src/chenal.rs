use chenal::Channel;

use crate::{AsyncReceiver, AsyncSender, BlockingReceiver, BlockingSender};

pub mod mpsc {
    pub use chenal::mpsc::{channel as async_channel, channel as blocking_channel};
}

pub mod mpmc {
    pub use chenal::mpmc::{channel as async_channel, channel as blocking_channel};
}

pub mod spsc {
    pub use chenal::spsc::{channel as async_channel, channel as blocking_channel};
}

impl<T: Send + 'static, Ch: Channel> BlockingSender<T> for chenal::Tx<T, Ch> {
    fn send(&mut self, msg: T) {
        (*self).send_blocking(msg).unwrap();
    }
    fn clone(&self) -> Self {
        unimplemented!()
    }
}

impl<T: Send + 'static, Ch: Channel> AsyncSender<T> for chenal::Tx<T, Ch> {
    async fn send(&mut self, msg: T) {
        (*self).send(msg).await.unwrap();
    }
    fn clone(&self) -> Self {
        unimplemented!()
    }
}

impl<T: Send + 'static, Ch: Channel> BlockingSender<T> for chenal::MTx<T, Ch> {
    fn send(&mut self, msg: T) {
        (*self).send_blocking(msg).unwrap();
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}

impl<T: Send + 'static, Ch: Channel> AsyncSender<T> for chenal::MTx<T, Ch> {
    async fn send(&mut self, msg: T) {
        (*self).send(msg).await.unwrap();
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}

impl<T: Send + 'static, Ch: Channel> BlockingReceiver<T> for chenal::Rx<T, Ch> {
    fn recv(&mut self) -> T {
        (*self).recv_blocking().unwrap()
    }
    fn clone(&self) -> Self {
        unimplemented!()
    }
}

impl<T: Send + 'static, Ch: Channel> AsyncReceiver<T> for chenal::Rx<T, Ch> {
    async fn recv(&mut self) -> T {
        (*self).recv().await.unwrap()
    }
    fn clone(&self) -> Self {
        unimplemented!()
    }
}

impl<T: Send + 'static, Ch: Channel> BlockingReceiver<T> for chenal::MRx<T, Ch> {
    fn recv(&mut self) -> T {
        (*self).recv_blocking().unwrap()
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}

impl<T: Send + 'static, Ch: Channel> AsyncReceiver<T> for chenal::MRx<T, Ch> {
    async fn recv(&mut self) -> T {
        (*self).recv().await.unwrap()
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}
