pub use kanal::{bounded as blocking_channel, bounded_async as async_channel};

use crate::{AsyncReceiver, AsyncSender, BlockingReceiver, BlockingSender};

impl<T: Send + 'static> BlockingSender<T> for kanal::Sender<T> {
    fn send(&mut self, msg: T) {
        (*self).send(msg).unwrap();
    }
}

impl<T: Send + 'static> AsyncSender<T> for kanal::AsyncSender<T> {
    async fn send(&mut self, msg: T) {
        (*self).send(msg).await.unwrap();
    }
}

impl<T: Send + 'static> BlockingReceiver<T> for kanal::Receiver<T> {
    fn recv(&mut self) -> T {
        (*self).recv().unwrap()
    }
}

impl<T: Send + 'static> AsyncReceiver<T> for kanal::AsyncReceiver<T> {
    async fn recv(&mut self) -> T {
        (*self).recv().await.unwrap()
    }
}
