pub use tachyonix::channel as async_channel;

use crate::{AsyncReceiver, AsyncSender};

impl<T: Send + 'static> AsyncSender<T> for futures::channel::mpsc::Sender<T> {
    async fn send(&mut self, msg: T) {
        futures::SinkExt::send(self, msg).await.unwrap();
    }
}

impl<T: Send + 'static> AsyncReceiver<T> for futures::channel::mpsc::Receiver<T> {
    async fn recv(&mut self) -> T {
        self.recv().await.unwrap()
    }
}
