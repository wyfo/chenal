use crate::{AsyncReceiver, AsyncSender, FutureExt as _, Receiver, Sender};

pub mod mpsc {
    pub use futures::channel::mpsc::channel as async_channel;
}

impl<T: Send + 'static> Sender<T> for futures::channel::mpsc::Sender<T> {
    const CLONEABLE: bool = true;
    fn try_send(&mut self, msg: T) {
        futures::channel::mpsc::Sender::try_send(self, msg).unwrap();
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}

impl<T: Send + 'static> AsyncSender<T> for futures::channel::mpsc::Sender<T> {
    fn send(&mut self, msg: T) -> impl Future<Output = ()> + Send + '_ {
        futures::SinkExt::send(self, msg).unwrap()
    }
}

impl<T: Send + 'static> Receiver<T> for futures::channel::mpsc::Receiver<T> {
    const CLONEABLE: bool = false;
    fn try_recv(&mut self) -> T {
        self.try_recv().unwrap()
    }
    fn clone(&self) -> Self {
        unimplemented!()
    }
}

impl<T: Send + 'static> AsyncReceiver<T> for futures::channel::mpsc::Receiver<T> {
    fn recv(&mut self) -> impl Future<Output = T> + Send + '_ {
        self.recv().unwrap()
    }
}
