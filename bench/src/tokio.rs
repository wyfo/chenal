use crate::{AsyncReceiver, AsyncSender, BlockingReceiver, BlockingSender, FutureExt as _, Receiver, Sender};

pub mod mpsc {
    pub use tokio::sync::mpsc::{channel as async_channel, channel as blocking_channel};
}

impl<T: Send + 'static> Sender<T> for tokio::sync::mpsc::Sender<T> {
    const CLONEABLE: bool = true;
    fn try_send(&mut self, msg: T) {
        (*self).try_send(msg).unwrap();
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}

impl<T: Send + 'static> BlockingSender<T> for tokio::sync::mpsc::Sender<T> {
    fn send(&mut self, msg: T) {
        (*self).blocking_send(msg).unwrap();
    }
}

impl<T: Send + 'static> AsyncSender<T> for tokio::sync::mpsc::Sender<T> {
    fn send(&mut self, msg: T) -> impl Future<Output = ()> + Send + '_ {
        (*self).send(msg).unwrap()
    }
}

impl<T: Send + 'static> Receiver<T> for tokio::sync::mpsc::Receiver<T> {
    const CLONEABLE: bool = false;
    fn try_recv(&mut self) -> T {
        (*self).try_recv().unwrap()
    }
    fn clone(&self) -> Self {
        unimplemented!()
    }
}

impl<T: Send + 'static> BlockingReceiver<T> for tokio::sync::mpsc::Receiver<T> {
    fn recv(&mut self) -> T {
        self.blocking_recv().unwrap()
    }
}

impl<T: Send + 'static> AsyncReceiver<T> for tokio::sync::mpsc::Receiver<T> {
    fn recv(&mut self) -> impl Future<Output = T> + Send + '_ {
        self.recv().unwrap()
    }
}
