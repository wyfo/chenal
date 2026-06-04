use crate::{AsyncReceiver, AsyncSender, BlockingReceiver, BlockingSender, FutureExt as _, Receiver, Sender};

pub mod mpmc {
    pub use kanal::{bounded as blocking_channel, bounded_async as async_channel};
}

pub use mpmc as mpsc;
pub use mpmc as spmc;
pub use mpmc as spsc;

impl<T: Send + 'static> Sender<T> for kanal::Sender<T> {
    const CLONEABLE: bool = true;
    fn try_send(&mut self, msg: T) {
        (*self).try_send(msg).unwrap();
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}

impl<T: Send + 'static> BlockingSender<T> for kanal::Sender<T> {
    fn send(&mut self, msg: T) {
        (*self).send(msg).unwrap();
    }
}

impl<T: Send + 'static> Sender<T> for kanal::AsyncSender<T> {
    const CLONEABLE: bool = true;
    fn try_send(&mut self, msg: T) {
        (*self).try_send(msg).unwrap();
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}

impl<T: Send + 'static> AsyncSender<T> for kanal::AsyncSender<T> {
    fn send(&mut self, msg: T) -> impl Future<Output = ()> + Send + '_ {
        (*self).send(msg).unwrap()
    }
}

impl<T: Send + 'static> Receiver<T> for kanal::Receiver<T> {
    const CLONEABLE: bool = true;
    fn try_recv(&mut self) -> T {
        (*self).try_recv().unwrap().unwrap()
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}

impl<T: Send + 'static> BlockingReceiver<T> for kanal::Receiver<T> {
    fn recv(&mut self) -> T {
        (*self).recv().unwrap()
    }
}

impl<T: Send + 'static> Receiver<T> for kanal::AsyncReceiver<T> {
    const CLONEABLE: bool = true;
    fn try_recv(&mut self) -> T {
        (*self).try_recv().unwrap().unwrap()
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}

impl<T: Send + 'static> AsyncReceiver<T> for kanal::AsyncReceiver<T> {
    fn recv(&mut self) -> impl Future<Output = T> + Send + '_ {
        (*self).recv().unwrap()
    }
}
