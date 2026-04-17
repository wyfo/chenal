#[cfg(feature = "blocking")]
use crate::blocking::*;
use crate::{
    channel::{ArcChan, Channel},
    errors::{SendError, TrySendError},
    macros::channel_end,
};

pub(crate) trait TxState<T> {
    fn capacity(&self) -> Option<usize>;
    fn is_closed(&self) -> bool;
    fn close(&self) -> bool;
}

pub struct Tx<T, Ch: Channel> {
    chan: ArcChan<T, Ch>,
}
channel_end!(Tx);

impl<T, Ch: Channel> Tx<T, Ch> {
    pub fn try_send(&mut self, msg: T) -> Result<(), TrySendError<T>> {
        todo!()
    }

    pub async fn send(&mut self, msg: T) -> Result<(), SendError<T>> {
        todo!()
    }

    #[cfg(feature = "blocking")]
    pub fn send_blocking(&mut self, msg: T) -> Result<(), SendError<T>> {
        todo!()
    }

    #[cfg(feature = "blocking")]
    pub fn send_deadline(&mut self, msg: T, deadline: Instant) -> Result<(), SendTimeoutError<T>> {
        todo!()
    }

    #[cfg(feature = "blocking")]
    pub fn send_timeout(&mut self, msg: T, timeout: Duration) -> Result<(), SendTimeoutError<T>> {
        todo!()
    }
}

pub struct UTx<T, Ch: Channel> {
    chan: ArcChan<T, Ch>,
}
channel_end!(UTx);

impl<T, Ch: Channel> UTx<T, Ch> {
    pub async fn send(&mut self, msg: T) -> Result<(), SendError<T>> {
        todo!()
    }
}

pub struct MTx<T, Ch: Channel> {
    chan: ArcChan<T, Ch>,
}
channel_end!(MTx);

impl<T, Ch: Channel> MTx<T, Ch> {
    pub fn try_send(&self, msg: T) -> Result<(), TrySendError<T>> {
        todo!()
    }

    pub async fn send(&self, msg: T) -> Result<(), SendError<T>> {
        todo!()
    }

    #[cfg(feature = "blocking")]
    pub fn send_blocking(&self, msg: T) -> Result<(), SendError<T>> {
        todo!()
    }

    #[cfg(feature = "blocking")]
    pub fn send_deadline(&self, msg: T, deadline: Instant) -> Result<(), SendTimeoutError<T>> {
        todo!()
    }

    #[cfg(feature = "blocking")]
    pub fn send_timeout(&self, msg: T, timeout: Duration) -> Result<(), SendTimeoutError<T>> {
        todo!()
    }
}

pub struct UMTx<T, Ch: Channel> {
    chan: ArcChan<T, Ch>,
}
channel_end!(UMTx);

impl<T, Ch: Channel> UMTx<T, Ch> {
    pub fn send(&self, msg: T) -> Result<(), SendError<T>> {
        todo!()
    }
}
