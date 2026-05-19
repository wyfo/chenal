pub use crossfire::mpsc::{bounded_async as async_channel, bounded_blocking as blocking_channel};

use crate::{AsyncReceiver, AsyncSender, BlockingReceiver, BlockingSender};

impl<T: Send + 'static> BlockingSender<T> for crossfire::MTx<crossfire::mpsc::Array<T>> {
    fn send(&mut self, msg: T) {
        crossfire::BlockingTxTrait::send(self, msg).unwrap();
    }
}

impl<T: Send + Unpin + 'static> AsyncSender<T> for crossfire::MAsyncTx<crossfire::mpsc::Array<T>> {
    async fn send(&mut self, msg: T) {
        crossfire::AsyncTxTrait::send(self, msg).await.unwrap();
    }
}

impl<T: Send + 'static> BlockingReceiver<T> for crossfire::Rx<crossfire::mpsc::Array<T>> {
    fn recv(&mut self) -> T {
        (*self).recv().unwrap()
    }
}

impl<T: Send + Unpin + 'static> AsyncReceiver<T> for crossfire::AsyncRx<crossfire::mpsc::Array<T>> {
    async fn recv(&mut self) -> T {
        (*self).recv().await.unwrap()
    }
}
