#![allow(clippy::semicolon_if_nothing_returned)]
use std::{fmt, panic, sync::Arc};

use chenal::compat::tokio::{
    mpsc,
    mpsc::error::{TryRecvError, TrySendError},
};
use tokio_test::*;

mod support {
    pub(crate) mod mpsc_stream {
        #![allow(dead_code)]

        use std::{
            pin::Pin,
            task::{Context, Poll},
        };

        use chenal::compat::tokio::{
            mpsc,
            mpsc::{Receiver, Sender},
        };
        use tokio_stream::Stream;

        // TODO uncomment when unbounded channel is implemented
        // struct UnboundedStream<T> {
        //     recv: UnboundedReceiver<T>,
        // }
        // impl<T> Stream for UnboundedStream<T> {
        //     type Item = T;
        //     fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<T>> {
        //         Pin::into_inner(self).recv.poll_recv(cx)
        //     }
        // }
        //
        // pub fn unbounded_channel_stream<T: Unpin>() -> (UnboundedSender<T>, impl Stream<Item = T>) {
        //     let (tx, rx) = mpsc::unbounded_channel();
        //
        //     let stream = UnboundedStream { recv: rx };
        //
        //     (tx, stream)
        // }

        struct BoundedStream<T> {
            recv: Receiver<T>,
        }
        impl<T> Stream for BoundedStream<T> {
            type Item = T;
            fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<T>> {
                Pin::into_inner(self).recv.poll_recv(cx)
            }
        }

        pub fn channel_stream<T: Unpin>(size: usize) -> (Sender<T>, impl Stream<Item = T>) {
            let (tx, rx) = mpsc::channel(size);

            let stream = BoundedStream { recv: rx };

            (tx, stream)
        }
    }
}

#[allow(unused)]
trait AssertRefUnwindSafe: panic::RefUnwindSafe {}
impl<T> AssertRefUnwindSafe for mpsc::Receiver<T> {}
impl<T> AssertRefUnwindSafe for mpsc::Sender<T> {}
// TODO uncomment when unbounded channel is implemented
// impl<T> AssertRefUnwindSafe for mpsc::UnboundedReceiver<T> {}
// impl<T> AssertRefUnwindSafe for mpsc::UnboundedSender<T> {}
impl<T> AssertRefUnwindSafe for mpsc::WeakSender<T> {}
// TODO uncomment when unbounded channel is implemented
// impl<T> AssertRefUnwindSafe for mpsc::WeakUnboundedSender<T> {}

#[allow(unused)]
trait AssertUnwindSafe: panic::UnwindSafe {}
impl<T> AssertUnwindSafe for mpsc::Receiver<T> {}
impl<T> AssertUnwindSafe for mpsc::Sender<T> {}
// TODO uncomment when unbounded channel is implemented
// impl<T> AssertUnwindSafe for mpsc::UnboundedReceiver<T> {}
// impl<T> AssertUnwindSafe for mpsc::UnboundedSender<T> {}
impl<T> AssertUnwindSafe for mpsc::WeakSender<T> {}
// impl<T> AssertUnwindSafe for mpsc::WeakUnboundedSender<T> {}

#[tokio::test]
async fn send_recv_stream_with_buffer() {
    use tokio_stream::StreamExt;

    let (tx, rx) = support::mpsc_stream::channel_stream::<i32>(16);
    let mut rx = Box::pin(rx);

    tokio::spawn(async move {
        assert_ok!(tx.send(1).await);
        assert_ok!(tx.send(2).await);
    });

    assert_eq!(Some(1), rx.next().await);
    assert_eq!(Some(2), rx.next().await);
    assert_eq!(None, rx.next().await);
}

#[tokio::test]
async fn async_send_recv_with_buffer() {
    let (tx, mut rx) = mpsc::channel(16);

    tokio::spawn(async move {
        assert_ok!(tx.send(1).await);
        assert_ok!(tx.send(2).await);
    });

    assert_eq!(Some(1), rx.recv().await);
    assert_eq!(Some(2), rx.recv().await);
    assert_eq!(None, rx.recv().await);
}

#[tokio::test]
async fn async_send_recv_many_with_buffer() {
    let (tx, mut rx) = mpsc::channel(2);
    let mut buffer = Vec::<i32>::with_capacity(3);

    // With `limit=0` does not sleep, returns immediately
    assert_eq!(0, rx.recv_many(&mut buffer, 0).await);

    let handle = tokio::spawn(async move {
        assert_ok!(tx.send(1).await);
        assert_ok!(tx.send(2).await);
        assert_ok!(tx.send(7).await);
        assert_ok!(tx.send(0).await);
    });

    let limit = 3;
    let mut recv_count = 0usize;
    while recv_count < 4 {
        recv_count += rx.recv_many(&mut buffer, limit).await;
        assert_eq!(buffer.len(), recv_count);
    }

    assert_eq!(vec![1, 2, 7, 0], buffer);
    assert_eq!(0, rx.recv_many(&mut buffer, limit).await);
    handle.await.unwrap();
}

