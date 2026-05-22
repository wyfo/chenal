use crate::{AsyncReceiver, AsyncSender};

pub mod mpsc {
    pub use async_channel::bounded as async_channel;
}

impl<T: Send + 'static> AsyncSender<T> for async_channel::Sender<T> {
    async fn send(&mut self, msg: T) {
        (*self).send(msg).await.unwrap();
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}

impl<T: Send + 'static> AsyncReceiver<T> for async_channel::Receiver<T> {
    async fn recv(&mut self) -> T {
        (*self).recv().await.unwrap()
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}
