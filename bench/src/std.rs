use crate::{BlockingReceiver, BlockingSender, Receiver, Sender};

pub mod mpsc {
    pub use std::sync::mpsc::sync_channel as blocking_channel;
}

impl<T: Send + 'static> Sender<T> for std::sync::mpsc::SyncSender<T> {
    const CLONEABLE: bool = true;
    fn try_send(&mut self, msg: T) {
        (*self).try_send(msg).unwrap();
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}

impl<T: Send + 'static> BlockingSender<T> for std::sync::mpsc::SyncSender<T> {
    fn send(&mut self, msg: T) {
        (*self).send(msg).unwrap();
    }
}

impl<T: Send + 'static> Receiver<T> for std::sync::mpsc::Receiver<T> {
    const CLONEABLE: bool = false;
    fn try_recv(&mut self) -> T {
        (*self).try_recv().unwrap()
    }
    fn clone(&self) -> Self {
        unimplemented!()
    }
}

impl<T: Send + 'static> BlockingReceiver<T> for std::sync::mpsc::Receiver<T> {
    fn recv(&mut self) -> T {
        (*self).recv().unwrap()
    }
}
