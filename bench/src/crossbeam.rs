use crate::{BlockingReceiver, BlockingSender, Receiver, Sender};

pub mod mpmc {
    pub use crossbeam_channel::bounded as blocking_channel;
}

impl<T: Send + 'static> Sender<T> for crossbeam_channel::Sender<T> {
    const CLONEABLE: bool = true;
    fn try_send(&mut self, msg: T) {
        (*self).try_send(msg).unwrap();
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}

impl<T: Send + 'static> BlockingSender<T> for crossbeam_channel::Sender<T> {
    fn send(&mut self, msg: T) {
        (*self).send(msg).unwrap();
    }
}

impl<T: Send + 'static> Receiver<T> for crossbeam_channel::Receiver<T> {
    const CLONEABLE: bool = true;
    fn try_recv(&mut self) -> T {
        (*self).try_recv().unwrap()
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}

impl<T: Send + 'static> BlockingReceiver<T> for crossbeam_channel::Receiver<T> {
    fn recv(&mut self) -> T {
        (*self).recv().unwrap()
    }
}
