use crate::{AsyncReceiver, AsyncSender, BlockingReceiver, BlockingSender};

pub struct MTx<T>(chenal::mpsc::MTx<T>);
impl<T> Clone for MTx<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
pub struct Rx<T>(chenal::mpsc::Rx<T>);

pub fn channel<T>(capacity: usize) -> (MTx<T>, Rx<T>) {
    let (tx, rx) = chenal::mpsc::channel(capacity);
    (MTx(tx), Rx(rx))
}

pub use channel as async_channel;
pub use channel as blocking_channel;

impl<T: Send + 'static> BlockingSender<T> for MTx<T> {
    fn send(&mut self, mut msg: T) {
        let backoff = crossbeam_utils::Backoff::new();
        while !backoff.is_completed() {
            match self.0.try_send(msg) {
                Ok(_) => return,
                Err(chenal::errors::TrySendError::Full(m)) => msg = m,
                _ => unreachable!(),
            }
            backoff.snooze();
        }
        self.0.send_blocking(msg).unwrap();
    }
}

impl<T: Send + 'static> AsyncSender<T> for MTx<T> {
    async fn send(&mut self, mut msg: T) {
        let backoff = crossbeam_utils::Backoff::new();
        while !backoff.is_completed() {
            match self.0.try_send(msg) {
                Ok(_) => return,
                Err(chenal::errors::TrySendError::Full(m)) => msg = m,
                _ => unreachable!(),
            }
            backoff.snooze();
        }
        self.0.send(msg).await.unwrap();
    }
}

impl<T: Send + 'static> BlockingReceiver<T> for Rx<T> {
    fn recv(&mut self) -> T {
        let backoff = crossbeam_utils::Backoff::new();
        while !backoff.is_completed() {
            match self.0.try_recv() {
                Ok(msg) => return msg,
                Err(chenal::errors::TryRecvError::Empty) => {}
                _ => unreachable!(),
            }
            backoff.snooze();
        }
        self.0.recv_blocking().unwrap()
    }
}

impl<T: Send + 'static> AsyncReceiver<T> for Rx<T> {
    async fn recv(&mut self) -> T {
        let backoff = crossbeam_utils::Backoff::new();
        while !backoff.is_completed() {
            match self.0.try_recv() {
                Ok(msg) => return msg,
                Err(chenal::errors::TryRecvError::Empty) => {}
                _ => unreachable!(),
            }
            backoff.snooze();
        }
        self.0.recv().await.unwrap()
    }
}
