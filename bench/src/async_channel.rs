use crate::{AsyncReceiver, AsyncSender, FutureExt as _, Receiver, Sender};

pub mod mpmc {
    pub use async_channel::bounded as async_channel;
}

impl<T: Send + 'static> Sender<T> for async_channel::Sender<T> {
    const CLONEABLE: bool = true;
    fn try_send(&mut self, msg: T) {
        (*self).try_send(msg).unwrap();
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}

impl<T: Send + 'static> AsyncSender<T> for async_channel::Sender<T> {
    fn send(&mut self, msg: T) -> impl Future<Output = ()> + Send + '_ {
        (*self).send(msg).unwrap()
    }
}

impl<T: Send + 'static> Receiver<T> for async_channel::Receiver<T> {
    const CLONEABLE: bool = true;
    fn try_recv(&mut self) -> T {
        (*self).try_recv().unwrap()
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}

impl<T: Send + 'static> AsyncReceiver<T> for async_channel::Receiver<T> {
    fn recv(&mut self) -> impl Future<Output = T> + Send + '_ {
        (*self).recv().unwrap()
    }
}
