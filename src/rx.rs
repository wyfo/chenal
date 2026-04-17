#[cfg(feature = "blocking")]
use crate::blocking::*;
use crate::{
    channel::{ArcChan, Channel},
    errors::{RecvError, TryRecvError},
    macros::channel_end,
};

pub(crate) trait RxState<T> {
    fn capacity(&self) -> Option<usize>;
    fn is_closed(&self) -> bool;
    fn close(&self) -> bool;
}

pub struct Rx<T, Ch: Channel> {
    chan: ArcChan<T, Ch>,
}
channel_end!(Rx);

impl<T, Ch: Channel> Rx<T, Ch> {
    pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
        todo!()
    }

    pub async fn recv(&mut self) -> Result<T, RecvError> {
        todo!()
    }

    #[cfg(feature = "blocking")]
    pub fn recv_blocking(&mut self) -> Result<T, RecvError> {
        todo!()
    }

    #[cfg(feature = "blocking")]
    pub fn recv_deadline(&mut self, deadline: Instant) -> Result<T, RecvTimeoutError> {
        todo!()
    }

    #[cfg(feature = "blocking")]
    pub fn recv_timeout(&mut self, timeout: Duration) -> Result<T, RecvError> {
        todo!()
    }
}

pub struct MRx<T, Ch: Channel> {
    chan: ArcChan<T, Ch>,
}
channel_end!(MRx);

impl<T, Ch: Channel> MRx<T, Ch> {
    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        todo!()
    }

    pub async fn recv(&self) -> Result<T, RecvError> {
        todo!()
    }

    #[cfg(feature = "blocking")]
    pub fn recv_blocking(&self) -> Result<T, RecvError> {
        todo!()
    }

    #[cfg(feature = "blocking")]
    pub fn recv_deadline(&self, deadline: Instant) -> Result<T, RecvTimeoutError> {
        todo!()
    }

    #[cfg(feature = "blocking")]
    pub fn recv_timeout(&self, timeout: Duration) -> Result<T, RecvTimeoutError> {
        todo!()
    }
}
