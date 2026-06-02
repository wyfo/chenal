use chenal::Channel;

use crate::{AsyncReceiver, AsyncSender, BlockingReceiver, BlockingSender, Receiver, Sender};

pub struct Tx<T, Ch: Channel>(chenal::Tx<T, Ch>);
pub struct MTx<T, Ch: Channel>(chenal::MTx<T, Ch>);
pub struct Rx<T, Ch: Channel>(chenal::Rx<T, Ch>);
pub struct MRx<T, Ch: Channel>(chenal::MRx<T, Ch>);

pub mod mpmc {
    pub use channel as async_channel;
    pub use channel as blocking_channel;
    use chenal::mpmc::Array;

    use super::{MRx, MTx};

    pub fn channel<T>(capacity: usize) -> (MTx<T, Array>, MRx<T, Array>) {
        let (tx, rx) = chenal::mpmc::channel(capacity);
        (MTx(tx), MRx(rx))
    }
}

pub mod mpmc_racy {
    pub use channel as async_channel;
    pub use channel as blocking_channel;
    use chenal::{mpmc::RacyArray, Channel};

    use super::{MRx, MTx};

    pub fn channel<T>(capacity: usize) -> (MTx<T, RacyArray>, MRx<T, RacyArray>) {
        let (tx, rx) = RacyArray::new(capacity).channel();
        (MTx(tx), MRx(rx))
    }
}

pub mod mpsc {
    pub use channel as async_channel;
    pub use channel as blocking_channel;
    use chenal::mpsc::Array;

    use super::{MTx, Rx};

    pub fn channel<T>(capacity: usize) -> (MTx<T, Array>, Rx<T, Array>) {
        let (tx, rx) = chenal::mpsc::channel(capacity);
        (MTx(tx), Rx(rx))
    }
}

pub mod spmc {
    pub use channel as async_channel;
    pub use channel as blocking_channel;
    use chenal::spmc::Array;

    use super::{MRx, Tx};

    pub fn channel<T>(capacity: usize) -> (Tx<T, Array>, MRx<T, Array>) {
        let (tx, rx) = chenal::spmc::channel(capacity);
        (Tx(tx), MRx(rx))
    }
}

pub mod spsc {
    pub use channel as async_channel;
    pub use channel as blocking_channel;
    use chenal::spsc::Array;

    use super::{Rx, Tx};

    pub fn channel<T>(capacity: usize) -> (Tx<T, Array>, Rx<T, Array>) {
        let (tx, rx) = chenal::spsc::channel(capacity);
        (Tx(tx), Rx(rx))
    }
}

impl<T: Send + 'static, Ch: Channel> Sender<T> for Tx<T, Ch> {
    const CLONEABLE: bool = false;
    fn try_send(&mut self, msg: T) {
        self.0.try_send(msg).unwrap();
    }
    fn clone(&self) -> Self {
        unimplemented!()
    }
}

impl<T: Send + 'static, Ch: Channel> BlockingSender<T> for Tx<T, Ch> {
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

impl<T: Send + 'static, Ch: Channel> AsyncSender<T> for Tx<T, Ch> {
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

impl<T: Send + 'static, Ch: Channel> Sender<T> for MTx<T, Ch> {
    const CLONEABLE: bool = true;
    fn try_send(&mut self, msg: T) {
        self.0.try_send(msg).unwrap();
    }
    fn clone(&self) -> Self {
        Self(Clone::clone(&self.0))
    }
}

impl<T: Send + 'static, Ch: Channel> BlockingSender<T> for MTx<T, Ch> {
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

impl<T: Send + 'static, Ch: Channel> AsyncSender<T> for MTx<T, Ch> {
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

impl<T: Send + 'static, Ch: Channel> Receiver<T> for Rx<T, Ch> {
    const CLONEABLE: bool = false;
    fn try_recv(&mut self) -> T {
        self.0.try_recv().unwrap()
    }
    fn clone(&self) -> Self {
        unimplemented!()
    }
}

impl<T: Send + 'static, Ch: Channel> BlockingReceiver<T> for Rx<T, Ch> {
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

impl<T: Send + 'static, Ch: Channel> AsyncReceiver<T> for Rx<T, Ch> {
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

impl<T: Send + 'static, Ch: Channel> Receiver<T> for MRx<T, Ch> {
    const CLONEABLE: bool = true;
    fn try_recv(&mut self) -> T {
        self.0.try_recv().unwrap()
    }
    fn clone(&self) -> Self {
        Self(Clone::clone(&self.0))
    }
}

impl<T: Send + 'static, Ch: Channel> BlockingReceiver<T> for MRx<T, Ch> {
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

impl<T: Send + 'static, Ch: Channel> AsyncReceiver<T> for MRx<T, Ch> {
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
