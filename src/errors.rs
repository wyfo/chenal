use core::{error, fmt, hint::unreachable_unchecked};

#[cfg(feature = "blocking")]
pub use crate::blocking::{RecvTimeoutError, SendTimeoutError};

pub(crate) enum TryAcquireError {
    Closed,
    Unavailable,
}
pub(crate) struct AcquireError;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum TrySendError<T> {
    Closed(T),
    Full(T),
}

impl<T> TrySendError<T> {
    pub fn is_closed(&self) -> bool {
        matches!(self, Self::Closed(_))
    }

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

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum TryRecvError {
    Closed,
    Empty,
}

impl TryRecvError {
    pub fn is_closed(&self) -> bool {
        matches!(self, Self::Closed)
    }

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