#[test]
#[should_panic]
fn buffer_gteq_one() {
    mpsc::channel::<i32>(0);
}

// TODO uncomment when unbounded channel is implemented
// #[tokio::test]
// async fn send_recv_unbounded() {
//     let (tx, mut rx) = mpsc::unbounded_channel::<i32>();
//
//     // Using `try_send`
//     assert_ok!(tx.send(1));
//     assert_ok!(tx.send(2));
//
//     assert_eq!(rx.recv().await, Some(1));
//     assert_eq!(rx.recv().await, Some(2));
//
//     drop(tx);
//
//     assert!(rx.recv().await.is_none());
// }

// TODO uncomment when unbounded channel is implemented
// #[tokio::test]
// async fn send_recv_many_unbounded() {
//     let (tx, mut rx) = mpsc::unbounded_channel::<i32>();
//
//     let mut buffer: Vec<i32> = Vec::new();
//
//     // With `limit=0` does not sleep, returns immediately
//     rx.recv_many(&mut buffer, 0).await;
//     assert_eq!(0, buffer.len());
//
//     assert_ok!(tx.send(7));
//     assert_ok!(tx.send(13));
//     assert_ok!(tx.send(100));
//     assert_ok!(tx.send(1002));
//
//     rx.recv_many(&mut buffer, 0).await;
//     assert_eq!(0, buffer.len());
//
//     let mut count = 0;
//     while count < 4 {
//         count += rx.recv_many(&mut buffer, 1).await;
//     }
//     assert_eq!(count, 4);
//     assert_eq!(vec![7, 13, 100, 1002], buffer);
//     let final_capacity = buffer.capacity();
//     assert!(final_capacity > 0);
//
//     buffer.clear();
//
//     assert_ok!(tx.send(5));
//     assert_ok!(tx.send(6));
//     assert_ok!(tx.send(7));
//     assert_ok!(tx.send(2));
//
//     // Re-use existing capacity
//     count = rx.recv_many(&mut buffer, 32).await;
//
//     assert_eq!(final_capacity, buffer.capacity());
//     assert_eq!(count, 4);
//     assert_eq!(vec![5, 6, 7, 2], buffer);
//
//     drop(tx);
//
//     // recv_many will immediately return zero if the channel
//     // is closed and no more messages are waiting
//     assert_eq!(0, rx.recv_many(&mut buffer, 4).await);
//     assert!(rx.recv().await.is_none());
// }

#[tokio::test]
async fn send_recv_many_bounded_capacity() {
    let mut buffer: Vec<String> = Vec::with_capacity(9);
    let limit = buffer.capacity();
    let (tx, mut rx) = mpsc::channel(100);

    let mut expected: Vec<String> = (0..limit)
        .map(|x: usize| format!("{x}"))
        .collect::<Vec<_>>();
    for x in expected.clone() {
        tx.send(x).await.unwrap()
    }
    tx.send("one more".to_string()).await.unwrap();

    // Here `recv_many` receives all but the last value;
    // the initial capacity is adequate, so the buffer does
    // not increase in side.
    assert_eq!(buffer.capacity(), rx.recv_many(&mut buffer, limit).await);
    assert_eq!(expected, buffer);
    assert_eq!(limit, buffer.capacity());

    // Receive up more values:
    assert_eq!(1, rx.recv_many(&mut buffer, limit).await);
    assert!(buffer.capacity() > limit);
    expected.push("one more".to_string());
    assert_eq!(expected, buffer);

    tokio::spawn(async move {
        tx.send("final".to_string()).await.unwrap();
    });

    // 'tx' is dropped, but `recv_many` is guaranteed not
    // to return 0 as the channel has outstanding permits
    assert_eq!(1, rx.recv_many(&mut buffer, limit).await);
    expected.push("final".to_string());
    assert_eq!(expected, buffer);
    // The channel is now closed and `recv_many` returns 0.
    assert_eq!(0, rx.recv_many(&mut buffer, limit).await);
    assert_eq!(expected, buffer);
}

