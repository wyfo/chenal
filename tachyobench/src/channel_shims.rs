pub mod async_channel {
    use ::async_channel as channel;

    #[derive(Clone)]
    pub struct Sender<T> {
        inner: channel::Sender<T>,
    }
    impl<T> Sender<T> {
        pub async fn send(&mut self, message: T) {
            self.inner.send(message).await.unwrap();
        }
    }

    pub struct Receiver<T> {
        inner: channel::Receiver<T>,
    }
    impl<T> Receiver<T> {
        pub async fn recv(&mut self) -> Option<T> {
            self.inner.recv().await.ok()
        }
    }

    pub fn channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
        let (s, r) = channel::bounded(capacity);
        (Sender { inner: s }, Receiver { inner: r })
    }
}

pub mod flume {
    use ::flume as channel;

    #[derive(Clone)]
    pub struct Sender<T> {
        inner: channel::Sender<T>,
    }
    impl<T> Sender<T> {
        pub async fn send(&mut self, message: T) {
            self.inner.send_async(message).await.unwrap();
        }
    }

    pub struct Receiver<T> {
        inner: channel::Receiver<T>,
    }
    impl<T> Receiver<T> {
        pub async fn recv(&mut self) -> Option<T> {
            self.inner.recv_async().await.ok()
        }
    }

    pub fn channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
        let (s, r) = channel::bounded(capacity);
        (Sender { inner: s }, Receiver { inner: r })
    }
}

pub mod futures_mpsc {
    use std::fmt::Debug;

    use ::futures_channel::mpsc as channel;
    use ::futures_util::{sink::SinkExt, stream::StreamExt};

    #[derive(Clone)]
    pub struct Sender<T> {
        inner: channel::Sender<T>,
    }
    impl<T: Debug> Sender<T> {
        pub async fn send(&mut self, message: T) {
            self.inner.send(message).await.unwrap();
        }
    }

    pub struct Receiver<T> {
        inner: channel::Receiver<T>,
    }
    impl<T> Receiver<T> {
        pub async fn recv(&mut self) -> Option<T> {
            self.inner.next().await
        }
    }

    pub fn channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
        let (s, r) = channel::channel(capacity);
        (Sender { inner: s }, Receiver { inner: r })
    }
}

pub mod postage_mpsc {
    use std::fmt::Debug;

    use ::postage::{mpsc as channel, sink::Sink, stream::Stream};

    #[derive(Clone)]
    pub struct Sender<T> {
        inner: channel::Sender<T>,
    }
    impl<T: Debug> Sender<T> {
        pub async fn send(&mut self, message: T) {
            self.inner.send(message).await.unwrap();
        }
    }

    pub struct Receiver<T> {
        inner: channel::Receiver<T>,
    }
    impl<T> Receiver<T> {
        pub async fn recv(&mut self) -> Option<T> {
            self.inner.recv().await
        }
    }

    pub fn channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
        let (s, r) = channel::channel(capacity);
        (Sender { inner: s }, Receiver { inner: r })
    }
}

pub mod tachyonix {
    use ::tachyonix as channel;

    #[derive(Clone)]
    pub struct Sender<T> {
        inner: channel::Sender<T>,
    }
    impl<T: std::fmt::Debug> Sender<T> {
        pub async fn send(&mut self, message: T) {
            self.inner.send(message).await.unwrap();
        }
    }

    pub struct Receiver<T> {
        inner: channel::Receiver<T>,
    }
    impl<T> Receiver<T> {
        pub async fn recv(&mut self) -> Option<T> {
            self.inner.recv().await.ok()
        }
    }

    pub fn channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
        let (s, r) = channel::channel(capacity);
        (Sender { inner: s }, Receiver { inner: r })
    }
}

pub mod thingbuf {
    use ::thingbuf::mpsc as channel;

    #[derive(Clone)]
    pub struct Sender<T> {
        inner: channel::Sender<T>,
    }
    impl<T: std::fmt::Debug + Default + Clone> Sender<T> {
        pub async fn send(&mut self, message: T) {
            self.inner.send(message).await.unwrap();
        }
    }

    pub struct Receiver<T> {
        inner: channel::Receiver<T>,
    }
    impl<T: Default + Clone> Receiver<T> {
        pub async fn recv(&mut self) -> Option<T> {
            self.inner.recv().await
        }
    }

    pub fn channel<T: Default + Clone>(capacity: usize) -> (Sender<T>, Receiver<T>) {
        let (s, r) = channel::channel(capacity);
        (Sender { inner: s }, Receiver { inner: r })
    }
}

pub mod tokio_mpsc {
    use std::fmt::Debug;

    use ::tokio::sync::mpsc as channel;

