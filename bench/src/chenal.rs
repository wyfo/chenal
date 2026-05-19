use crate::{AsyncReceiver, AsyncSender, BlockingReceiver, BlockingSender};

pub fn channel<T>(capacity: usize) -> (chenal::mpsc::MTx<T>, chenal::mpsc::Rx<T>) {
    chenal::mpsc::channel(capacity)
}

pub use channel as async_channel;
pub use channel as blocking_channel;

impl<T: Send + 'static> BlockingSender<T> for chenal::mpsc::MTx<T> {
    fn send(&mut self, msg: T) {
        (*self).send_blocking(msg).unwrap();
    }
}

impl<T: Send + 'static> AsyncSender<T> for chenal::mpsc::MTx<T> {
    async fn send(&mut self, msg: T) {
        (*self).send(msg).await.unwrap();
    }
}

impl<T: Send + 'static> BlockingReceiver<T> for chenal::mpsc::Rx<T> {
    fn recv(&mut self) -> T {
        self.recv_blocking().unwrap()
    }
}

impl<T: Send + 'static> AsyncReceiver<T> for chenal::mpsc::Rx<T> {
    async fn recv(&mut self) -> T {
        self.recv().await.unwrap()
    }
}