// TODO uncomment when unbounded channel is implemented
// #[tokio::test]
// async fn send_recv_many_unbounded_capacity() {
//     let mut buffer: Vec<String> = Vec::with_capacity(9); // capacity >= 9
//     let limit = buffer.capacity();
//     let (tx, mut rx) = mpsc::unbounded_channel();
//
//     let mut expected: Vec<String> = (0..limit)
//         .map(|x: usize| format!("{x}"))
//         .collect::<Vec<_>>();
//     for x in expected.clone() {
//         tx.send(x).unwrap()
//     }
//     tx.send("one more".to_string()).unwrap();
//
//     // Here `recv_many` receives all but the last value;
//     // the initial capacity is adequate, so the buffer does
//     // not increase in side.
//     assert_eq!(buffer.capacity(), rx.recv_many(&mut buffer, limit).await);
//     assert_eq!(expected, buffer);
//     assert_eq!(limit, buffer.capacity());
//
//     // Receive up more values:
//     assert_eq!(1, rx.recv_many(&mut buffer, limit).await);
//     assert!(buffer.capacity() > limit);
//     expected.push("one more".to_string());
//     assert_eq!(expected, buffer);
//
//     tokio::spawn(async move {
//         tx.send("final".to_string()).unwrap();
//     });
//
//     // 'tx' is dropped, but `recv_many` is guaranteed not
//     // to return 0 as the channel has outstanding permits
//     assert_eq!(1, rx.recv_many(&mut buffer, limit).await);
//     expected.push("final".to_string());
//     assert_eq!(expected, buffer);
//     // The channel is now closed and `recv_many` returns 0.
//     assert_eq!(0, rx.recv_many(&mut buffer, limit).await);
//     assert_eq!(expected, buffer);
// }

#[tokio::test]
async fn recv_many_with_non_empty_buffer_bounded_rx_closed_and_idle() {
    let (_tx, mut rx) = mpsc::channel::<i32>(1);

    let mut buffer: Vec<i32> = vec![1];

    rx.close();

    assert_eq!(0, rx.recv_many(&mut buffer, 1).await);
    assert_eq!(vec![1], buffer);
}

// TODO uncomment when unbounded channel is implemented
// #[tokio::test]
// async fn recv_many_with_non_empty_buffer_unbounded_rx_closed_and_idle() {
//     let (_tx, mut rx) = mpsc::unbounded_channel::<i32>();
//
//     let mut buffer: Vec<i32> = vec![1];
//
//     rx.close();
//
//     assert_eq!(0, rx.recv_many(&mut buffer, 1).await);
//     assert_eq!(vec![1], buffer);
// }

// TODO uncomment when unbounded channel is implemented
// #[tokio::test]
// async fn async_send_recv_unbounded() {
//     let (tx, mut rx) = mpsc::unbounded_channel();
//
//     tokio::spawn(async move {
//         assert_ok!(tx.send(1));
//         assert_ok!(tx.send(2));
//     });
//
//     assert_eq!(Some(1), rx.recv().await);
//     assert_eq!(Some(2), rx.recv().await);
//     assert_eq!(None, rx.recv().await);
// }

// TODO uncomment when unbounded channel is implemented
// #[tokio::test]
// async fn send_recv_stream_unbounded() {
//     use tokio_stream::StreamExt;
//
//     let (tx, rx) = support::mpsc_stream::unbounded_channel_stream::<i32>();
//
//     let mut rx = Box::pin(rx);
//
//     tokio::spawn(async move {
//         assert_ok!(tx.send(1));
//         assert_ok!(tx.send(2));
//     });
//
//     assert_eq!(Some(1), rx.next().await);
//     assert_eq!(Some(2), rx.next().await);
//     assert_eq!(None, rx.next().await);
// }

#[tokio::test]
async fn no_t_bounds_buffer() {
    struct NoImpls;

    let (tx, mut rx) = mpsc::channel(100);

    // sender should be Debug even though T isn't Debug
    is_debug(&tx);
    // same with Receiver
    is_debug(&rx);
    // and sender should be Clone even though T isn't Clone
    assert!(tx.clone().try_send(NoImpls).is_ok());

    assert!(rx.recv().await.is_some());
}

// TODO uncomment when unbounded channel is implemented
// #[tokio::test]
// async fn no_t_bounds_unbounded() {
//     struct NoImpls;
//
//     let (tx, mut rx) = mpsc::unbounded_channel();
//
//     // sender should be Debug even though T isn't Debug
//     is_debug(&tx);
//     // same with Receiver
//     is_debug(&rx);
//     // and sender should be Clone even though T isn't Clone
//     assert!(tx.clone().send(NoImpls).is_ok());
//
//     assert!(rx.recv().await.is_some());
// }

#[tokio::test]
async fn recv_close_gets_none_idle() {
    let (tx, mut rx) = mpsc::channel::<i32>(10);

    rx.close();

    assert!(rx.recv().await.is_none());

    assert_err!(tx.send(1).await);
}

#[tokio::test]
async fn tx_close_gets_none() {
    let (_, mut rx) = mpsc::channel::<i32>(10);
    assert!(rx.recv().await.is_none());
}

