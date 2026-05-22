use crate::{AsyncReceiver, AsyncSender};

pub mod mpsc {
    pub use futures::channel::mpsc::channel as async_channel;
}

impl<T: Send + 'static> AsyncSender<T> for futures::channel::mpsc::Sender<T> {
    async fn send(&mut self, msg: T) {
        futures::SinkExt::send(self, msg).await.unwrap();
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}

impl<T: Send + 'static> AsyncReceiver<T> for futures::channel::mpsc::Receiver<T> {
    async fn recv(&mut self) -> T {
        self.recv().await.unwrap()
    }
    fn clone(&self) -> Self {
        unimplemented!()
    }
}
