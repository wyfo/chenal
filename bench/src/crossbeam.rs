use crate::{BlockingReceiver, BlockingSender};

pub mod mpmc {
    pub use crossbeam_channel::bounded as blocking_channel;
}

impl<T: Send + 'static> BlockingSender<T> for crossbeam_channel::Sender<T> {
    fn send(&mut self, msg: T) {
        (*self).send(msg).unwrap();
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}

impl<T: Send + 'static> BlockingReceiver<T> for crossbeam_channel::Receiver<T> {
    fn recv(&mut self) -> T {
        (*self).recv().unwrap()
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}