#[tokio::test]
async fn try_send_fail() {
    let (tx, mut rx) = mpsc::channel(1);

    tx.try_send("hello").unwrap();

    // This should fail
    match assert_err!(tx.try_send("fail")) {
        TrySendError::Full(..) => {}
        _ => panic!(),
    }

    assert_eq!(rx.recv().await, Some("hello"));

    assert_ok!(tx.try_send("goodbye"));
    drop(tx);

    assert_eq!(rx.recv().await, Some("goodbye"));
    assert!(rx.recv().await.is_none());
}

#[tokio::test]
async fn try_send_fail_with_try_recv() {
    let (tx, mut rx) = mpsc::channel(1);

    tx.try_send("hello").unwrap();

    // This should fail
    match assert_err!(tx.try_send("fail")) {
        TrySendError::Full(..) => {}
        _ => panic!(),
    }

    assert_eq!(rx.try_recv(), Ok("hello"));

    assert_ok!(tx.try_send("goodbye"));
    drop(tx);

    assert_eq!(rx.try_recv(), Ok("goodbye"));
    assert_eq!(rx.try_recv(), Err(TryRecvError::Disconnected));
}

#[test]
fn unconsumed_messages_are_dropped() {
    let msg = Arc::new(());

    let (tx, rx) = mpsc::channel(100);

    tx.try_send(msg.clone()).unwrap();

    assert_eq!(2, Arc::strong_count(&msg));

    drop((tx, rx));

    assert_eq!(1, Arc::strong_count(&msg));
}

#[test]

fn blocking_recv() {
    let (tx, mut rx) = mpsc::channel::<u8>(1);

    let sync_code = std::thread::spawn(move || {
        assert_eq!(Some(10), rx.blocking_recv());
    });

    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async move {
            let _ = tx.send(10).await;
        });
    sync_code.join().unwrap()
}

#[tokio::test]
#[should_panic]
async fn blocking_recv_async() {
    let (_tx, mut rx) = mpsc::channel::<()>(1);
    let _ = rx.blocking_recv();
}

#[test]
fn blocking_send() {
    let (tx, mut rx) = mpsc::channel::<u8>(1);

    let sync_code = std::thread::spawn(move || {
        tx.blocking_send(10).unwrap();
    });

    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async move {
            assert_eq!(Some(10), rx.recv().await);
        });
    sync_code.join().unwrap()
}

#[tokio::test]
#[should_panic]
async fn blocking_send_async() {
    let (tx, _rx) = mpsc::channel::<()>(1);
    let _ = tx.blocking_send(());
}

#[test]
fn try_recv_bounded() {
    let (tx, mut rx) = mpsc::channel(5);

    tx.try_send("hello").unwrap();
    tx.try_send("hello").unwrap();
    tx.try_send("hello").unwrap();
    tx.try_send("hello").unwrap();
    tx.try_send("hello").unwrap();
    assert!(tx.try_send("hello").is_err());

    assert_eq!(Ok("hello"), rx.try_recv());
    assert_eq!(Ok("hello"), rx.try_recv());
    assert_eq!(Ok("hello"), rx.try_recv());
    assert_eq!(Ok("hello"), rx.try_recv());
    assert_eq!(Ok("hello"), rx.try_recv());
    assert_eq!(Err(TryRecvError::Empty), rx.try_recv());

    tx.try_send("hello").unwrap();
    tx.try_send("hello").unwrap();
    tx.try_send("hello").unwrap();
    tx.try_send("hello").unwrap();
    assert_eq!(Ok("hello"), rx.try_recv());
    tx.try_send("hello").unwrap();
    tx.try_send("hello").unwrap();
    assert!(tx.try_send("hello").is_err());
    assert_eq!(Ok("hello"), rx.try_recv());
    assert_eq!(Ok("hello"), rx.try_recv());
    assert_eq!(Ok("hello"), rx.try_recv());
    assert_eq!(Ok("hello"), rx.try_recv());
    assert_eq!(Ok("hello"), rx.try_recv());
    assert_eq!(Err(TryRecvError::Empty), rx.try_recv());

    tx.try_send("hello").unwrap();
    tx.try_send("hello").unwrap();
    tx.try_send("hello").unwrap();
    drop(tx);
    assert_eq!(Ok("hello"), rx.try_recv());
    assert_eq!(Ok("hello"), rx.try_recv());
    assert_eq!(Ok("hello"), rx.try_recv());
    assert_eq!(Err(TryRecvError::Disconnected), rx.try_recv());
}

