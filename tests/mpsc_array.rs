use core::{
    pin::pin,
    task::{Context, Poll, Waker},
};
use std::{iter, sync::Arc, thread::ScopedJoinHandle};

use chenal::{
    Channel, MTx, Rx,
    errors::{RecvError, SendError, TryRecvError, TrySendError},
    mpsc::Array,
};
use futures::executor::block_on;
use rstest::rstest;

struct Bool<const BOOL: bool>;
const TRUE: Bool<true> = Bool::<true>;
const FALSE: Bool<false> = Bool::<false>;

trait Send2<T> {
    fn send2(&self, msg: T, sync: bool) -> Result<(), SendError<T>>;
}
impl<T, const UB: bool> Send2<T> for MTx<T, Array<usize, UB>> {
    #[cfg_attr(not(feature = "blocking"), allow(unused_variables))]
    fn send2(&self, msg: T, sync: bool) -> Result<(), SendError<T>> {
        #[cfg(feature = "blocking")]
        if sync {
            return self.send_blocking(msg);
        }
        block_on(self.send(msg))
    }
}
trait Recv2<T> {
    fn recv2(&mut self, sync: bool) -> Result<T, RecvError>;
}
impl<T, const UB: bool> Recv2<T> for Rx<T, Array<usize, UB>> {
    #[cfg_attr(not(feature = "blocking"), allow(unused_variables))]
    fn recv2(&mut self, sync: bool) -> Result<T, RecvError> {
        #[cfg(feature = "blocking")]
        if sync {
            return self.recv_blocking();
        }
        block_on(self.recv())
    }
}

#[rstest]
fn mpsc<const UB: bool>(#[values(false, true)] sync: bool, #[values(FALSE, TRUE)] ub: Bool<UB>) {
    let _ = ub;
    let (tx, mut rx) = <Array<_, UB>>::new(2).channel();
    let mut values = std::thread::scope(|s| {
        s.spawn(|| tx.send2(0, sync));
        s.spawn(|| tx.send2(1, sync));
        s.spawn(|| tx.send2(2, sync));
        (0..3).map(|_| rx.recv2(sync).unwrap()).collect::<Vec<_>>()
    });
    values.sort_unstable();
    assert_eq!(values, [0, 1, 2]);
}

// Sequential send/recv preserves FIFO order.
#[rstest]
fn sequential<const UB: bool>(#[values(FALSE, TRUE)] ub: Bool<UB>) {
    let _ = ub;
    let (tx, mut rx) = <Array<_, UB>>::new(2).channel();
    tx.try_send(1).unwrap();
    tx.try_send(2).unwrap();
    assert_eq!(rx.try_recv(), Ok(1));
    assert_eq!(rx.try_recv(), Ok(2));
    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
}

// Ring buffer wraps around correctly across multiple laps.
#[rstest]
fn wrap_around<const UB: bool>(#[values(FALSE, TRUE)] ub: Bool<UB>) {
    let _ = ub;
    let (tx, mut rx) = <Array<_, UB>>::new(2).channel();
    for i in 0..4 {
        tx.try_send(i).unwrap();
        assert_eq!(rx.try_recv(), Ok(i));
    }
}

// try_send on a full channel returns Full without blocking.
#[rstest]
fn try_send_full<const UB: bool>(#[values(FALSE, TRUE)] ub: Bool<UB>) {
    let _ = ub;
    let (tx, mut rx) = <Array<_, UB>>::new(1).channel();
    assert!(!tx.is_full());
    tx.try_send(0).unwrap();
    assert!(tx.is_full());
    assert_eq!(tx.try_send(1), Err(TrySendError::Full(1)));
    assert_eq!(rx.try_recv(), Ok(0));
    tx.try_send(1).unwrap();
    assert_eq!(tx.try_send(2), Err(TrySendError::Full(2)));
}

// try_recv on an empty channel returns Empty without blocking.
#[rstest]
fn try_recv_empty<const UB: bool>(#[values(FALSE, TRUE)] ub: Bool<UB>) {
    let _ = ub;
    let (tx, mut rx) = <Array<_, UB>>::new(1).channel::<usize>();
    assert!(rx.is_empty());
    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
    tx.try_send(0).unwrap();
    assert!(!rx.is_empty());
    assert_eq!(rx.try_recv(), Ok(0));
    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
}

// Dropping all senders closes the channel; buffered messages are still readable.
#[rstest]
fn tx_drop_closes<const UB: bool>(
    #[values(false, true)] sync: bool,
    #[values(FALSE, TRUE)] ub: Bool<UB>,
) {
    let _ = ub;
    let (tx, mut rx) = <Array<_, UB>>::new(2).channel();
    let tx2 = tx.clone();
    assert!(!rx.is_closed());
    tx.try_send(42).unwrap();
    drop(tx2);
    assert!(!rx.is_closed());
    drop(tx);
    assert!(rx.is_closed());
    assert_eq!(rx.recv2(sync), Ok(42));
    assert_eq!(rx.try_recv(), Err(TryRecvError::Closed));
    assert_eq!(rx.recv2(sync), Err(RecvError));
}

// Dropping the receiver closes the channel; sends return Closed.
#[rstest]
fn rx_drop_closes<const UB: bool>(
    #[values(false, true)] sync: bool,
    #[values(FALSE, TRUE)] ub: Bool<UB>,
) {
    let _ = ub;
    let (tx, rx) = <Array<_, UB>>::new(2).channel::<usize>();
    assert!(!tx.is_closed());
    drop(rx);
    assert!(tx.is_closed());
    assert_eq!(tx.try_send(0), Err(TrySendError::Closed(0)));
    assert_eq!(tx.send2(0, sync), Err(SendError(0)));
}

