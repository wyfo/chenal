use core::{
    pin::pin,
    task::{Context, Poll, Waker},
};
use std::{sync::Arc, thread};

use chenal::{
    Channel, MRx, Tx,
    errors::{RecvError, SendError, TryRecvError, TrySendError},
    spmc::RacyArray,
};
#[cfg(not(loom))]
use futures::executor::block_on;
#[cfg(loom)]
use loom::future::block_on;
use rstest::rstest;

trait Send2<T> {
    fn send2(&mut self, msg: T, sync: bool) -> Result<(), SendError<T>>;
}
impl<T> Send2<T> for Tx<T, RacyArray<usize>> {
    fn send2(&mut self, msg: T, sync: bool) -> Result<(), SendError<T>> {
        if sync {
            self.send_blocking(msg)
        } else {
            block_on(self.send(msg))
        }
    }
}
trait Recv2<T> {
    fn recv2(&self, sync: bool) -> Result<T, RecvError>;
}
impl<T> Recv2<T> for MRx<T, RacyArray<usize>> {
    fn recv2(&self, sync: bool) -> Result<T, RecvError> {
        if sync {
            self.recv_blocking()
        } else {
            block_on(self.recv())
        }
    }
}

#[rstest]
fn spmc(#[values(false, true)] sync: bool) {
    let (mut tx, rx) = <RacyArray>::new(2).channel::<usize>();
    let mut values = thread::scope(|s| {
        let r1 = s.spawn(|| rx.recv2(sync).unwrap());
        let r2 = s.spawn(|| rx.recv2(sync).unwrap());
        let r3 = s.spawn(|| rx.recv2(sync).unwrap());
        for i in 0..3 {
            tx.send2(i, sync).unwrap();
        }
        vec![r1.join().unwrap(), r2.join().unwrap(), r3.join().unwrap()]
    });
    values.sort_unstable();
    assert_eq!(values, [0, 1, 2]);
}

// Sequential send/recv preserves FIFO order.
#[rstest]
fn sequential() {
    let (mut tx, rx) = <RacyArray>::new(2).channel::<usize>();
    tx.try_send(1).unwrap();
    tx.try_send(2).unwrap();
    assert_eq!(rx.try_recv(), Ok(1));
    assert_eq!(rx.try_recv(), Ok(2));
    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
}

// Ring buffer wraps around correctly across multiple laps.
#[rstest]
fn wrap_around() {
    let (mut tx, rx) = <RacyArray>::new(2).channel::<usize>();
    for i in 0..4 {
        tx.try_send(i).unwrap();
        assert_eq!(rx.try_recv(), Ok(i));
    }
}

