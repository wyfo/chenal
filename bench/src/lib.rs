use ::std::{
    fmt,
    pin::Pin,
    task::{Context, Poll},
};

pub mod async_channel;
pub mod chenal;
pub mod chenal_32;
pub mod crossbeam;
pub mod crossfire;
pub mod flume;
pub mod futures;
pub mod kanal;
pub mod postage;
pub mod std;
pub mod tachyonix;
pub mod tokio;

pub trait Sender<T>: Send + 'static {
    const CLONEABLE: bool;
    fn try_send(&mut self, msg: T);
    fn clone(&self) -> Self;
}

pub trait BlockingSender<T>: Sender<T> {
    fn send(&mut self, msg: T);
}

pub trait AsyncSender<T>: Sender<T> {
    fn send(&mut self, msg: T) -> impl Future<Output = ()> + Send + '_;
}

pub trait Receiver<T>: Send + 'static {
    const CLONEABLE: bool;
    fn try_recv(&mut self) -> T;
    fn clone(&self) -> Self;
}

pub trait BlockingReceiver<T>: Receiver<T> {
    fn recv(&mut self) -> T;
}

pub trait AsyncReceiver<T>: Receiver<T> {
    fn recv(&mut self) -> impl Future<Output = T> + Send + '_;
}

trait Unwrappable {
    type Output;
    fn unwrap(self) -> Self::Output;
}
impl<T> Unwrappable for Option<T> {
    type Output = T;
    fn unwrap(self) -> Self::Output {
        self.unwrap()
    }
}
impl<T, E: fmt::Debug> Unwrappable for Result<T, E> {
    type Output = T;
    fn unwrap(self) -> Self::Output {
        self.unwrap()
    }
}

struct UnwrapFuture<F>(F);
impl<F: Future<Output: Unwrappable>> Future for UnwrapFuture<F> {
    type Output = <F::Output as Unwrappable>::Output;

    #[inline(always)]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe { self.map_unchecked_mut(|this| &mut this.0) }
            .poll(cx)
            .map(Unwrappable::unwrap)
    }
}

trait FutureExt: Sized {
    fn unwrap(self) -> UnwrapFuture<Self>;
}
impl<F: Future<Output: Unwrappable>> FutureExt for F {
    fn unwrap(self) -> UnwrapFuture<Self> {
        UnwrapFuture(self)
    }
}

#[unsafe(no_mangle)]
fn tachyonix_send(tx: &mut ::tachyonix::Sender<usize>, msg: usize) {
    Sender::try_send(tx, msg);
}

#[unsafe(no_mangle)]
fn chenal_send(tx: &mut ::chenal::mpsc::MTx<usize>, msg: usize) {
    Sender::try_send(tx, msg);
}
