use ::std::{fmt, future::Future};

pub const CAPACITIES: &[usize] = &[64, 256, 1024, 16384];
pub const N_MSGS: usize = 100_000;

pub fn sender_counts() -> Vec<usize> {
    let par = ::std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let last = par.saturating_sub(1).max(4);
    let mut v = vec![1usize, 2, 4];
    if !v.contains(&last) {
        v.push(last);
    }
    v
}

#[derive(Clone, Copy, PartialEq)]
pub enum RecvWork {
    None,
    Spin,
}

impl fmt::Display for RecvWork {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            RecvWork::None => "none",
            RecvWork::Spin => "spin",
        })
    }
}

pub mod async_channel;
pub mod chenal;
pub mod chenal_32;
pub mod chenal_loop;
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
    // `&mut self` because of `Sink` channels
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
