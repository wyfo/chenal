pub use tokio::sync::mpsc::{channel as async_channel, channel as blocking_channel};

use crate::{AsyncReceiver, AsyncSender, BlockingReceiver, BlockingSender};

impl<T: Send + 'static> BlockingSender<T> for tokio::sync::mpsc::Sender<T> {
    fn send(&mut self, msg: T) {
        (*self).blocking_send(msg).unwrap();
    }
}

impl<T: Send + 'static> AsyncSender<T> for tokio::sync::mpsc::Sender<T> {
    async fn send(&mut self, msg: T) {
        (*self).send(msg).await.unwrap();
    }
}

impl<T: Send + 'static> BlockingReceiver<T> for tokio::sync::mpsc::Receiver<T> {
    fn recv(&mut self) -> T {
        self.blocking_recv().unwrap()
    }
}

impl<T: Send + 'static> AsyncReceiver<T> for tokio::sync::mpsc::Receiver<T> {
    async fn recv(&mut self) -> T {
        self.recv().await.unwrap()
    }
}
