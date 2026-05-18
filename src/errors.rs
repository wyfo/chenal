//! Error types.
use core::{error, fmt, hint::unreachable_unchecked};

#[cfg(feature = "blocking")]
pub use crate::blocking::{RecvTimeoutError, SendTimeoutError};

pub(crate) enum TryAcquireError {
    Closed,
    Unavailable,
}
pub(crate) struct AcquireError;

/// The channel is either full or closed.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum TrySendError<T> {
    /// The channel is closed.
    Closed(T),
    /// The channel is full.
    Full(T),
}

impl<T> TrySendError<T> {
    /// Returns `true` if the channel is closed.
    pub fn is_closed(&self) -> bool {
        matches!(self, Self::Closed(_))
    }

    /// Returns `true` if the channel is full.
    pub fn is_full(&self) -> bool {
        matches!(self, Self::Full(_))
    }
}

impl<T> fmt::Debug for TrySendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Full(_) => f.debug_tuple("TrySendError::Full").finish_non_exhaustive(),
            Self::Closed(_) => f
                .debug_tuple("TrySendError::Closed")
                .finish_non_exhaustive(),
        }
    }
}

impl<T> fmt::Display for TrySendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Full(_) => "sending on a full channel".fmt(f),
            Self::Closed(_) => "sending on a closed channel".fmt(f),
        }
    }
}

impl<T> error::Error for TrySendError<T> {}

impl<T> From<(TryAcquireError, T)> for TrySendError<T> {
    fn from((err, msg): (TryAcquireError, T)) -> Self {
        match err {
            TryAcquireError::Closed => Self::Closed(msg),
            TryAcquireError::Unavailable => Self::Full(msg),
        }
    }
}

/// The channel is closed.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct SendError<T>(pub T);

impl<T> fmt::Debug for SendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SendError").finish_non_exhaustive()
    }
}

impl<T> fmt::Display for SendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "sending on a closed channel".fmt(f)
    }
}

impl<T> error::Error for SendError<T> {}

impl<T> From<SendError<T>> for TrySendError<T> {
    fn from(err: SendError<T>) -> Self {
        match err {
            SendError(t) => Self::Closed(t),
        }
    }
}

impl<T> From<(AcquireError, T)> for SendError<T> {
    fn from((_err, msg): (AcquireError, T)) -> Self {
        Self(msg)
    }
}

impl<T> From<(TryAcquireError, T)> for SendError<T> {
    fn from((err, msg): (TryAcquireError, T)) -> Self {
        match err {
            TryAcquireError::Closed => Self(msg),
            _ => unsafe { unreachable_unchecked() },
        }
    }
}

/// The channel is either empty or closed (and empty).
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum TryRecvError {
    /// The channel is closed and empty.
    Closed,
    /// The channel is empty but not closed.
    Empty,
}

impl TryRecvError {
    /// Returns `true` if the channel is closed.
    pub fn is_closed(&self) -> bool {
        matches!(self, Self::Closed)
    }

    /// Returns `true` if the channel is empty.
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }
}

impl fmt::Display for TryRecvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => "receiving on an empty channel".fmt(f),
            Self::Closed => "receiving on a closed channel".fmt(f),
        }
    }
}

impl From<TryAcquireError> for TryRecvError {
    fn from(err: TryAcquireError) -> Self {
        match err {
            TryAcquireError::Closed => Self::Closed,
            TryAcquireError::Unavailable => Self::Empty,
        }
    }
}

/// The channel is closed and empty.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct RecvError;

impl fmt::Display for RecvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "receiving on a closed channel".fmt(f)
    }
}

impl error::Error for RecvError {}

impl From<RecvError> for TryRecvError {
    fn from(err: RecvError) -> Self {
        match err {
            RecvError => Self::Closed,
        }
    }
}

impl From<AcquireError> for RecvError {
    fn from(_err: AcquireError) -> Self {
        Self
    }
}

impl From<TryAcquireError> for RecvError {
    fn from(err: TryAcquireError) -> Self {
        match err {
            TryAcquireError::Closed => Self,
            _ => unsafe { unreachable_unchecked() },
        }
    }
}
