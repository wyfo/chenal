use std::fmt::Debug;

use futures::FutureExt;
use postage::prelude::*;

use crate::{
    AsyncReceiver, AsyncSender, BlockingReceiver, BlockingSender, FutureExt as _, Receiver, Sender,
};

pub mod mpsc {
    pub use postage::mpsc::{channel as async_channel, channel as blocking_channel};
}

impl<T: Send + Debug + 'static> Sender<T> for postage::mpsc::Sender<T> {
    const CLONEABLE: bool = true;
    fn try_send(&mut self, msg: T) {
        Sink::try_send(self, msg).unwrap();
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}

impl<T: Send + Debug + 'static> BlockingSender<T> for postage::mpsc::Sender<T> {
    fn send(&mut self, msg: T) {
        Sink::blocking_send(self, msg).unwrap();
    }
}

impl<T: Send + Debug + 'static> AsyncSender<T> for postage::mpsc::Sender<T> {
    fn send(&mut self, msg: T) -> impl Future<Output = ()> + Send + '_ {
        Sink::send(self, msg).unwrap()
    }
}

impl<T: Send + 'static> Receiver<T> for postage::mpsc::Receiver<T> {
    const CLONEABLE: bool = false;
    fn try_recv(&mut self) -> T {
        Stream::recv(self).now_or_never().unwrap().unwrap()
    }
    fn clone(&self) -> Self {
        unimplemented!()
    }
}

impl<T: Send + 'static> BlockingReceiver<T> for postage::mpsc::Receiver<T> {
    fn recv(&mut self) -> T {
        Stream::blocking_recv(self).unwrap()
    }
}

impl<T: Send + 'static> AsyncReceiver<T> for postage::mpsc::Receiver<T> {
    fn recv(&mut self) -> impl Future<Output = T> + Send + '_ {
        Stream::recv(self).unwrap()
    }
}
