pub use crossbeam_channel::bounded as blocking_channel;

use crate::{BlockingReceiver, BlockingSender};

impl<T: Send + 'static> BlockingSender<T> for crossbeam_channel::Sender<T> {
    fn send(&mut self, msg: T) {
        (*self).send(msg).unwrap();
    }
}

impl<T: Send + 'static> BlockingReceiver<T> for crossbeam_channel::Receiver<T> {
    fn recv(&mut self) -> T {
        (*self).recv().unwrap()
    }
}
