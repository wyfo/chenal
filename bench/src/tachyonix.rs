use crate::{AsyncReceiver, AsyncSender, Receiver, Sender};

pub mod mpsc {
    pub use tachyonix::channel as async_channel;
}

impl<T: Send + 'static> Sender<T> for tachyonix::Sender<T> {
    const CLONEABLE: bool = true;
    fn try_send(&mut self, msg: T) {
        (*self).try_send(msg).unwrap_or_else(|_| panic!("try_send failed"));
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}

impl<T: Send + 'static> AsyncSender<T> for tachyonix::Sender<T> {
    async fn send(&mut self, msg: T) {
        (*self).send(msg).await.unwrap();
    }
}

impl<T: Send + 'static> Receiver<T> for tachyonix::Receiver<T> {
    const CLONEABLE: bool = false;
    fn try_recv(&mut self) -> T {
        self.try_recv().unwrap()
    }
    fn clone(&self) -> Self {
        unimplemented!()
    }
}

impl<T: Send + 'static> AsyncReceiver<T> for tachyonix::Receiver<T> {
    async fn recv(&mut self) -> T {
        self.recv().await.unwrap()
    }
}
