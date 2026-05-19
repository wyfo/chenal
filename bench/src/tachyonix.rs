pub use tachyonix::channel as async_channel;

use crate::{AsyncReceiver, AsyncSender};

impl<T: Send + 'static> AsyncSender<T> for tachyonix::Sender<T> {
    async fn send(&mut self, msg: T) {
        (*self).send(msg).await.unwrap();
    }
}

impl<T: Send + 'static> AsyncReceiver<T> for tachyonix::Receiver<T> {
    async fn recv(&mut self) -> T {
        self.recv().await.unwrap()
    }
}
