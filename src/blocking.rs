pub(crate) extern crate std;
use alloc::{sync::Arc, task::Wake};
use core::{
    error, fmt,
    task::{RawWakerVTable, Waker},
};
pub(crate) use std::time::{Duration, Instant};

#[cfg(not(loom))]
use crate::loom::thread;
use crate::{
    errors::{RecvError, SendError, TryAcquireError},
    loom::thread_local,
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

#[cfg(loom)]
#[derive(Default)]
pub(crate) struct LoomParker {
    mutex: loom::sync::Mutex<bool>,
    condvar: loom::sync::Condvar,
}

#[cfg(loom)]
impl LoomParker {
    fn park(&self) {
        let mut guard = self.mutex.lock().unwrap();
        while !*guard {
            guard = self.condvar.wait(guard).unwrap();
        }
        *guard = false;
    }
    fn unpark(&self) {
        let mut guard = self.mutex.lock().unwrap();
        *guard = true;
        self.condvar.notify_one();
    }
}

// https://github.com/tokio-rs/loom/issues/249
struct ThreadWaker(#[cfg(not(loom))] thread::Thread, #[cfg(loom)] LoomParker);
impl Wake for ThreadWaker {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.0.unpark();
    }
}
thread_local! {
    static THREAD: Arc<ThreadWaker> = Arc::new(ThreadWaker(#[cfg(not(loom))]thread::current(), #[cfg(loom)] LoomParker::default()));
}

static LAZY_VTABLE: RawWakerVTable = RawWakerVTable::new(
    |_| THREAD.with(|t| t.clone()).into(),
    |_| THREAD.with(|t| t.wake_by_ref()),
    |_| THREAD.with(|t| t.wake_by_ref()),
    |_| {},
);

pub(crate) static PARK_WAKER: Waker = unsafe { Waker::new(core::ptr::null(), &LAZY_VTABLE) };

pub(crate) enum Parker {
    Blocking,
    Deadline(Instant),
    Timeout {
        timeout: Duration,
        deadline: Option<Instant>,
    },
}

impl Parker {
    #[cfg_attr(loom, expect(unused))]
    pub(crate) fn park(&mut self) -> Result<(), TryAcquireError> {
        let (deadline, now) = match self {
            Self::Blocking => {
                #[cfg(not(loom))]
                thread::park();
                // https://github.com/tokio-rs/loom/issues/249
                #[cfg(loom)]
                THREAD.with(|t| t.0.park());
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
