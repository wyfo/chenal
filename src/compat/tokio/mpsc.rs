use alloc::vec::Vec;
use core::{
    fmt,
    pin::pin,
    task::{ready, Context, Poll},
};

use crate::{
    compat::tokio::mpsc::error::TryRecvError,
    errors::{SendError, TrySendError},
    mpsc,
    Weak,
};

pub mod error {
    #[cfg(feature = "compat-tokio-time")]
    use core::fmt;

    pub use crate::errors::{SendError, TrySendError};
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum TryRecvError {
        Empty,
        Disconnected,
    }
    #[cfg(feature = "compat-tokio-time")]
    #[derive(Clone, Copy, Eq, PartialEq)]
    pub enum SendTimeoutError<T> {
        Timeout(T),
        Closed(T),
    }
    #[cfg(feature = "compat-tokio-time")]
    impl<T> fmt::Debug for SendTimeoutError<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Timeout(_) => f.debug_tuple("Timeout").finish_non_exhaustive(),
                Self::Closed(_) => f.debug_tuple("Closed").finish_non_exhaustive(),
            }
        }
    }
}

pub struct Sender<T>(mpsc::MTx<T>);
pub struct Receiver<T>(mpsc::Rx<T>);
pub struct WeakSender<T>(Weak<mpsc::MTx<T>>);

pub fn channel<T>(buffer: usize) -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = mpsc::channel(buffer);
    (Sender(tx), Receiver(rx))
}

impl<T> Sender<T> {
    pub async fn send(&self, value: T) -> Result<(), SendError<T>> {
        self.0.send(value).await
    }

    pub fn try_send(&self, message: T) -> Result<(), TrySendError<T>> {
        self.0.try_send(message)
    }

    #[cfg(feature = "blocking")]
    pub fn blocking_send(&self, value: T) -> Result<(), SendError<T>> {
        assert!(
            tokio::runtime::Handle::try_current().is_err(),
            "blocking_send called from async context"
        );
        self.0.send_blocking(value)
    }

    pub fn is_closed(&self) -> bool {
        self.0.is_closed()
    }

    pub async fn closed(&self) {
        self.0.closed().await;
    }

    pub fn max_capacity(&self) -> usize {
        self.0.capacity()
    }

    pub fn same_channel(&self, other: &Sender<T>) -> bool {
        self.0.channel_id() == other.0.channel_id()
    }

    pub fn downgrade(&self) -> WeakSender<T> {
        WeakSender(self.0.downgrade())
    }

    #[cfg(feature = "compat-tokio-time")]
    pub async fn send_timeout(
        &self,
        value: T,
        timeout: tokio::time::Duration,
    ) -> Result<(), error::SendTimeoutError<T>> {
        let mut send = pin!(self.0.send(value));
        match tokio::time::timeout(timeout, send.as_mut()).await {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(SendError(v))) => Err(error::SendTimeoutError::Closed(v)),
            Err(_) => Err(error::SendTimeoutError::Timeout(send.cancel().unwrap())),
        }
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> fmt::Debug for Sender<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Sender").field(&self.0).finish()
    }
}

impl<T> Receiver<T> {
    pub async fn recv(&mut self) -> Option<T> {
        self.0.recv().await.ok()
    }

    pub async fn recv_many(&mut self, buffer: &mut Vec<T>, limit: usize) -> usize {
        if limit == 0 {
            return 0;
        }
        match self.recv().await {
            Some(msg) => buffer.push(msg),
            None => return 0,
        }
        for i in 1..limit {
            match self.try_recv() {
                Ok(msg) => buffer.push(msg),
                Err(_) => return i,
            }
        }
        limit
    }

    pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
        self.0.try_recv().map_err(|e| match e {
            crate::errors::TryRecvError::Empty => TryRecvError::Empty,
            crate::errors::TryRecvError::Closed => TryRecvError::Disconnected,
        })
    }

    #[cfg(feature = "blocking")]
    pub fn blocking_recv(&mut self) -> Option<T> {
        assert!(
            tokio::runtime::Handle::try_current().is_err(),
            "blocking_recv called from async context"
        );
        self.0.recv_blocking().ok()
    }

    #[cfg(feature = "blocking")]
    pub fn blocking_recv_many(&mut self, buffer: &mut Vec<T>, limit: usize) -> usize {
        match self.blocking_recv() {
            Some(msg) => buffer.push(msg),
            None => return 0,
        }
        for i in 1..limit {
            match self.try_recv() {
                Ok(msg) => buffer.push(msg),
                Err(_) => return i,
            }
        }
        limit
    }

    pub fn close(&mut self) {
        self.0.close();
    }

    pub fn is_closed(&self) -> bool {
        self.0.is_closed()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn max_capacity(&self) -> usize {
        self.0.capacity()
    }

    pub fn poll_recv(&mut self, cx: &mut Context<'_>) -> Poll<Option<T>> {
        pin!(self.0.recv()).poll(cx).map(Result::ok)
    }

    pub fn poll_recv_many(
        &mut self,
        cx: &mut Context<'_>,
        buffer: &mut Vec<T>,
        limit: usize,
    ) -> Poll<usize> {
        if limit == 0 {
            return Poll::Ready(0);
        }
        match ready!(self.poll_recv(cx)) {
            Some(msg) => buffer.push(msg),
            None => return Poll::Ready(0),
        }
        for i in 1..limit {
            match self.try_recv() {
                Ok(msg) => buffer.push(msg),
                Err(_) => return Poll::Ready(i),
            }
        }
        Poll::Ready(limit)
    }
}

impl<T> fmt::Debug for Receiver<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Receiver").field(&self.0).finish()
    }
}

impl<T> WeakSender<T> {
    pub fn upgrade(&self) -> Option<Sender<T>> {
        self.0.upgrade().map(Sender)
    }
}

impl<T> Clone for WeakSender<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> fmt::Debug for WeakSender<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("WeakSender").field(&self.0).finish()
    }
}
