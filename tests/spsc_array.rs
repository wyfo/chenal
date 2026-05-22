use core::{
    pin::pin,
    task::{Context, Poll, Waker},
};
use std::{sync::Arc, thread};

use chenal::{
    Channel, Rx, Tx,
    errors::{RecvError, SendError, TryRecvError, TrySendError},
    spsc::Array,
};
use futures::executor::block_on;
use rstest::rstest;

struct Usize<const USIZE: usize>;
const ONE: Usize<1> = Usize::<1>;
const TWO: Usize<2> = Usize::<2>;

trait Send2<T> {
    fn send2(&mut self, msg: T, sync: bool) -> Result<(), SendError<T>>;
}
impl<T, const BLOCK_SIZE: usize> Send2<T> for Tx<T, Array<BLOCK_SIZE, usize>> {
    fn send2(&mut self, msg: T, sync: bool) -> Result<(), SendError<T>> {
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
impl<T, const BLOCK_SIZE: usize> Recv2<T> for Rx<T, Array<BLOCK_SIZE, usize>> {
    fn recv2(&mut self, sync: bool) -> Result<T, RecvError> {
        if sync {
            self.recv_blocking()
        } else {
            block_on(self.recv())
        }
    }
}

// Sequential send/recv preserves FIFO order.
#[test]
fn sequential() {
    let (mut tx, mut rx) = <Array>::new(2).channel();
    tx.try_send(1).unwrap();
    tx.try_send(2).unwrap();
    assert_eq!(rx.try_recv(), Ok(1));
    assert_eq!(rx.try_recv(), Ok(2));
    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
}

// Ring buffer wraps around correctly across multiple laps.
#[rstest]
fn wrap_around<const BS: usize>(#[values(ONE, TWO)] bs: Usize<BS>) {
    let _ = bs;
    let (mut tx, mut rx) = <Array<BS>>::new(2).channel();
    for i in 0..4 {
        tx.try_send(i).unwrap();
        assert_eq!(rx.try_recv(), Ok(i));
    }
}

#[rstest]
fn spsc<const BS: usize>(#[values(false, true)] sync: bool, #[values(ONE, TWO)] bs: Usize<BS>) {
    let _ = bs;
    let (mut tx, mut rx) = <Array<BS>>::new(2).channel();
    let values = thread::scope(|s| {
        s.spawn(|| {
            tx.send2(0, sync).unwrap();
            tx.send2(1, sync).unwrap();
            tx.send2(2, sync).unwrap();
        });
        (0..3).map(|_| rx.recv2(sync).unwrap()).collect::<Vec<_>>()
    });
    assert_eq!(values, [0, 1, 2]);
}

// try_send on a full channel returns Full without blocking.
#[test]
fn try_send_full() {
    let (mut tx, mut rx) = <Array>::new(1).channel();
    assert!(!tx.is_full());
    tx.try_send(0).unwrap();
    assert!(tx.is_full());
    assert_eq!(tx.try_send(1), Err(TrySendError::Full(1)));
    assert_eq!(rx.try_recv(), Ok(0));
    tx.try_send(1).unwrap();
    assert_eq!(tx.try_send(2), Err(TrySendError::Full(2)));
}

// try_recv on an empty channel returns Empty without blocking.
#[test]
fn try_recv_empty() {
    let (mut tx, mut rx) = <Array>::new(1).channel::<usize>();
    assert!(rx.is_empty());
    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
    tx.try_send(0).unwrap();
    assert!(!rx.is_empty());
    assert_eq!(rx.try_recv(), Ok(0));
    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
}

// Dropping the sender closes the channel; buffered messages are still readable.
#[rstest]
fn tx_drop_closes(#[values(false, true)] sync: bool) {
    let (mut tx, mut rx) = <Array>::new(2).channel();
    assert!(!rx.is_closed());
    tx.try_send(42).unwrap();
    drop(tx);
    assert!(rx.is_closed());
    assert_eq!(rx.recv2(sync), Ok(42));
    assert_eq!(rx.try_recv(), Err(TryRecvError::Closed));
    assert_eq!(rx.recv2(sync), Err(RecvError));
}

// Dropping the receiver closes the channel; sends return Closed.
#[rstest]
fn rx_drop_closes(#[values(false, true)] sync: bool) {
    let (mut tx, rx) = <Array>::new(2).channel::<usize>();
    assert!(!tx.is_closed());
    drop(rx);
    assert!(tx.is_closed());
    assert_eq!(tx.try_send(0), Err(TrySendError::Closed(0)));
    assert_eq!(tx.send2(0, sync), Err(SendError(0)));
}

// Dropping the sender while recv is blocked wakes the receiver with RecvError.
#[rstest]
fn tx_drop_while_recv_waiting(#[values(false, true)] sync: bool) {
    let (tx, mut rx) = <Array>::new(1).channel::<usize>();
    let recv = thread::spawn(move || rx.recv2(sync));
    drop(tx);
    assert_eq!(recv.join().unwrap(), Err(RecvError));
}

// Dropping the receiver while send is blocked wakes the sender with SendError.
#[rstest]
fn rx_drop_while_send_waiting(#[values(false, true)] sync: bool) {
    let (mut tx, rx) = <Array>::new(1).channel();
    tx.try_send(0).unwrap();
    let send = thread::spawn(move || tx.send2(1, sync));
    drop(rx);
    assert_eq!(send.join().unwrap(), Err(SendError(1)));
}

// Closing the channel concurrently with sends; all messages are either received or returned as errors.
#[rstest]
fn concurrent_close<const BS: usize>(
    #[values(false, true)] sync: bool,
    #[values(ONE, TWO)] bs: Usize<BS>,
) {
    let _ = bs;
    let (mut tx, mut rx) = <Array<BS>>::new(2).channel();
    let close_handle = tx.close_handle();
    let (sent, received) = thread::scope(|s| {
        let sent = s.spawn(|| tx.send2(0, sync));
        let received = s.spawn(|| rx.recv2(sync));
        s.spawn(|| close_handle.close());
        (sent.join().unwrap(), received.join().unwrap())
    });
    assert!(matches!(
        (sent, received),
        (Ok(()), Ok(0)) | (Err(SendError(0)), Err(RecvError))
    ));
}

// A canceled SendFuture does not deliver its message.
#[rstest]
fn send_future_cancel(#[values(false, true)] take: bool) {
    let (mut tx, mut rx) = <Array>::new(1).channel();
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
#[test]
fn recv_future_cancel() {
    let (mut tx, mut rx) = <Array>::new(1).channel();
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
    let (mut tx, mut rx) = <Array>::new(1).channel();
    let mut cx = Context::from_waker(Waker::noop());
    let mut fut = pin!(tx.send(0));
    if cancel {
        assert_eq!(fut.as_mut().cancel(), Some(0));
    } else {
        assert_eq!(fut.as_mut().poll(&mut cx), Poll::Ready(Ok(())));
        assert_eq!(rx.try_recv(), Ok(0));
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
    let (mut tx, mut rx) = <Array>::new(4).channel();
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
    <Array<BS>>::new(capacity).channel::<()>();
}

// Capacity is reduced by partially read block.
#[rstest]
#[case(ONE, false)]
#[case(TWO, true)]
fn partial_block_withholds_capacity<const BS: usize>(#[case] bs: Usize<BS>, #[case] reduced: bool) {
    let _ = bs;
    let (mut tx, mut rx) = <Array<BS>>::new(4).channel();
    for i in 0..4 {
        tx.try_send(i).unwrap();
    }
    assert_eq!(rx.try_recv(), Ok(0));
    assert_eq!(tx.try_send(4).is_err(), reduced);
}

// Reading the last slot of a block wakes the blocked sender.
#[test]
fn block_completion_wakes_sender() {
    let (mut tx, mut rx) = <Array<2>>::new(2).channel();
    tx.try_send(0).unwrap();
    tx.try_send(1).unwrap();
    let mut send = pin!(tx.send(2));
    let mut cx = Context::from_waker(Waker::noop());
    assert!(send.as_mut().poll(&mut cx).is_pending());
    assert_eq!(rx.try_recv(), Ok(0));
    assert!(send.as_mut().poll(&mut cx).is_pending()); // block not complete
    assert_eq!(rx.try_recv(), Ok(1)); // completes the block
    assert_eq!(send.as_mut().poll(&mut cx), Poll::Ready(Ok(())));
    assert_eq!(rx.try_recv(), Ok(2));
}
