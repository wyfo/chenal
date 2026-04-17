pub(crate) extern crate std;
use alloc::{sync::Arc, task::Wake};
use core::{
    error, fmt, ptr,
    task::{Context, RawWaker, RawWakerVTable, Waker},
};
pub(crate) use std::time::{Duration, Instant};

use crate::{
    errors::{RecvError, SendError},
    loom::{thread, thread::Thread},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SendTimeoutError<T> {
    Timeout(T),
    Closed(T),
}

impl<T> fmt::Debug for SendTimeoutError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "SendTimeoutError(..)".fmt(f)
    }
}

impl<T> fmt::Display for SendTimeoutError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            SendTimeoutError::Timeout(..) => "timed out waiting on send operation".fmt(f),
            SendTimeoutError::Closed(..) => "sending on a closed channel".fmt(f),
        }
    }
}

impl<T> error::Error for SendTimeoutError<T> {}

impl<T> From<SendError<T>> for SendTimeoutError<T> {
    fn from(err: SendError<T>) -> SendTimeoutError<T> {
        match err {
            SendError(e) => SendTimeoutError::Closed(e),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RecvTimeoutError {
    Timeout,
    Closed,
}

impl fmt::Display for RecvTimeoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            RecvTimeoutError::Timeout => "timed out waiting on channel".fmt(f),
            RecvTimeoutError::Closed => "channel is empty and sending half is closed".fmt(f),
        }
    }
}

impl error::Error for RecvTimeoutError {}

impl From<RecvError> for RecvTimeoutError {
    fn from(err: RecvError) -> RecvTimeoutError {
        match err {
            RecvError => RecvTimeoutError::Closed,
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
static LAZY_WAKER: Waker = unsafe { Waker::new(ptr::null(), &LAZY_VTABLE) };

pub(crate) fn context() -> Context<'static> {
    Context::from_waker(&LAZY_WAKER)
}

pub(crate) struct TimeoutError;

#[derive(Default)]
pub(crate) struct Parker {
    deadline: Option<Instant>,
}

impl Parker {
    pub(crate) fn park(&mut self, timeout: Option<Duration>) -> Result<(), TimeoutError> {
        if let Some(timeout) = timeout {
            let now = Instant::now();
            let deadline = *self.deadline.get_or_insert(now + timeout);
            thread::park_timeout(deadline.checked_duration_since(now).ok_or(TimeoutError)?);
        } else {
            thread::park();
        }
        Ok(())
    }
}
