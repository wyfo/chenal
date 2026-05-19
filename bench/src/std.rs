pub use std::sync::mpsc::sync_channel as blocking_channel;

use crate::{BlockingReceiver, BlockingSender};

impl<T: Send + 'static> BlockingSender<T> for std::sync::mpsc::SyncSender<T> {
    fn send(&mut self, msg: T) {
        (*self).send(msg).unwrap();
    }
}

impl<T: Send + 'static> BlockingReceiver<T> for std::sync::mpsc::Receiver<T> {
    fn recv(&mut self) -> T {
        (*self).recv().unwrap()
    }
}
