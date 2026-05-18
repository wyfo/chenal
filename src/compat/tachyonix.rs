use core::{
    fmt,
    future::poll_fn,
    pin::{Pin, pin},
    task::Poll,
};

pub use crate::errors::{RecvError, SendError, TryRecvError, TrySendError};
use crate::mpsc;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum SendTimeoutError<T> {
    Timeout(T),
    Closed(T),
}

impl<T> fmt::Debug for SendTimeoutError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Timeout(_) => f.debug_tuple("Timeout").finish_non_exhaustive(),
            Self::Closed(_) => f.debug_tuple("Closed").finish_non_exhaustive(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecvTimeoutError {
    Timeout,
    Closed,
}

pub struct Sender<T>(mpsc::MTx<T>);
pub struct Receiver<T>(mpsc::Rx<T>);

pub fn channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = mpsc::channel(capacity);
    (Sender(tx), Receiver(rx))
}

async fn select<F1: Future, F2: Future>(
    mut f1: Pin<&mut F1>,
    mut f2: Pin<&mut F2>,
) -> Result<F1::Output, F2::Output> {
    poll_fn(|cx| {
        if let Poll::Ready(r1) = f1.as_mut().poll(cx) {
            return Poll::Ready(Ok(r1));
        }
        if let Poll::Ready(r2) = f2.as_mut().poll(cx) {
            return Poll::Ready(Err(r2));
        }
        Poll::Pending
    })
    .await
}

impl<T> Sender<T> {
    pub async fn send(&self, message: T) -> Result<(), SendError<T>> {
        self.0.send(message).await
    }

    pub fn try_send(&self, message: T) -> Result<(), TrySendError<T>> {
        self.0.try_send(message)
    }

    pub async fn send_timeout<D: core::future::Future<Output = ()>>(
        &self,
        message: T,
        deadline: D,
    ) -> Result<(), SendTimeoutError<T>> {
        let mut send = pin!(self.0.send(message));
        match select(send.as_mut(), pin!(deadline)).await {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(SendError(msg))) => Err(SendTimeoutError::Closed(msg)),
            Err(_) => Err(SendTimeoutError::Timeout(send.cancel().unwrap())),
        }
    }

    pub fn is_closed(&self) -> bool {
        self.0.is_closed()
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
    pub async fn recv(&mut self) -> Result<T, RecvError> {
        self.0.recv().await
    }

    pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
        self.0.try_recv()
    }

    pub async fn recv_timeout<D: core::future::Future<Output = ()>>(
        &mut self,
        deadline: D,
    ) -> Result<T, RecvTimeoutError> {
        use core::{future::poll_fn, pin::pin};
        let mut recv_fut = pin!(self.0.recv());
        let mut deadline = pin!(deadline);
        poll_fn(|cx| {
            if let Poll::Ready(r) = recv_fut.as_mut().poll(cx) {
                return Poll::Ready(match r {
                    Ok(v) => Ok(v),
                    Err(_) => Err(RecvTimeoutError::Closed),
                });
            }
            if deadline.as_mut().poll(cx).is_ready() {
                return Poll::Ready(Err(RecvTimeoutError::Timeout));
            }
            Poll::Pending
        })
        .await
    }

    pub fn close(&mut self) {
        self.0.close();
    }
}

impl<T> fmt::Debug for Receiver<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Receiver").field(&self.0).finish()
    }
}

#[cfg(feature = "stream")]
impl<T> futures_core::Stream for Receiver<T> {
    type Item = T;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> Poll<Option<T>> {
        use core::pin::pin;
        pin!(self.0.recv()).poll(cx).map(Result::ok)
    }
}