// try_send on a full channel returns Full without blocking.
#[rstest]
fn try_send_full() {
    let (mut tx, rx) = <RacyArray>::new(1).channel::<usize>();
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
fn try_recv_empty() {
    let (mut tx, rx) = <RacyArray>::new(1).channel::<usize>();
    assert!(rx.is_empty());
    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
    tx.try_send(0).unwrap();
    assert!(!rx.is_empty());
    assert_eq!(rx.try_recv(), Ok(0));
    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
}

// Dropping all senders closes the channel; buffered messages are still readable.
#[rstest]
fn tx_drop_closes(#[values(false, true)] sync: bool) {
    let (mut tx, rx) = <RacyArray>::new(2).channel::<usize>();
    tx.try_send(42).unwrap();
    assert!(!rx.is_closed());
    drop(tx);
    assert!(rx.is_closed());
    assert_eq!(rx.recv2(sync), Ok(42));
    assert_eq!(rx.try_recv(), Err(TryRecvError::Closed));
    assert_eq!(rx.recv2(sync), Err(RecvError));
}

// Dropping the receiver closes the channel; sends return Closed.
#[rstest]
fn rx_drop_closes(#[values(false, true)] sync: bool) {
    let (mut tx, rx) = <RacyArray>::new(2).channel::<usize>();
    let rx2 = rx.clone();
    let weak = rx2.downgrade();
    assert!(!tx.is_closed());
    drop(rx2);
    assert!(!tx.is_closed());
    drop(rx);
    assert!(tx.is_closed());
    assert_eq!(tx.try_send(0), Err(TrySendError::Closed(0)));
    assert_eq!(tx.send2(0, sync), Err(SendError(0)));
    assert!(weak.upgrade().is_none());
}

// Dropping the sender while recv is blocked wakes the receiver with RecvError.
#[rstest]
fn tx_drop_while_recv_waiting(#[values(false, true)] sync: bool) {
    let (tx, rx) = <RacyArray>::new(1).channel::<usize>();
    let recv = thread::spawn(move || rx.recv2(sync));
    drop(tx);
    assert_eq!(recv.join().unwrap(), Err(RecvError));
}

// Dropping the receiver while send is blocked wakes the sender with SendError.
#[rstest]
fn rx_drop_while_send_waiting(#[values(false, true)] sync: bool) {
    let (mut tx, rx) = <RacyArray>::new(1).channel::<usize>();
    tx.try_send(0).unwrap();
    let send = thread::spawn(move || tx.send2(1, sync));
    drop(rx);
    assert_eq!(send.join().unwrap(), Err(SendError(1)));
}

// Closing the channel concurrently with sends; all messages are either received or returned as errors.
#[rstest]
fn concurrent_close(#[values(false, true)] sync: bool) {
    let (mut tx, rx) = <RacyArray>::new(2).channel::<usize>();
    let mut values = thread::scope(|s| {
        let r1 = s.spawn(|| rx.recv2(sync).ok());
        let r2 = s.spawn(|| rx.recv2(sync).ok());
        let r3 = s.spawn(|| rx.recv2(sync).ok());
        s.spawn(|| rx.close());
        let recv = [r1, r2, r3].into_iter().filter_map(|r| r.join().unwrap());
        (0..3)
            .filter_map(|i| Some(tx.send2(i, sync).err()?.0))
            .chain(recv)
            .collect::<Vec<_>>()
    });
    values.sort_unstable();
    assert_eq!(values, [0, 1, 2]);
}

// A canceled SendFuture does not deliver its message.
#[rstest]
fn send_future_cancel(#[values(false, true)] take: bool) {
    let (mut tx, rx) = <RacyArray>::new(1).channel::<usize>();
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
fn recv_future_cancel() {
    let (mut tx, rx) = <RacyArray>::new(1).channel::<usize>();
    {
        let mut fut = pin!(rx.recv());
        let mut cx = Context::from_waker(Waker::noop());
        assert_eq!(fut.as_mut().poll(&mut cx), Poll::Pending);
        tx.try_send(0).unwrap();
    }
    assert_eq!(rx.try_recv(), Ok(0));
}

// Canceling a notified RecvFuture passes the notification to the next waiter.
#[rstest]
fn recv_future_cancel_wakes_next() {
    let (mut tx, rx) = <RacyArray>::new(1).channel::<usize>();
    let rx2 = rx.clone();
    let mut cx = Context::from_waker(Waker::noop());
    let mut recv2 = pin!(rx2.recv());
    {
        let mut recv1 = pin!(rx.recv());
        assert_eq!(recv1.as_mut().poll(&mut cx), Poll::Pending);
        assert_eq!(recv2.as_mut().poll(&mut cx), Poll::Pending);
        tx.try_send(0).unwrap(); // notifies recv1
    } // recv1 dropped, should re-notify recv2
    assert_eq!(recv2.as_mut().poll(&mut cx), Poll::Ready(Ok(0)));
}

// SendFuture panics if polled after completion.
#[rstest]
#[should_panic(expected = "future polled after completion")]
fn send_future_poll_after_completion(#[values(false, true)] cancel: bool) {
    let (mut tx, rx) = <RacyArray>::new(1).channel::<usize>();
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
    let (mut tx, rx) = <RacyArray>::new(4).channel();
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
#[case::overflow((1 << (usize::BITS / 2 - 1)) + 1)]
#[should_panic]
fn invalid_capacity(#[case] capacity: usize) {
    <RacyArray>::new(capacity).channel::<usize>();
}

#[cfg(loom)]
#[rstest]
fn loom_spmc(#[values(false, true)] sync: bool) {
    loom::model(move || {
        let (mut tx, rx) = <RacyArray>::new(2).channel::<usize>();
        let r1 = loom::thread::spawn({
            let rx = rx.clone();
            move || rx.recv2(sync).unwrap()
        });
        let r2 = loom::thread::spawn({
            let rx = rx.clone();
            move || rx.recv2(sync).unwrap()
        });
        let r3 = loom::thread::spawn({
            let rx = rx.clone();
            move || rx.recv2(sync).unwrap()
        });
        for i in 0..3 {
            tx.send2(i, sync).unwrap();
        }
        let mut values = vec![r1.join().unwrap(), r2.join().unwrap(), r3.join().unwrap()];
        values.sort_unstable();
        assert_eq!(values, [0, 1, 2]);
    });
}

#[cfg(loom)]
#[rstest]
fn loom_concurrent_close(#[values(false, true)] sync: bool) {
    loom::model(move || {
        let (mut tx, rx) = <RacyArray>::new(2).channel::<usize>();
        let r1 = loom::thread::spawn({
            let rx = rx.clone();
            move || rx.recv2(sync).ok()
        });
        let r2 = loom::thread::spawn({
            let rx = rx.clone();
            move || rx.recv2(sync).ok()
        });
        let r3 = loom::thread::spawn({
            let rx = rx.clone();
            move || rx.recv2(sync).ok()
        });
        loom::thread::spawn(move || rx.close());
        let recv = [r1, r2, r3].into_iter().filter_map(|r| r.join().unwrap());
        let mut values: Vec<usize> = (0..3)
            .filter_map(|i| Some(tx.send2(i, sync).err()?.0))
            .chain(recv)
            .collect();
        values.sort_unstable();
        assert_eq!(values, [0, 1, 2]);
    });
}