// TODO uncomment when unbounded channel is implemented
// #[test]
// fn try_recv_unbounded() {
//     for num in 0..100 {
//         let (tx, mut rx) = mpsc::unbounded_channel();
//
//         for i in 0..num {
//             tx.send(i).unwrap();
//         }
//
//         for i in 0..num {
//             assert_eq!(rx.try_recv(), Ok(i));
//         }
//
//         assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
//         drop(tx);
//         assert_eq!(rx.try_recv(), Err(TryRecvError::Disconnected));
//     }
// }

#[test]
fn try_recv_after_receiver_close() {
    let (_tx, mut rx) = mpsc::channel::<()>(5);

    assert_eq!(Err(TryRecvError::Empty), rx.try_recv());
    rx.close();
    assert_eq!(Err(TryRecvError::Disconnected), rx.try_recv());
}

#[test]
fn try_recv_close_while_empty_bounded() {
    let (tx, mut rx) = mpsc::channel::<()>(5);

    assert_eq!(Err(TryRecvError::Empty), rx.try_recv());
    drop(tx);
    assert_eq!(Err(TryRecvError::Disconnected), rx.try_recv());
}

// TODO uncomment when unbounded channel will be implemented
// #[test]
// fn try_recv_close_while_empty_unbounded() {
//     let (tx, mut rx) = mpsc::unbounded_channel::<()>();
//
//     assert_eq!(Err(TryRecvError::Empty), rx.try_recv());
//     drop(tx);
//     assert_eq!(Err(TryRecvError::Disconnected), rx.try_recv());
// }

#[cfg(feature = "compat-tokio-time")]
#[tokio::test(start_paused = true)]
async fn recv_timeout() {
    use std::time::Duration;

    use mpsc::error::SendTimeoutError::{Closed, Timeout};

    let (tx, rx) = mpsc::channel(5);

    assert_eq!(tx.send_timeout(10, Duration::from_secs(1)).await, Ok(()));
    assert_eq!(tx.send_timeout(20, Duration::from_secs(1)).await, Ok(()));
    assert_eq!(tx.send_timeout(30, Duration::from_secs(1)).await, Ok(()));
    assert_eq!(tx.send_timeout(40, Duration::from_secs(1)).await, Ok(()));
    assert_eq!(tx.send_timeout(50, Duration::from_secs(1)).await, Ok(()));
    assert_eq!(
        tx.send_timeout(60, Duration::from_secs(1)).await,
        Err(Timeout(60))
    );

    drop(rx);
    assert_eq!(
        tx.send_timeout(70, Duration::from_secs(1)).await,
        Err(Closed(70))
    );
}

#[cfg(feature = "compat-tokio-time")]
#[test]
#[should_panic = "there is no reactor running, must be called from the context of a Tokio 1.x runtime"]
fn recv_timeout_panic() {
    use std::time::Duration;

    use futures::future::FutureExt;

    let (tx, _rx) = mpsc::channel(5);
    tx.send_timeout(10, Duration::from_secs(1)).now_or_never();
}

#[tokio::test]
async fn test_rx_is_closed_when_calling_close_with_sender() {
    // is_closed should return true after calling close but still has a sender
    let (_tx, mut rx) = mpsc::channel::<()>(10);
    rx.close();

    assert!(rx.is_closed());
}

#[tokio::test]
async fn test_rx_is_closed_when_dropping_all_senders() {
    // is_closed should return true after dropping all senders
    let (tx, rx) = mpsc::channel::<()>(10);
    let another_tx = tx.clone();
    let task = tokio::spawn(async move {
        drop(another_tx);
    });

    drop(tx);
    let _ = task.await;

    assert!(rx.is_closed());
}

#[tokio::test]
async fn test_rx_is_not_closed_when_there_are_senders() {
    // is_closed should return false when there is a sender
    let (_tx, rx) = mpsc::channel::<()>(10);
    assert!(!rx.is_closed());
}

#[tokio::test]
async fn test_rx_is_not_closed_when_there_are_senders_and_buffer_filled() {
    // is_closed should return false when there is a sender, even if enough messages have been sent to fill the channel
    let (tx, rx) = mpsc::channel(10);
    for i in 0..10 {
        assert!(tx.send(i).await.is_ok());
    }
    assert!(!rx.is_closed());
}

#[tokio::test]
async fn test_rx_is_closed_when_there_are_no_senders_and_there_are_messages() {
    // is_closed should return true when there are messages in the buffer, but no senders
    let (tx, rx) = mpsc::channel(10);
    for i in 0..10 {
        assert!(tx.send(i).await.is_ok());
    }
    drop(tx);
    assert!(rx.is_closed());
}

