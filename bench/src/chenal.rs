use chenal::{backoff::BackoffStrategy, Channel};

use crate::{
    AsyncReceiver, AsyncSender, BlockingReceiver, BlockingSender, FutureExt, Receiver, Sender,
};

pub mod mpsc {
    pub use chenal::mpsc::{channel as async_channel, channel as blocking_channel};
}

pub mod mpmc {
    pub use chenal::mpmc::{channel as async_channel, channel as blocking_channel};
}

pub mod mpmc_racy {
    pub use channel as async_channel;
    pub use channel as blocking_channel;
    use chenal::{mpmc::RacyArray, Channel, MRx, MTx};

    pub fn channel<T>(capacity: usize) -> (MTx<T, RacyArray>, MRx<T, RacyArray>) {
        RacyArray::new(capacity).channel()
    }
}

pub mod spmc {
    pub use chenal::spmc::{channel as async_channel, channel as blocking_channel};
}

pub mod spsc {
    pub use chenal::spsc::{channel as async_channel, channel as blocking_channel};
}

impl<T: Send + 'static, Ch: Channel> Sender<T> for chenal::Tx<T, Ch> {
    const CLONEABLE: bool = false;
    fn try_send(&mut self, msg: T) {
        (*self).try_send(msg).unwrap();
    }
    fn clone(&self) -> Self {
        unimplemented!()
    }
}

impl<T: Send + 'static, Ch: Channel> BlockingSender<T> for chenal::Tx<T, Ch> {
    fn send(&mut self, msg: T) {
        (*self).send_blocking(msg).unwrap();
    }
}

impl<T: Send + 'static, Ch: Channel> AsyncSender<T> for chenal::Tx<T, Ch> {
    fn send(&mut self, msg: T) -> impl Future<Output = ()> + Send + '_ {
        (*self).send(msg).unwrap()
    }
}

impl<T: Send + 'static, Ch: Channel, B: BackoffStrategy> Sender<T> for chenal::MTx<T, Ch, B> {
    const CLONEABLE: bool = true;
    fn try_send(&mut self, msg: T) {
        (*self).try_send(msg).unwrap();
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}

impl<T: Send + 'static, Ch: Channel, B: BackoffStrategy> BlockingSender<T>
    for chenal::MTx<T, Ch, B>
{
    fn send(&mut self, msg: T) {
        (*self).send_blocking(msg).unwrap();
    }
}

impl<T: Send + 'static, Ch: Channel, B: BackoffStrategy> AsyncSender<T> for chenal::MTx<T, Ch, B> {
    fn send(&mut self, msg: T) -> impl Future<Output = ()> + Send + '_ {
        (*self).send(msg).unwrap()
    }
}

impl<T: Send + 'static, Ch: Channel> Receiver<T> for chenal::Rx<T, Ch> {
    const CLONEABLE: bool = false;
    fn try_recv(&mut self) -> T {
        (*self).try_recv().unwrap()
    }
    fn clone(&self) -> Self {
        unimplemented!()
    }
}

impl<T: Send + 'static, Ch: Channel> BlockingReceiver<T> for chenal::Rx<T, Ch> {
    fn recv(&mut self) -> T {
        (*self).recv_blocking().unwrap()
    }
}

impl<T: Send + 'static, Ch: Channel> AsyncReceiver<T> for chenal::Rx<T, Ch> {
    fn recv(&mut self) -> impl Future<Output = T> + Send + '_ {
        (*self).recv().unwrap()
    }
}

impl<T: Send + 'static, Ch: Channel, B: BackoffStrategy> Receiver<T> for chenal::MRx<T, Ch, B> {
    const CLONEABLE: bool = true;
    fn try_recv(&mut self) -> T {
        (*self).try_recv().unwrap()
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}

impl<T: Send + 'static, Ch: Channel, B: BackoffStrategy> BlockingReceiver<T>
    for chenal::MRx<T, Ch, B>
{
    fn recv(&mut self) -> T {
        (*self).recv_blocking().unwrap()
    }
}

impl<T: Send + 'static, Ch: Channel, B: BackoffStrategy> AsyncReceiver<T>
    for chenal::MRx<T, Ch, B>
{
    fn recv(&mut self) -> impl Future<Output = T> + Send + '_ {
        (*self).recv().unwrap()
    }
}
