use crate::{AsyncReceiver, AsyncSender, BlockingReceiver, BlockingSender};

pub mod mpmc {
    pub use flume::{bounded as async_channel, bounded as blocking_channel};
}

impl<T: Send + 'static> BlockingSender<T> for flume::Sender<T> {
    fn send(&mut self, msg: T) {
        (*self).send(msg).unwrap();
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}

impl<T: Send + 'static> AsyncSender<T> for flume::Sender<T> {
    async fn send(&mut self, msg: T) {
        (*self).send_async(msg).await.unwrap();
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}

impl<T: Send + 'static> BlockingReceiver<T> for flume::Receiver<T> {
    fn recv(&mut self) -> T {
        (*self).recv().unwrap()
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}

impl<T: Send + 'static> AsyncReceiver<T> for flume::Receiver<T> {
    async fn recv(&mut self) -> T {
        (*self).recv_async().await.unwrap()
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}
