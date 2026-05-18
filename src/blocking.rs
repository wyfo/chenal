pub(crate) extern crate std;
use alloc::{sync::Arc, task::Wake};
use core::{
    error, fmt, ptr,
    task::{RawWaker, RawWakerVTable, Waker},
};
pub(crate) use std::time::{Duration, Instant};

use crate::{
    errors::{RecvError, SendError, TryAcquireError},
    loom::{thread, thread::Thread},
};

/// The operation either timed out or the channel is closed.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SendTimeoutError<T> {
    /// The channel is closed.
    Closed(T),
    /// The operation timed out.
    Timeout(T),
}

impl<T> SendTimeoutError<T> {
    /// Returns `true` if the channel was closed.
    pub fn is_closed(&self) -> bool {
        matches!(self, Self::Closed(_))
    }

    /// Returns `true` if the operation timed out.
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout(_))
    }
}

impl<T> fmt::Debug for SendTimeoutError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "SendTimeoutError(..)".fmt(f)
    }
}

impl<T> fmt::Display for SendTimeoutError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            SendTimeoutError::Timeout(_) => "timed out waiting on send operation".fmt(f),
            SendTimeoutError::Closed(_) => "sending on a closed channel".fmt(f),
        }
    }
}

impl<T> error::Error for SendTimeoutError<T> {}

impl<T> From<SendError<T>> for SendTimeoutError<T> {
    fn from(err: SendError<T>) -> Self {
        match err {
            SendError(e) => SendTimeoutError::Closed(e),
        }
    }
}

impl<T> From<(TryAcquireError, T)> for SendTimeoutError<T> {
    fn from((err, msg): (TryAcquireError, T)) -> Self {
        match err {
            TryAcquireError::Closed => Self::Closed(msg),
            TryAcquireError::Unavailable => Self::Timeout(msg),
        }
    }
}

/// The operation either timed out or the channel is closed and empty.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RecvTimeoutError {
    /// The channel is closed and empty.
    Closed,
    /// The operation timed out.
    Timeout,
}

impl RecvTimeoutError {
    /// Returns `true` if the channel was closed.
    pub fn is_closed(&self) -> bool {
        matches!(self, Self::Closed)
    }

    /// Returns `true` if the operation timed out.
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout)
    }
}

impl fmt::Display for RecvTimeoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Timeout => "timed out waiting on channel".fmt(f),
            Self::Closed => "channel is empty and sending half is closed".fmt(f),
        }
    }
}

impl error::Error for RecvTimeoutError {}

impl From<RecvError> for RecvTimeoutError {
    fn from(err: RecvError) -> Self {
        match err {
            RecvError => Self::Closed,
        }
    }
}

impl From<TryAcquireError> for RecvTimeoutError {
    fn from(err: TryAcquireError) -> Self {
        match err {
            TryAcquireError::Closed => Self::Closed,
            TryAcquireError::Unavailable => Self::Timeout,
        }
    }
}

struct ThreadWaker(Thread);
impl Wake for ThreadWaker {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
}

std::thread_local! {
    static THREAD: Arc<ThreadWaker> = Arc::new(ThreadWaker(thread::current()));
}

static LAZY_VTABLE: RawWakerVTable = RawWakerVTable::new(
    |_| RawWaker::from(THREAD.with(|t| t.clone())),
    |_| THREAD.with(|t| t.wake_by_ref()),
    |_| THREAD.with(|t| t.wake_by_ref()),
    |_| {},
);
pub(crate) static PARK_WAKER: Waker = unsafe { Waker::new(ptr::null(), &LAZY_VTABLE) };

pub(crate) enum Parker {
    Blocking,
    Deadline(Instant),
    Timeout {
        timeout: Duration,
        deadline: Option<Instant>,
    },
}

impl Parker {
    #[cfg_attr(loom, expect(dead_code))]
    pub(crate) fn park(&mut self) -> Result<(), TryAcquireError> {
        let (deadline, now) = match self {
            Self::Blocking => {
                thread::park();
                return Ok(());
            }
            Self::Deadline(deadline) => (*deadline, Instant::now()),
            Self::Timeout { timeout, deadline } => {
                let now = Instant::now();
                (*deadline.get_or_insert(now + *timeout), now)
            }
        };
        #[cfg(loom)]
        unimplemented!();
        let timeout = deadline
            .checked_duration_since(now)
            .ok_or(TryAcquireError::Unavailable)?;
        #[cfg(not(loom))]
        thread::park_timeout(timeout);
        Ok(())
    }
}

impl From<()> for Parker {
    fn from(_: ()) -> Self {
        Self::Blocking
    }
}

impl From<Instant> for Parker {
    fn from(deadline: Instant) -> Self {
        Self::Deadline(deadline)
    }
}

impl From<Duration> for Parker {
    fn from(timeout: Duration) -> Self {
        Self::Timeout {
            timeout,
            deadline: None,
        }
    }
}
