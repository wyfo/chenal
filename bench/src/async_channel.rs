use crate::{AsyncReceiver, AsyncSender, Receiver, Sender};

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
    async fn send(&mut self, msg: T) {
        (*self).send(msg).await.unwrap();
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
    async fn recv(&mut self) -> T {
        (*self).recv().await.unwrap()
    }
}
