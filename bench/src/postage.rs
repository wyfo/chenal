use std::fmt::Debug;

pub use postage::mpsc::{channel as async_channel, channel as blocking_channel};
use postage::prelude::*;

use crate::{AsyncReceiver, AsyncSender, BlockingReceiver, BlockingSender};

impl<T: Send + Debug + 'static> BlockingSender<T> for postage::mpsc::Sender<T> {
    fn send(&mut self, msg: T) {
        Sink::blocking_send(self, msg).unwrap();
    }
}

impl<T: Send + Debug + 'static> AsyncSender<T> for postage::mpsc::Sender<T> {
    async fn send(&mut self, msg: T) {
        Sink::send(self, msg).await.unwrap();
    }
}

impl<T: Send + 'static> BlockingReceiver<T> for postage::mpsc::Receiver<T> {
    fn recv(&mut self) -> T {
        Stream::blocking_recv(self).unwrap()
    }
}

impl<T: Send + 'static> AsyncReceiver<T> for postage::mpsc::Receiver<T> {
    async fn recv(&mut self) -> T {
        Stream::recv(self).await.unwrap()
    }
}