#[tokio::test]
async fn test_rx_is_closed_when_there_are_messages_and_close_is_called() {
    // is_closed should return true when there are messages in the buffer, and close is called
    let (tx, mut rx) = mpsc::channel(10);
    for i in 0..10 {
        assert!(tx.send(i).await.is_ok());
    }
    rx.close();
    assert!(rx.is_closed());
}

#[tokio::test]
async fn test_rx_is_empty_when_no_messages_were_sent() {
    let (_tx, rx) = mpsc::channel::<()>(10);
    assert!(rx.is_empty())
}

#[tokio::test]
async fn test_rx_is_not_empty_when_there_are_messages_in_the_buffer() {
    let (tx, rx) = mpsc::channel::<()>(10);
    assert!(tx.send(()).await.is_ok());
    assert!(!rx.is_empty())
}

#[tokio::test]
async fn test_rx_is_not_empty_when_the_buffer_is_full() {
    let (tx, rx) = mpsc::channel(10);
    for i in 0..10 {
        assert!(tx.send(i).await.is_ok());
    }
    assert!(!rx.is_empty())
}

#[tokio::test]
async fn test_rx_is_not_empty_when_all_but_one_messages_are_consumed() {
    let (tx, mut rx) = mpsc::channel(10);
    for i in 0..10 {
        assert!(tx.send(i).await.is_ok());
    }

    for _ in 0..9 {
        assert!(rx.recv().await.is_some());
    }

    assert!(!rx.is_empty())
}

#[tokio::test]
async fn test_rx_is_empty_when_all_messages_are_consumed() {
    let (tx, mut rx) = mpsc::channel(10);
    for i in 0..10 {
        assert!(tx.send(i).await.is_ok());
    }
    while rx.try_recv().is_ok() {}
    assert!(rx.is_empty())
}

#[tokio::test]
async fn test_rx_is_empty_all_senders_are_dropped_and_messages_consumed() {
    let (tx, mut rx) = mpsc::channel(10);
    for i in 0..10 {
        assert!(tx.send(i).await.is_ok());
    }
    drop(tx);

    for _ in 0..10 {
        assert!(rx.recv().await.is_some());
    }

    assert!(rx.is_empty())
}

// TODO uncomment when channel len is implemented
// #[tokio::test]
// async fn test_rx_len_on_empty_channel() {
//     let (_tx, rx) = mpsc::channel::<()>(100);
//     assert_eq!(rx.len(), 0);
// }

// TODO uncomment when channel len is implemented
// #[tokio::test]
// async fn test_rx_len_on_empty_channel_without_senders() {
//     // when all senders are dropped, a "closed" value is added to the end of the linked list.
//     // here we test that the "closed" value does not change the len of the channel.
//
//     let (tx, rx) = mpsc::channel::<()>(100);
//     drop(tx);
//     assert_eq!(rx.len(), 0);
// }

// TODO uncomment when channel len is implemented
// #[tokio::test]
// async fn test_rx_len_on_filled_channel() {
//     let (tx, rx) = mpsc::channel(100);
//
//     for i in 0..100 {
//         assert!(tx.send(i).await.is_ok());
//     }
//     assert_eq!(rx.len(), 100);
// }

// TODO uncomment when channel len is implemented
// #[tokio::test]
// async fn test_rx_len_on_filled_channel_without_senders() {
//     let (tx, rx) = mpsc::channel(100);
//
//     for i in 0..100 {
//         assert!(tx.send(i).await.is_ok());
//     }
//     drop(tx);
//     assert_eq!(rx.len(), 100);
// }

// TODO uncomment when channel len is implemented
// #[tokio::test]
// async fn test_rx_len_when_consuming_all_messages() {
//     let (tx, mut rx) = mpsc::channel(100);
//
//     for i in 0..100 {
//         assert!(tx.send(i).await.is_ok());
//         assert_eq!(rx.len(), i + 1);
//     }
//
//     drop(tx);
//
//     for i in (0..100).rev() {
//         assert!(rx.recv().await.is_some());
//         assert_eq!(rx.len(), i);
//     }
// }

// TODO uncomment when channel len will be implemented
// #[tokio::test]
// async fn test_rx_len_when_close_is_called() {
//     let (tx, mut rx) = mpsc::channel(100);
//     tx.send(()).await.unwrap();
//     rx.close();
//
//     assert_eq!(rx.len(), 1);
// }

// TODO uncomment when channel len will be implemented
// #[tokio::test]
// async fn test_rx_len_when_close_is_called_before_dropping_sender() {
//     let (tx, mut rx) = mpsc::channel(100);
//     tx.send(()).await.unwrap();
//     rx.close();
//     drop(tx);
//
//     assert_eq!(rx.len(), 1);
// }

