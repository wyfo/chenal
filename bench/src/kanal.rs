use crate::{AsyncReceiver, AsyncSender, BlockingReceiver, BlockingSender};

pub mod mpmc {
    pub use kanal::{bounded as blocking_channel, bounded_async as async_channel};
}

impl<T: Send + 'static> BlockingSender<T> for kanal::Sender<T> {
    fn send(&mut self, msg: T) {
        (*self).send(msg).unwrap();
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}

impl<T: Send + 'static> AsyncSender<T> for kanal::AsyncSender<T> {
    async fn send(&mut self, msg: T) {
        (*self).send(msg).await.unwrap();
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}

impl<T: Send + 'static> BlockingReceiver<T> for kanal::Receiver<T> {
    fn recv(&mut self) -> T {
        (*self).recv().unwrap()
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}

impl<T: Send + 'static> AsyncReceiver<T> for kanal::AsyncReceiver<T> {
    async fn recv(&mut self) -> T {
        (*self).recv().await.unwrap()
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}
