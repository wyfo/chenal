use core::{
    marker::PhantomData,
    pin::pin,
    task::{Context, Poll, Waker},
};
use std::{iter, sync::Arc, thread};

use chenal::{
    Channel, MTx, Rx,
    backoff::{ExponentialBackoff, NoBackoff, UnboundedBackoffStrategy},
    errors::{RecvError, SendError, TryRecvError, TrySendError},
    mpsc::Array,
};
#[cfg(not(loom))]
use futures::executor::block_on;
#[cfg(loom)]
use loom::future::block_on;
use rstest::rstest;

struct Ub<U>(PhantomData<U>);
const NO: Ub<NoBackoff> = Ub(PhantomData);
const EXP: Ub<ExponentialBackoff<6, true>> = Ub(PhantomData);
struct Usize<const USIZE: usize>;
const ONE: Usize<1> = Usize::<1>;
const TWO: Usize<2> = Usize::<2>;

trait Send2<T> {
    fn send2(&self, msg: T, sync: bool) -> Result<(), SendError<T>>;
}
impl<T, const BLOCK_SIZE: usize, U: UnboundedBackoffStrategy> Send2<T>
    for MTx<T, Array<BLOCK_SIZE, usize, U>>
{
    fn send2(&self, msg: T, sync: bool) -> Result<(), SendError<T>> {
        if sync {
            self.send_blocking(msg)
        } else {
            block_on(self.send(msg))
        }
    }
}
trait Recv2<T> {
    fn recv2(&mut self, sync: bool) -> Result<T, RecvError>;
}
impl<T, const BLOCK_SIZE: usize, U: UnboundedBackoffStrategy> Recv2<T>
    for Rx<T, Array<BLOCK_SIZE, usize, U>>
{
    fn recv2(&mut self, sync: bool) -> Result<T, RecvError> {
        if sync {
            self.recv_blocking()
        } else {
            block_on(self.recv())
        }
    }
}

#[rstest]
fn mpsc<const BS: usize, U: UnboundedBackoffStrategy>(
    #[values(false, true)] sync: bool,
    #[values(ONE, TWO)] bs: Usize<BS>,
    #[values(NO, EXP)] ub: Ub<U>,
) {
    let _ = (bs, ub);
    let (tx, mut rx) = <Array<BS, _, U>>::new(2).channel();
    let mut values = thread::scope(|s| {
        s.spawn(|| tx.send2(0, sync).unwrap());
        s.spawn(|| tx.send2(1, sync).unwrap());
        s.spawn(|| tx.send2(2, sync).unwrap());
        (0..3).map(|_| rx.recv2(sync).unwrap()).collect::<Vec<_>>()
    });
    values.sort_unstable();
    assert_eq!(values, [0, 1, 2]);
}

// Sequential send/recv preserves FIFO order.
#[rstest]
fn sequential<const BS: usize, U: UnboundedBackoffStrategy>(
    #[values(ONE, TWO)] bs: Usize<BS>,
    #[values(NO, EXP)] ub: Ub<U>,
) {
    let _ = (bs, ub);
    let (tx, mut rx) = <Array<1, _, U>>::new(2).channel();
    tx.try_send(1).unwrap();
    tx.try_send(2).unwrap();
    assert_eq!(rx.try_recv(), Ok(1));
    assert_eq!(rx.try_recv(), Ok(2));
    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
}

// Ring buffer wraps around correctly across multiple laps.
#[rstest]
fn wrap_around<const BS: usize, U: UnboundedBackoffStrategy>(
    #[values(ONE, TWO)] bs: Usize<BS>,
    #[values(NO, EXP)] ub: Ub<U>,
) {
    let _ = (bs, ub);
    let (tx, mut rx) = <Array<BS, _, U>>::new(2).channel();
    for i in 0..4 {
        tx.try_send(i).unwrap();
        assert_eq!(rx.try_recv(), Ok(i));
    }
}

