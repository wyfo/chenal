use std::fmt::Debug;

use postage::prelude::*;

use crate::{AsyncReceiver, AsyncSender, BlockingReceiver, BlockingSender};

pub mod mpsc {
    pub use postage::mpsc::{channel as async_channel, channel as blocking_channel};
}

impl<T: Send + Debug + 'static> BlockingSender<T> for postage::mpsc::Sender<T> {
    fn send(&mut self, msg: T) {
        Sink::blocking_send(self, msg).unwrap();
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}

impl<T: Send + Debug + 'static> AsyncSender<T> for postage::mpsc::Sender<T> {
    async fn send(&mut self, msg: T) {
        Sink::send(self, msg).await.unwrap();
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}

impl<T: Send + 'static> BlockingReceiver<T> for postage::mpsc::Receiver<T> {
    fn recv(&mut self) -> T {
        Stream::blocking_recv(self).unwrap()
    }
    fn clone(&self) -> Self {
        unimplemented!()
    }
}

impl<T: Send + 'static> AsyncReceiver<T> for postage::mpsc::Receiver<T> {
    async fn recv(&mut self) -> T {
        Stream::recv(self).await.unwrap()
    }
    fn clone(&self) -> Self {
        unimplemented!()
    }
}