    #[derive(Clone)]
    pub struct Sender<T> {
        inner: channel::Sender<T>,
    }
    impl<T: Debug> Sender<T> {
        pub async fn send(&mut self, message: T) {
            self.inner.send(message).await.unwrap();
        }
    }

    pub struct Receiver<T> {
        inner: channel::Receiver<T>,
    }
    impl<T> Receiver<T> {
        pub async fn recv(&mut self) -> Option<T> {
            self.inner.recv().await
        }
    }

    pub fn channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
        let (s, r) = channel::channel(capacity);
        (Sender { inner: s }, Receiver { inner: r })
    }
}

pub mod chenal {
    use ::chenal::mpsc as channel;
    // type Backoff = ::chenal::backoff::ExponentialBackoff<3, false, true>;
    type Backoff = ::chenal::backoff::NoBackoff;

    #[derive(Clone)]
    pub struct Sender<T> {
        inner: channel::MTx<T, Backoff>,
    }
    impl<T: std::fmt::Debug> Sender<T> {
        pub async fn send(&mut self, message: T) {
            self.inner.send(message).await.unwrap();
        }
    }

    pub struct Receiver<T> {
        inner: channel::Rx<T>,
    }
    impl<T> Receiver<T> {
        pub async fn recv(&mut self) -> Option<T> {
            self.inner.recv().await.ok()
        }
    }

    pub fn channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
        let (s, r) = channel::channel(capacity);
        (
            Sender {
                inner: s.with_backoff(),
            },
            Receiver { inner: r },
        )
    }
}

pub mod chenal_vyukov {
    // type Backoff = ::chenal::backoff::ExponentialBackoff<3, false, true>;
    type Backoff = ::chenal::backoff::NoBackoff;

    #[derive(Clone)]
    pub struct Sender<T> {
        inner: chenal::MTx<T, chenal::mpsc::VyukovMpsc, Backoff>,
    }
    impl<T: std::fmt::Debug> Sender<T> {
        pub async fn send(&mut self, message: T) {
            self.inner.send(message).await.unwrap();
        }
    }

    pub struct Receiver<T> {
        inner: chenal::Rx<T, chenal::mpsc::VyukovMpsc>,
    }
    impl<T> Receiver<T> {
        pub async fn recv(&mut self) -> Option<T> {
            self.inner.recv().await.ok()
        }
    }

    pub fn channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
        use chenal::Channel;
        let (s, r) = chenal::mpsc::VyukovMpsc::new(capacity).channel();
        (
            Sender {
                inner: s.with_backoff(),
            },
            Receiver { inner: r },
        )
    }
}

pub mod chenal_mpmc {
    use ::chenal::mpmc as channel;
    // type Backoff = ::chenal::backoff::ExponentialBackoff<3, false, true>;
    type Backoff = ::chenal::backoff::NoBackoff;

    #[derive(Clone)]
    pub struct Sender<T> {
        inner: channel::MTx<T, Backoff>,
    }
    impl<T: std::fmt::Debug> Sender<T> {
        pub async fn send(&mut self, message: T) {
            self.inner.send(message).await.unwrap();
        }
    }

    pub struct Receiver<T> {
        inner: channel::MRx<T, Backoff>,
    }
    impl<T> Receiver<T> {
        pub async fn recv(&mut self) -> Option<T> {
            self.inner.recv().await.ok()
        }
    }

    pub fn channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
        let (s, r) = channel::channel(capacity);
        (
            Sender {
                inner: s.with_backoff(),
            },
            Receiver {
                inner: r.with_backoff(),
            },
        )
    }
}

pub mod chenal_mpmc_racy {
    // type Backoff = ::chenal::backoff::ExponentialBackoff<3, false, true>;
    type Backoff = ::chenal::backoff::NoBackoff;

    #[derive(Clone)]
    pub struct Sender<T> {
        inner: chenal::MTx<T, chenal::mpmc::RacyArray, Backoff>,
    }
    impl<T: std::fmt::Debug> Sender<T> {
        pub async fn send(&mut self, message: T) {
            self.inner.send(message).await.unwrap();
        }
    }

    pub struct Receiver<T> {
        inner: chenal::MRx<T, chenal::mpmc::RacyArray, Backoff>,
    }
    impl<T> Receiver<T> {
        pub async fn recv(&mut self) -> Option<T> {
            self.inner.recv().await.ok()
        }
    }

    pub fn channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
        use chenal::Channel;
        let (s, r) = chenal::mpmc::RacyArray::new(capacity).channel();
        (
            Sender {
                inner: s.with_backoff(),
            },
            Receiver {
                inner: r.with_backoff(),
            },
        )
    }
}

pub mod chenal2 {
    // type Backoff = ::chenal::backoff::ExponentialBackoff<3, false, true>;
    type Backoff = ::chenal::backoff::NoBackoff;
    type Array = chenal::mpsc::Array<1, usize, { !chenal::DEFAULT_UNBOUNDED_BACKOFF }>;