// Dropping the sender while recv is blocked wakes the receiver with RecvError.
#[rstest]
fn tx_drop_while_recv_waiting<const UB: bool>(
    #[values(false, true)] sync: bool,
    #[values(FALSE, TRUE)] ub: Bool<UB>,
) {
    let _ = ub;
    let (tx, mut rx) = <Array<_, UB>>::new(1).channel::<usize>();
    let recv = std::thread::spawn(move || rx.recv2(sync));
    drop(tx);
    assert_eq!(recv.join().unwrap(), Err(RecvError));
}

// Dropping the receiver while send is blocked wakes the sender with SendError.
#[rstest]
fn rx_drop_while_send_waiting<const UB: bool>(
    #[values(false, true)] sync: bool,
    #[values(FALSE, TRUE)] ub: Bool<UB>,
) {
    let _ = ub;
    let (tx, rx) = <Array<_, UB>>::new(1).channel();
    tx.try_send(0).unwrap();
    let send = std::thread::spawn(move || tx.send2(1, sync));
    drop(rx);
    assert_eq!(send.join().unwrap(), Err(SendError(1)));
}

// Closing the channel concurrently with sends; all messages are either received or returned as errors.
#[rstest]
fn concurrent_close<const UB: bool>(
    #[values(false, true)] sync: bool,
    #[values(FALSE, TRUE)] ub: Bool<UB>,
) {
    let _ = ub;
    let (tx, mut rx) = <Array<_, UB>>::new(2).channel();
    let mut values = std::thread::scope(|s| {
        let s1 = s.spawn(|| tx.send2(0, sync));
        let s2 = s.spawn(|| tx.send2(1, sync));
        let s3 = s.spawn(|| tx.send2(2, sync));
        s.spawn(|| tx.close());
        let err = |res: ScopedJoinHandle<Result<(), SendError<i32>>>| {
            res.join().unwrap().err().map(|SendError(m)| m)
        };
        iter::from_fn(|| rx.recv2(sync).ok())
            .chain(err(s1))
            .chain(err(s2))
            .chain(err(s3))
            .collect::<Vec<_>>()
    });
    values.sort_unstable();
    assert_eq!(values, [0, 1, 2]);
}

// A canceled SendFuture does not deliver its message.
#[rstest]
fn send_future_cancel<const UB: bool>(
    #[values(FALSE, TRUE)] ub: Bool<UB>,
    #[values(false, true)] take: bool,
) {
    let _ = ub;
    let (tx, mut rx) = <Array<_, UB>>::new(1).channel();
    tx.try_send(0).unwrap();
    {
        let mut fut = pin!(tx.send(1));
        let mut cx = Context::from_waker(Waker::noop());
        assert_eq!(fut.as_mut().poll(&mut cx), Poll::Pending);
        if take {
            assert_eq!(fut.cancel(), Some(1));
        }
        assert_eq!(rx.try_recv(), Ok(0));
    }
    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
}

// A canceled RecvFuture does not consume a message.
#[rstest]
fn recv_future_cancel<const UB: bool>(#[values(FALSE, TRUE)] ub: Bool<UB>) {
    let _ = ub;
    let (tx, mut rx) = <Array<_, UB>>::new(1).channel();
    {
        let mut fut = pin!(rx.recv());
        let mut cx = Context::from_waker(Waker::noop());
        assert_eq!(fut.as_mut().poll(&mut cx), Poll::Pending);
        tx.try_send(0).unwrap();
    }
    assert_eq!(rx.try_recv(), Ok(0));
}

// SendFuture panics if polled after completion.
#[rstest]
#[should_panic(expected = "future polled after completion")]
fn send_future_poll_after_completion(#[values(false, true)] cancel: bool) {
    let (tx, mut rx) = <Array>::new(1).channel();
    let mut cx = Context::from_waker(Waker::noop());
    let mut fut = pin!(tx.send(0));
    if cancel {
        assert_eq!(fut.as_mut().cancel(), Some(0));
    } else {
        assert_eq!(fut.as_mut().poll(&mut cx), Poll::Ready(Ok(())));
        assert_eq!(rx.try_recv(), Ok(0)); // drain so the second poll can acquire a slot
    }
    let _ = fut.as_mut().poll(&mut cx);
}

// Messages still in the buffer are dropped when the channel is dropped.
#[rstest]
#[case::empty(0, 0)]
#[case::full(0, 4)]
#[case::contiguous(0, 2)]
#[case::wrapped(3, 2)]
fn drop_buffered(#[case] offset: usize, #[case] msgs: usize) {
    let arc = Arc::new(());
    let (tx, mut rx) = <Array>::new(4).channel();
    assert_eq!(tx.capacity(), 4);
    assert_eq!(rx.capacity(), 4);
    for _ in 0..offset {
        tx.try_send(arc.clone()).unwrap();
        rx.try_recv().unwrap();
    }
    for _ in 0..msgs {
        tx.try_send(arc.clone()).unwrap();
    }
    assert_eq!(Arc::strong_count(&arc), 1 + msgs);
    drop((tx, rx));
    assert_eq!(Arc::strong_count(&arc), 1);
}