// TODO uncomment when channel len will be implemented
// #[tokio::test]
// async fn test_rx_len_when_close_is_called_after_dropping_sender() {
//     let (tx, mut rx) = mpsc::channel(100);
//     tx.send(()).await.unwrap();
//     drop(tx);
//     rx.close();
//
//     assert_eq!(rx.len(), 1);
// }

// TODO uncomment when unbounded channel will be implemented
// #[tokio::test]
// async fn test_rx_unbounded_is_closed_when_calling_close_with_sender() {
//     // is_closed should return true after calling close but still has a sender
//     let (_tx, mut rx) = mpsc::unbounded_channel::<()>();
//     rx.close();
//
//     assert!(rx.is_closed());
// }

// TODO uncomment when unbounded channel will be implemented
// #[tokio::test]
// async fn test_rx_unbounded_is_closed_when_dropping_all_senders() {
//     // is_closed should return true after dropping all senders
//     let (tx, rx) = mpsc::unbounded_channel::<()>();
//     let another_tx = tx.clone();
//     let task = tokio::spawn(async move {
//         drop(another_tx);
//     });
//
//     drop(tx);
//     let _ = task.await;
//
//     assert!(rx.is_closed());
// }

// TODO uncomment when unbounded channel will be implemented
// #[tokio::test]
// async fn test_rx_unbounded_is_not_closed_when_there_are_senders() {
//     // is_closed should return false when there is a sender
//     let (_tx, rx) = mpsc::unbounded_channel::<()>();
//     assert!(!rx.is_closed());
// }

// TODO uncomment when unbounded channel will be implemented
// #[tokio::test]
// async fn test_rx_unbounded_is_closed_when_there_are_no_senders_and_there_are_messages() {
//     // is_closed should return true when there are messages in the buffer, but no senders
//     let (tx, rx) = mpsc::unbounded_channel();
//     for i in 0..10 {
//         assert!(tx.send(i).is_ok());
//     }
//     drop(tx);
//     assert!(rx.is_closed());
// }

// TODO uncomment when unbounded channel will be implemented
// #[tokio::test]
// async fn test_rx_unbounded_is_closed_when_there_are_messages_and_close_is_called() {
//     // is_closed should return true when there are messages in the buffer, and close is called
//     let (tx, mut rx) = mpsc::unbounded_channel();
//     for i in 0..10 {
//         assert!(tx.send(i).is_ok());
//     }
//     rx.close();
//     assert!(rx.is_closed());
// }

// TODO uncomment when unbounded channel will be implemented
// #[tokio::test]
// async fn test_rx_unbounded_is_empty_when_no_messages_were_sent() {
//     let (_tx, rx) = mpsc::unbounded_channel::<()>();
//     assert!(rx.is_empty())
// }

// TODO uncomment when unbounded channel will be implemented
// #[tokio::test]
// async fn test_rx_unbounded_is_not_empty_when_there_are_messages_in_the_buffer() {
//     let (tx, rx) = mpsc::unbounded_channel();
//     assert!(tx.send(()).is_ok());
//     assert!(!rx.is_empty())
// }

// TODO uncomment when unbounded channel will be implemented
// #[tokio::test]
// async fn test_rx_unbounded_is_not_empty_when_all_but_one_messages_are_consumed() {
//     let (tx, mut rx) = mpsc::unbounded_channel();
//     for i in 0..10 {
//         assert!(tx.send(i).is_ok());
//     }
//
//     for _ in 0..9 {
//         assert!(rx.recv().await.is_some());
//     }
//
//     assert!(!rx.is_empty())
// }

// TODO uncomment when unbounded channel will be implemented
// #[tokio::test]
// async fn test_rx_unbounded_is_empty_when_all_messages_are_consumed() {
//     let (tx, mut rx) = mpsc::unbounded_channel();
//     for i in 0..10 {
//         assert!(tx.send(i).is_ok());
//     }
//     while rx.try_recv().is_ok() {}
//     assert!(rx.is_empty())
// }

// TODO uncomment when unbounded channel will be implemented
// #[tokio::test]
// async fn test_rx_unbounded_is_empty_all_senders_are_dropped_and_messages_consumed() {
//     let (tx, mut rx) = mpsc::unbounded_channel();
//     for i in 0..10 {
//         assert!(tx.send(i).is_ok());
//     }
//     drop(tx);
//
//     for _ in 0..10 {
//         assert!(rx.recv().await.is_some());
//     }
//
//     assert!(rx.is_empty())
// }

// TODO uncomment when unbounded channel will be implemented
// #[tokio::test]
// async fn test_rx_unbounded_len_on_empty_channel() {
//     let (_tx, rx) = mpsc::unbounded_channel::<()>();
//     assert_eq!(rx.len(), 0);
// }