    #[derive(Clone)]
    pub struct Sender<T> {
        inner: chenal::MTx<T, Array, Backoff>,
    }
    impl<T: std::fmt::Debug> Sender<T> {
        pub async fn send(&mut self, message: T) {
            self.inner.send(message).await.unwrap();
        }
    }

    pub struct Receiver<T> {
        inner: chenal::Rx<T, Array>,
    }
    impl<T> Receiver<T> {
        pub async fn recv(&mut self) -> Option<T> {
            self.inner.recv().await.ok()
        }
    }

    pub fn channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
        use chenal::Channel;
        let (s, r) = Array::new(capacity).channel();
        (
            Sender {
                inner: s.with_backoff(),
            },
            Receiver { inner: r },
        )
    }
}

pub mod chenal2_mpmc {
    // type Backoff = ::chenal::backoff::ExponentialBackoff<3, false, true>;
    type Backoff = ::chenal::backoff::NoBackoff;

    type Array = chenal::mpmc::Array<usize, { !chenal::DEFAULT_UNBOUNDED_BACKOFF }>;

    #[derive(Clone)]
    pub struct Sender<T> {
        inner: chenal::MTx<T, Array, Backoff>,
    }
    impl<T: std::fmt::Debug> Sender<T> {
        pub async fn send(&mut self, message: T) {
            self.inner.send(message).await.unwrap();
        }
    }

    pub struct Receiver<T> {
        inner: chenal::MRx<T, Array, Backoff>,
    }
    impl<T> Receiver<T> {
        pub async fn recv(&mut self) -> Option<T> {
            self.inner.recv().await.ok()
        }
    }

    pub fn channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
        use chenal::Channel;
        let (s, r) = Array::new(capacity).channel();
        (
            Sender {
                inner: s.with_backoff(),
            },
            Receiver {
                inner: r.with_backoff(),
            },
        )
    }
}

pub mod chenal2_mpmc_racy {
    // type Backoff = ::chenal::backoff::ExponentialBackoff<3, false, true>;
    type Backoff = ::chenal::backoff::NoBackoff;
    type Array = chenal::mpmc::RacyArray<usize, { !chenal::DEFAULT_UNBOUNDED_BACKOFF }>;

    #[derive(Clone)]
    pub struct Sender<T> {
        inner: chenal::MTx<T, Array, Backoff>,
    }
    impl<T: std::fmt::Debug> Sender<T> {
        pub async fn send(&mut self, message: T) {
            self.inner.send(message).await.unwrap();
        }
    }

    pub struct Receiver<T> {
        inner: chenal::MRx<T, Array, Backoff>,
    }
    impl<T> Receiver<T> {
        pub async fn recv(&mut self) -> Option<T> {
            self.inner.recv().await.ok()
        }
    }

    pub fn channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
        use chenal::Channel;
        let (s, r) = chenal::mpmc::RacyArray::new(capacity).channel();
        (
            Sender {
                inner: s.with_backoff(),
            },
            Receiver {
                inner: r.with_backoff(),
            },
        )
    }
}

pub mod crossfire {
    use ::crossfire as channel;

    #[derive(Clone)]
    pub struct Sender<T: Send + Unpin + 'static> {
        inner: channel::MAsyncTx<channel::mpsc::Array<T>>,
    }
    impl<T: Send + Unpin + std::fmt::Debug + 'static> Sender<T> {
        pub async fn send(&mut self, message: T) {
            self.inner.send(message).await.unwrap();
        }
    }

    pub struct Receiver<T: Send + Unpin + 'static> {
        inner: channel::AsyncRx<channel::mpsc::Array<T>>,
    }
    impl<T: Send + Unpin + 'static> Receiver<T> {
        pub async fn recv(&mut self) -> Option<T> {
            self.inner.recv().await.ok()
        }
    }

    pub fn channel<T: Send + Unpin + 'static>(capacity: usize) -> (Sender<T>, Receiver<T>) {
        let (s, r) = channel::mpsc::bounded_async(capacity);
        (Sender { inner: s }, Receiver { inner: r })
    }
}

pub mod kanal {
    use ::kanal as channel;

    #[derive(Clone)]
    pub struct Sender<T> {
        inner: channel::AsyncSender<T>,
    }
    impl<T> Sender<T> {
        pub async fn send(&mut self, message: T) {
            self.inner.send(message).await.unwrap();
        }
    }

    pub struct Receiver<T> {
        inner: channel::AsyncReceiver<T>,
    }
    impl<T> Receiver<T> {
        pub async fn recv(&mut self) -> Option<T> {
            self.inner.recv().await.ok()
        }
    }

    pub fn channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
        let (s, r) = channel::bounded_async(capacity);
        (Sender { inner: s }, Receiver { inner: r })
    }
}