// try_send on a full channel returns Full without blocking.
#[rstest]
fn try_send_full<U: UnboundedBackoffStrategy>(#[values(NO, EXP)] ub: Ub<U>) {
    let _ = ub;
    let (tx, mut rx) = <Array<1, _, U>>::new(1).channel();
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
fn try_recv_empty<U: UnboundedBackoffStrategy>(#[values(NO, EXP)] ub: Ub<U>) {
    let _ = ub;
    let (tx, mut rx) = <Array<1, _, U>>::new(1).channel::<usize>();
    assert!(rx.is_empty());
    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
    tx.try_send(0).unwrap();
    assert!(!rx.is_empty());
    assert_eq!(rx.try_recv(), Ok(0));
    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
}

// Dropping all senders closes the channel; buffered messages are still readable.
#[rstest]
fn tx_drop_closes<U: UnboundedBackoffStrategy>(
    #[values(false, true)] sync: bool,
    #[values(NO, EXP)] ub: Ub<U>,
) {
    let _ = ub;
    let (tx, mut rx) = <Array<1, _, U>>::new(2).channel();
    let tx2 = tx.clone();
    let weak = tx2.downgrade();
    tx.try_send(42).unwrap();
    assert!(!rx.is_closed());
    drop(tx2);
    assert!(!rx.is_closed());
    drop(tx);
    assert!(rx.is_closed());
    assert_eq!(rx.recv2(sync), Ok(42));
    assert_eq!(rx.try_recv(), Err(TryRecvError::Closed));
    assert_eq!(rx.recv2(sync), Err(RecvError));
    assert!(weak.upgrade().is_none());
}

// Dropping the receiver closes the channel; sends return Closed.
#[rstest]
fn rx_drop_closes<U: UnboundedBackoffStrategy>(
    #[values(false, true)] sync: bool,
    #[values(NO, EXP)] ub: Ub<U>,
) {
    let _ = ub;
    let (tx, rx) = <Array<1, _, U>>::new(2).channel::<usize>();
    assert!(!tx.is_closed());
    drop(rx);
    assert!(tx.is_closed());
    assert_eq!(tx.try_send(0), Err(TrySendError::Closed(0)));
    assert_eq!(tx.send2(0, sync), Err(SendError(0)));
}

// Dropping the sender while recv is blocked wakes the receiver with RecvError.
#[rstest]
fn tx_drop_while_recv_waiting<U: UnboundedBackoffStrategy>(
    #[values(false, true)] sync: bool,
    #[values(NO, EXP)] ub: Ub<U>,
) {
    let _ = ub;
    let (tx, mut rx) = <Array<1, _, U>>::new(1).channel::<usize>();
    let recv = thread::spawn(move || rx.recv2(sync));
    drop(tx);
    assert_eq!(recv.join().unwrap(), Err(RecvError));
}

// Dropping the receiver while send is blocked wakes the sender with SendError.
#[rstest]
fn rx_drop_while_send_waiting<U: UnboundedBackoffStrategy>(
    #[values(false, true)] sync: bool,
    #[values(NO, EXP)] ub: Ub<U>,
) {
    let _ = ub;
    let (tx, rx) = <Array<1, _, U>>::new(1).channel();
    tx.try_send(0).unwrap();
    let send = thread::spawn(move || tx.send2(1, sync));
    drop(rx);
    assert_eq!(send.join().unwrap(), Err(SendError(1)));
}

// Closing the channel concurrently with sends; all messages are either received or returned as errors.
#[rstest]
fn concurrent_close<const BS: usize, U: UnboundedBackoffStrategy>(
    #[values(false, true)] sync: bool,
    #[values(ONE, TWO)] bs: Usize<BS>,
    #[values(NO, EXP)] ub: Ub<U>,
) {
    let _ = (bs, ub);
    let (tx, mut rx) = <Array<BS, _, U>>::new(2).channel();
    let mut values = thread::scope(|s| {
        let s1 = s.spawn(|| tx.send2(0, sync));
        let s2 = s.spawn(|| tx.send2(1, sync));
        let s3 = s.spawn(|| tx.send2(2, sync));
        s.spawn(|| tx.close());
        let send = [s1, s2, s3]
            .into_iter()
            .flat_map(|s| Some(s.join().unwrap().err()?.0));
        iter::from_fn(|| rx.recv2(sync).ok())
            .chain(send)
            .collect::<Vec<_>>()
    });
    values.sort_unstable();
    assert_eq!(values, [0, 1, 2]);
}

// A canceled SendFuture does not deliver its message.
#[rstest]
fn send_future_cancel<U: UnboundedBackoffStrategy>(
    #[values(NO, EXP)] ub: Ub<U>,
    #[values(false, true)] take: bool,
) {
    let _ = ub;
    let (tx, mut rx) = <Array<1, _, U>>::new(1).channel();
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

// Canceling a notified SendFuture passes the notification to the next waiter.
#[rstest]
fn send_future_cancel_wakes_next<U: UnboundedBackoffStrategy>(#[values(NO, EXP)] ub: Ub<U>) {
    let _ = ub;
    let (tx, mut rx) = <Array<1, _, U>>::new(1).channel();
    tx.try_send(0).unwrap();
    let mut cx = Context::from_waker(Waker::noop());
    let mut send1 = pin!(tx.send(1));
    assert_eq!(send1.as_mut().poll(&mut cx), Poll::Pending);
    let mut send2 = pin!(tx.send(2));
    assert_eq!(send2.as_mut().poll(&mut cx), Poll::Pending);
    assert_eq!(rx.try_recv(), Ok(0)); // notifies send1
    assert_eq!(send1.as_mut().cancel(), Some(1)); // cancels send1, should re-notify send2
    assert_eq!(send2.as_mut().poll(&mut cx), Poll::Ready(Ok(())));
    assert_eq!(rx.try_recv(), Ok(2));
}

// A canceled RecvFuture does not consume a message.
#[rstest]
fn recv_future_cancel<U: UnboundedBackoffStrategy>(#[values(NO, EXP)] ub: Ub<U>) {
    let _ = ub;
    let (tx, mut rx) = <Array<1, _, U>>::new(1).channel();
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

// Invalid capacities are rejected.
#[rstest]
#[case::zero(0, ONE)]
#[case::overflow((1 << (usize::BITS / 2 - 1)) + 1, ONE)]
#[case::not_multiple_of_block_size(3, TWO)]
#[should_panic]
fn invalid_capacity<const BS: usize>(#[case] capacity: usize, #[case] bs: Usize<BS>) {
    let _ = bs;
    <Array<BS>>::new(capacity).channel::<usize>();
}

// Capacity is reduced by partially read block.
#[rstest]
#[case(ONE, false)]
#[case(TWO, true)]
fn partial_block_withholds_capacity<const BS: usize>(#[case] bs: Usize<BS>, #[case] reduced: bool) {
    let _ = bs;
    let (tx, mut rx) = <Array<BS>>::new(4).channel();
    for i in 0..4 {
        tx.try_send(i).unwrap();
    }
    assert_eq!(rx.try_recv(), Ok(0));
    assert_eq!(tx.try_send(4).is_err(), reduced);
}

// Reading the last slot of a block wakes `BLOCK_SIZE` blocked senders at once.
#[rstest]
fn block_completion_wakes_senders<U: UnboundedBackoffStrategy>(#[values(NO, EXP)] ub: Ub<U>) {
    let _ = ub;
    let (tx, mut rx) = <Array<2, _, U>>::new(2).channel();
    tx.try_send(0).unwrap();
    tx.try_send(1).unwrap();
    let mut send1 = pin!(tx.send(2));
    let mut send2 = pin!(tx.send(3));
    let mut cx = Context::from_waker(Waker::noop());
    assert!(send1.as_mut().poll(&mut cx).is_pending());
    assert!(send2.as_mut().poll(&mut cx).is_pending());
    assert_eq!(rx.try_recv(), Ok(0));
    assert!(send1.as_mut().poll(&mut cx).is_pending());
    assert!(send2.as_mut().poll(&mut cx).is_pending());
    thread::scope(|s| {
        s.spawn(|| block_on(send1));
        s.spawn(|| block_on(send2));
        s.spawn(|| assert_eq!(rx.try_recv(), Ok(1)));
    });
}

#[cfg(loom)]
#[rstest]
fn loom_mpsc<const BS: usize, U: UnboundedBackoffStrategy>(
    #[values(false, true)] sync: bool,
    #[values(ONE, TWO)] bs: Usize<BS>,
    #[values(NO, EXP)] ub: Ub<U>,
) {
    let _ = (bs, ub);
    loom::model(move || {
        let (tx, mut rx) = <Array<BS, _, U>>::new(2).channel();
        loom::thread::spawn({
            let tx = tx.clone();
            move || tx.send2(0, sync).unwrap()
        });
        loom::thread::spawn({
            let tx = tx.clone();
            move || tx.send2(1, sync).unwrap()
        });
        loom::thread::spawn({
            let tx = tx.clone();
            move || tx.send2(2, sync).unwrap()
        });
        let mut values = (0..3).map(|_| rx.recv2(sync).unwrap()).collect::<Vec<_>>();
        values.sort_unstable();
        assert_eq!(values, [0, 1, 2]);
    });
}

#[cfg(loom)]
#[rstest]
fn loom_concurrent_close<const BS: usize, U: UnboundedBackoffStrategy>(
    #[values(false, true)] sync: bool,
    #[values(ONE, TWO)] bs: Usize<BS>,
    #[values(NO, EXP)] ub: Ub<U>,
) {
    let _ = (bs, ub);
    loom::model(move || {
        let (tx, mut rx) = <Array<BS, _, U>>::new(2).channel::<usize>();
        let h1 = loom::thread::spawn({
            let tx = tx.clone();
            move || tx.send2(0, sync)
        });
        let h2 = loom::thread::spawn({
            let tx = tx.clone();
            move || tx.send2(1, sync)
        });
        let h3 = loom::thread::spawn({
            let tx = tx.clone();
            move || tx.send2(2, sync)
        });
        loom::thread::spawn({
            let tx = tx.clone();
            move || tx.close()
        });
        let send = [h1, h2, h3]
            .into_iter()
            .flat_map(|h| Some(h.join().unwrap().err()?.0));
        let mut values: Vec<usize> = iter::from_fn(|| rx.recv2(sync).ok()).chain(send).collect();
        values.sort_unstable();
        assert_eq!(values, [0, 1, 2]);
    });
}