// TODO uncomment when unbounded channel will be implemented
// #[tokio::test]
// async fn test_rx_unbounded_len_on_empty_channel_without_senders() {
//     // when all senders are dropped, a "closed" value is added to the end of the linked list.
//     // here we test that the "closed" value does not change the len of the channel.
//
//     let (tx, rx) = mpsc::unbounded_channel::<()>();
//     drop(tx);
//     assert_eq!(rx.len(), 0);
// }

// TODO uncomment when unbounded channel will be implemented
// #[tokio::test]
// async fn test_rx_unbounded_len_with_multiple_messages() {
//     let (tx, rx) = mpsc::unbounded_channel();
//
//     for i in 0..100 {
//         assert!(tx.send(i).is_ok());
//     }
//     assert_eq!(rx.len(), 100);
// }

// TODO uncomment when unbounded channel will be implemented
// #[tokio::test]
// async fn test_rx_unbounded_len_with_multiple_messages_and_dropped_senders() {
//     let (tx, rx) = mpsc::unbounded_channel();
//
//     for i in 0..100 {
//         assert!(tx.send(i).is_ok());
//     }
//     drop(tx);
//     assert_eq!(rx.len(), 100);
// }

// TODO uncomment when unbounded channel will be implemented
// #[tokio::test]
// async fn test_rx_unbounded_len_when_consuming_all_messages() {
//     let (tx, mut rx) = mpsc::unbounded_channel();
//
//     for i in 0..100 {
//         assert!(tx.send(i).is_ok());
//         assert_eq!(rx.len(), i + 1);
//     }
//
//     drop(tx);
//
//     for i in (0..100).rev() {
//         assert!(rx.recv().await.is_some());
//         assert_eq!(rx.len(), i);
//     }
// }

// TODO uncomment when unbounded channel will be implemented
// #[tokio::test]
// async fn test_rx_unbounded_len_when_close_is_called() {
//     let (tx, mut rx) = mpsc::unbounded_channel();
//     tx.send(()).unwrap();
//     rx.close();
//
//     assert_eq!(rx.len(), 1);
// }

// TODO uncomment when unbounded channel will be implemented
// #[tokio::test]
// async fn test_rx_unbounded_len_when_close_is_called_before_dropping_sender() {
//     let (tx, mut rx) = mpsc::unbounded_channel();
//     tx.send(()).unwrap();
//     rx.close();
//     drop(tx);
//
//     assert_eq!(rx.len(), 1);
// }

// TODO uncomment when unbounded channel will be implemented
// #[tokio::test]
// async fn test_rx_unbounded_len_when_close_is_called_after_dropping_sender() {
//     let (tx, mut rx) = mpsc::unbounded_channel();
//     tx.send(()).unwrap();
//     drop(tx);
//     rx.close();
//
//     assert_eq!(rx.len(), 1);
// }

// TODO uncomment when channel len will be implemented
// // Regression test for https://github.com/tokio-rs/tokio/issues/6602
// #[tokio::test]
// async fn test_is_empty_32_msgs() {
//     let (sender, mut receiver) = mpsc::channel(33);
//
//     for value in 1..257 {
//         sender.send(value).await.unwrap();
//         receiver.recv().await.unwrap();
//         assert!(receiver.is_empty(), "{value}. len: {}", receiver.len());
//     }
// }

// TODO uncomment when unbounded channel will be implemented
// #[test]
// #[cfg(not(panic = "abort"))]
// fn drop_all_elements_during_panic() {
//     use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
//
//     use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
//
//     static COUNTER: AtomicUsize = AtomicUsize::new(0);
//
//     struct A(bool);
//     impl Drop for A {
//         // cause a panic when inner value is `true`.
//         fn drop(&mut self) {
//             COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
//             if self.0 {
//                 panic!("panic!")
//             }
//         }
//     }
//
//     fn func(tx: UnboundedSender<A>, rx: UnboundedReceiver<A>) {
//         tx.send(A(true)).unwrap();
//         tx.send(A(false)).unwrap();
//         tx.send(A(false)).unwrap();
//
//         drop(rx);
//
//         // `mpsc::Rx`'s drop is called and gets panicked while dropping the first value,
//         // but will keep dropping following elements.
//     }
//
//     let (tx, rx) = mpsc::unbounded_channel();
//
//     let _ = panic::catch_unwind(panic::AssertUnwindSafe(|| {
//         func(tx.clone(), rx);
//     }));
//
//     // all A's destructor should be called at this point, even before `mpsc::Chan`'s
//     // drop gets called.
//     assert_eq!(COUNTER.load(Relaxed), 3);
//
//     drop(tx);
//     // `mpsc::Chan`'s drop is called, freeing the `Block` memory allocation.
// }

fn is_debug<T: fmt::Debug>(_: &T) {}
