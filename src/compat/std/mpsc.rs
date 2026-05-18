use core::{cell::UnsafeCell, fmt};

pub use crate::errors::{RecvError, SendError};
use crate::{
    blocking::{
        Duration, RecvTimeoutError as ChenalRecvTimeoutError,
        SendTimeoutError as ChenalSendTimeoutError,
    },
    errors, mpsc,
};

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum TrySendError<T> {
    Full(T),
    Disconnected(T),
}

impl<T> fmt::Debug for TrySendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Full(_) => f.debug_tuple("Full").finish_non_exhaustive(),
            Self::Disconnected(_) => f.debug_tuple("Disconnected").finish_non_exhaustive(),
        }
    }
}

impl<T> From<errors::TrySendError<T>> for TrySendError<T> {
    fn from(e: errors::TrySendError<T>) -> Self {
        match e {
            errors::TrySendError::Full(t) => Self::Full(t),
            errors::TrySendError::Closed(t) => Self::Disconnected(t),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TryRecvError {
    Empty,
    Disconnected,
}

impl From<errors::TryRecvError> for TryRecvError {
    fn from(e: errors::TryRecvError) -> Self {
        match e {
            errors::TryRecvError::Empty => Self::Empty,
            errors::TryRecvError::Closed => Self::Disconnected,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecvTimeoutError {
    Timeout,
    Disconnected,
}

impl From<ChenalRecvTimeoutError> for RecvTimeoutError {
    fn from(e: ChenalRecvTimeoutError) -> Self {
        match e {
            ChenalRecvTimeoutError::Timeout => Self::Timeout,
            ChenalRecvTimeoutError::Closed => Self::Disconnected,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum SendTimeoutError<T> {
    Timeout(T),
    Disconnected(T),
}

impl<T> fmt::Debug for SendTimeoutError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Timeout(_) => f.debug_tuple("Timeout").finish_non_exhaustive(),
            Self::Disconnected(_) => f.debug_tuple("Disconnected").finish_non_exhaustive(),
        }
    }
}

impl<T> From<ChenalSendTimeoutError<T>> for SendTimeoutError<T> {
    fn from(e: ChenalSendTimeoutError<T>) -> Self {
        match e {
            ChenalSendTimeoutError::Timeout(t) => Self::Timeout(t),
            ChenalSendTimeoutError::Closed(t) => Self::Disconnected(t),
        }
    }
}

pub struct SyncSender<T>(mpsc::MTx<T>);
pub struct Receiver<T>(UnsafeCell<mpsc::Rx<T>>);

pub fn sync_channel<T>(bound: usize) -> (SyncSender<T>, Receiver<T>) {
    let (tx, rx) = mpsc::channel(bound);
    (SyncSender(tx), Receiver(UnsafeCell::new(rx)))
}

impl<T> SyncSender<T> {
    pub fn send(&self, t: T) -> Result<(), SendError<T>> {
        self.0.send_blocking(t)
    }

    pub fn try_send(&self, t: T) -> Result<(), TrySendError<T>> {
        self.0.try_send(t).map_err(Into::into)
    }

    pub fn send_timeout(&self, t: T, timeout: Duration) -> Result<(), SendTimeoutError<T>> {
        self.0.send_timeout(t, timeout).map_err(Into::into)
    }
}

impl<T> Clone for SyncSender<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> fmt::Debug for SyncSender<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SyncSender").field(&self.0).finish()
    }
}

impl<T> fmt::Debug for Receiver<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // SAFETY: Receiver is !Sync; no concurrent access is possible.
        // We obtain &Rx<T> (not &mut), so there is no aliasing conflict.
        f.debug_tuple("Receiver")
            .field(unsafe { &*self.0.get() })
            .finish()
    }
}

impl<T> Receiver<T> {
    #[expect(clippy::mut_from_ref)]
    fn rx(&self) -> &mut mpsc::Rx<T> {
        unsafe { &mut *self.0.get() }
    }

    pub fn recv(&self) -> Result<T, RecvError> {
        self.rx().recv_blocking()
    }

    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        self.rx().try_recv().map_err(Into::into)
    }

    pub fn recv_timeout(&self, timeout: Duration) -> Result<T, RecvTimeoutError> {
        self.rx().recv_timeout(timeout).map_err(Into::into)
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter(self)
    }

    pub fn try_iter(&self) -> TryIter<'_, T> {
        TryIter(self)
    }
}

impl<T> IntoIterator for Receiver<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }
}

impl<'a, T> IntoIterator for &'a Receiver<T> {
    type Item = T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

pub struct Iter<'a, T>(&'a Receiver<T>);
pub struct TryIter<'a, T>(&'a Receiver<T>);
pub struct IntoIter<T>(Receiver<T>);

impl<T> Iterator for Iter<'_, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.0.recv().ok()
    }
}

impl<T> Iterator for TryIter<'_, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.0.try_recv().ok()
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.0.recv().ok()
    }
}
