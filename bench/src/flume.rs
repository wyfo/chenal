use crate::{AsyncReceiver, AsyncSender, BlockingReceiver, BlockingSender, Receiver, Sender};

pub mod mpmc {
    pub use flume::{bounded as async_channel, bounded as blocking_channel};
}

impl<T: Send + 'static> Sender<T> for flume::Sender<T> {
    const CLONEABLE: bool = true;
    fn try_send(&mut self, msg: T) {
        (*self).try_send(msg).unwrap();
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}

impl<T: Send + 'static> BlockingSender<T> for flume::Sender<T> {
    fn send(&mut self, msg: T) {
        (*self).send(msg).unwrap();
    }
}

impl<T: Send + 'static> AsyncSender<T> for flume::Sender<T> {
    async fn send(&mut self, msg: T) {
        (*self).send_async(msg).await.unwrap();
    }
}

impl<T: Send + 'static> Receiver<T> for flume::Receiver<T> {
    const CLONEABLE: bool = true;
    fn try_recv(&mut self) -> T {
        (*self).try_recv().unwrap()
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}

impl<T: Send + 'static> BlockingReceiver<T> for flume::Receiver<T> {
    fn recv(&mut self) -> T {
        (*self).recv().unwrap()
    }
}

impl<T: Send + 'static> AsyncReceiver<T> for flume::Receiver<T> {
    async fn recv(&mut self) -> T {
        (*self).recv_async().await.unwrap()
    }
}
