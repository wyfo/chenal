use std::{
    collections::HashSet,
    fmt::Debug,
    hint::{black_box, spin_loop},
    marker::PhantomData,
    pin::pin,
    sync::{
        atomic::{AtomicUsize, Ordering::Relaxed}, LazyLock,
        Mutex,
    },
    task::{Context, Waker},
    thread,
    time::{Duration, Instant},
};

use chenal_bench::{
    AsyncReceiver, AsyncSender, BlockingReceiver, BlockingSender, Receiver, Sender,
};
use criterion::{criterion_group, criterion_main, Criterion};
use futures::executor::block_on;

const MESSAGE_COUNT: usize = 1000;
const MIN_SPIN: usize = 32;
const MAX_SPIN: usize = 64;

struct SpinBarrier(AtomicUsize);

impl SpinBarrier {
    fn new() -> Self {
        Self(AtomicUsize::new(1))
    }

    fn wait(&self) {
        self.0.fetch_sub(1, Relaxed);
        while self.0.load(Relaxed) != 0 {}
    }

    fn wrap<R>(&self, f: impl FnOnce() -> R) -> impl FnOnce() -> R {
        self.0.fetch_add(1, Relaxed);
        || {
            self.wait();
            f()
        }
    }

    fn time<R>(&self, f: impl FnOnce() -> R) -> Duration {
        self.wait();
        let start = Instant::now();
        f();
        start.elapsed()
    }
}

fn pause(count: usize) {
    for _ in 0..count {
        spin_loop();
    }
}

trait Runner<S, R> {
    const ASYNC: bool = false;
    fn send_future(_sender: &mut S) -> impl Future + '_ {
        async {}
    }
    fn recv_future(_receiver: &mut R) -> impl Future + '_ {
        async {}
    }
    fn run_send(sender: S, msg_count: usize, spin: usize) -> S;
    fn run_recv(receiver: R, msg_count: usize, spin: usize) -> R;
}

struct Blocking<T>(PhantomData<T>);
struct Async<T>(PhantomData<T>);

impl<T: Default, S: BlockingSender<T>, R: BlockingReceiver<T>> Runner<S, R> for Blocking<T> {
    fn run_send(mut sender: S, msg_count: usize, spin: usize) -> S {
        for _ in 0..msg_count {
            pause(spin);
            sender.send(black_box(T::default()));
        }
        sender
    }
    fn run_recv(mut receiver: R, msg_count: usize, spin: usize) -> R {
        for _ in 0..msg_count {
            pause(spin);
            black_box(receiver.recv());
        }
        receiver
    }
}

impl<T: Default, S: AsyncSender<T>, R: AsyncReceiver<T>> Runner<S, R> for Async<T> {
    const ASYNC: bool = true;
    fn send_future(sender: &mut S) -> impl Future + '_ {
        sender.send(black_box(T::default()))
    }

    fn recv_future(receiver: &mut R) -> impl Future + '_ {
        receiver.recv()
    }
    fn run_send(mut sender: S, msg_count: usize, spin: usize) -> S {
        block_on(async move {
            for _ in 0..msg_count {
                pause(spin);
                sender.send(black_box(T::default())).await;
            }
            sender
        })
    }
    fn run_recv(mut receiver: R, msg_count: usize, spin: usize) -> R {
        block_on(async move {
            for _ in 0..msg_count {
                pause(spin);
                black_box(receiver.recv().await);
            }
            receiver
        })
    }
}

fn bench_try_send<T: Default + Debug + Unpin + 'static, S: Sender<T>, R: Receiver<T>>(
    channel: impl Fn(usize) -> (S, R),
) -> Duration {
    let (mut tx, _rx) = channel(MESSAGE_COUNT);
    let start = Instant::now();
    for _ in 0..MESSAGE_COUNT {
        tx.try_send(black_box(T::default()));
    }
    start.elapsed()
}

fn bench_try_recv<T: Default + Debug + Unpin + 'static, S: Sender<T>, R: Receiver<T>>(
    channel: impl Fn(usize) -> (S, R),
) -> Duration {
    let (mut tx, mut rx) = channel(MESSAGE_COUNT);
    for _ in 0..MESSAGE_COUNT {
        tx.try_send(black_box(T::default()));
    }
    let start = Instant::now();
    for _ in 0..MESSAGE_COUNT {
        black_box(rx.try_recv());
    }
    start.elapsed()
}

fn bench_poll_send<
    Run: Runner<S, R>,
    T: Default + Debug + Unpin + 'static,
    S: Sender<T>,
    R: Receiver<T>,
>(
    channel: impl Fn(usize) -> (S, R),
) -> Duration {
    let (mut tx, _rx) = channel(MESSAGE_COUNT);
    for _ in 0..MESSAGE_COUNT {
        tx.try_send(black_box(T::default()));
    }
    let mut context = Context::from_waker(Waker::noop());
    let start = Instant::now();
    for _ in 0..MESSAGE_COUNT {
        let send = pin!(Run::send_future(&mut tx));
        assert!(send.poll(&mut context).is_pending());
    }
    start.elapsed()
}

fn bench_poll_recv<
    Run: Runner<S, R>,
    T: Default + Debug + Unpin + 'static,
    S: Sender<T>,
    R: Receiver<T>,
>(
    channel: impl Fn(usize) -> (S, R),
) -> Duration {
    let (_tx, mut rx) = channel(MESSAGE_COUNT);
    let mut context = Context::from_waker(Waker::noop());
    let start = Instant::now();
    for _ in 0..MESSAGE_COUNT {
        let recv = pin!(Run::recv_future(&mut rx));
        assert!(recv.poll(&mut context).is_pending());
    }
    start.elapsed()
}

fn bench_send<
    Run: Runner<S, R>,
    T: Default + Debug + Unpin + 'static,
    S: Sender<T>,
    R: Receiver<T>,
>(
    channel: impl Fn(usize) -> (S, R),
    contended: bool,
) -> Duration {
    let barrier = SpinBarrier::new();
    let (mut tx, rx) = channel(4 * MESSAGE_COUNT);
    for _ in 0..MESSAGE_COUNT {
        tx.try_send(black_box(T::default()));
    }
    thread::scope(|s| {
        let _rx = s.spawn(barrier.wrap(|| Run::run_recv(rx, MESSAGE_COUNT, MIN_SPIN)));
        let mut threads = vec![];
        if contended {
            threads.push(s.spawn({
                let tx = tx.clone();
                barrier.wrap(|| Run::run_send(tx, MESSAGE_COUNT, MIN_SPIN))
            }));
            threads.push(s.spawn({
                let tx = tx.clone();
                barrier.wrap(|| Run::run_send(tx, MESSAGE_COUNT, MAX_SPIN))
            }));
        }
        let res = barrier.time(|| Run::run_send(tx, MESSAGE_COUNT, 0));
        for t in threads {
            t.join().unwrap();
        }
        res
    })
}

fn bench_recv<
    Run: Runner<S, R>,
    T: Default + Debug + Unpin + 'static,
    S: Sender<T>,
    R: Receiver<T>,
>(
    channel: impl Fn(usize) -> (S, R),
    contended: bool,
) -> Duration {
    let barrier = SpinBarrier::new();
    let (mut tx, rx) = channel(4 * MESSAGE_COUNT);
    for _ in 0..3 * MESSAGE_COUNT {
        tx.try_send(black_box(T::default()));
    }
    thread::scope(|s| {
        let _tx = s.spawn(barrier.wrap(|| Run::run_send(tx, MESSAGE_COUNT, MIN_SPIN)));
        let mut threads = vec![];
        if contended {
            threads.push(s.spawn(barrier.wrap({
                let rx = rx.clone();
                || Run::run_recv(rx, MESSAGE_COUNT, MIN_SPIN)
            })));
            threads.push(s.spawn(barrier.wrap({
                let rx = rx.clone();
                || Run::run_recv(rx, MESSAGE_COUNT, MAX_SPIN)
            })));
        }
        let res = barrier.time(|| Run::run_recv(rx, MESSAGE_COUNT, 0));
        for t in threads {
            t.join().unwrap();
        }
        res
    })
}

fn bench_channel<
    Run: Runner<S, R>,
    T: Default + Debug + Unpin + 'static,
    S: Sender<T>,
    R: Receiver<T>,
>(
    c: &mut Criterion,
    name: &str,
    kind: &str,
    run: &str,
    channel: impl Fn(usize) -> (S, R),
) {
    fn bench(c: &mut Criterion, id: String, f: impl Fn() -> Duration) {
        c.bench_function(&id, |b| {
            b.iter_custom(|iters| (0..iters).map(|_| f()).sum())
        });
    }
    let msg_size = size_of::<T>() / 8;
    static TRY: LazyLock<Mutex<HashSet<(String, String, usize)>>> = LazyLock::new(Default::default);
    let try_key = (name.to_string(), kind.to_string(), msg_size);
    if TRY.lock().unwrap().insert(try_key) {
        let try_send = format!("{name}/{kind}/try_send/msg_size={msg_size}");
        bench(c, try_send, || bench_try_send(&channel));
        let try_recv = format!("{name}/{kind}/try_recv/msg_size={msg_size}");
        bench(c, try_recv, || bench_try_recv(&channel));
    }
    if Run::ASYNC {
        let try_send = format!("{name}/{kind}/poll_send/msg_size={msg_size}");
        bench(c, try_send, || bench_poll_send::<Run, _, _, _>(&channel));
        let try_recv = format!("{name}/{kind}/poll_recv/msg_size={msg_size}");
        bench(c, try_recv, || bench_poll_recv::<Run, _, _, _>(&channel));
    }
    let contended: fn(bool) -> &'static [bool] = |cloneable| {
        if cloneable { &[false, true] } else { &[false] }
    };
    for &contended in contended(S::CLONEABLE) {
        let send = format!("{name}/{kind}/send_{run}/msg_size={msg_size}/contended={contended}");
        bench(c, send, || bench_send::<Run, _, _, _>(&channel, contended));
    }
    for &contended in contended(R::CLONEABLE) {
        let recv = format!("{name}/{kind}/recv_{run}/msg_size={msg_size}/contended={contended}");
        bench(c, recv, || bench_recv::<Run, _, _, _>(&channel, contended));
    }
}

macro_rules! bench_channel {
    ($c:ident, $name:ident, $kind:ident($run:ident) $($tt:tt)*) => {
        bench_channel!(@ $c, $name, $kind, $run);
        bench_channel!($c, $name $($tt)*)
    };
    ($c:ident, $name:ident, $kind:ident $($tt:tt)*) => {
        bench_channel!($c, $name, $kind(async));
        bench_channel!($c, $name, $kind(blocking));
        bench_channel!($c, $name $($tt)*)
    };
    ($c:ident, $name:ident $(,)?) => {};
    (@ $c:ident, $name:ident, $kind:ident, async) => {
        bench_channel!(@ $c, $name, $kind, async, async_channel, Async);
    };
    (@ $c:ident, $name:ident, $kind:ident, blocking) => {
        bench_channel!(@ $c, $name, $kind, blocking, blocking_channel, Blocking);
    };
    (@ $c:ident, $name:ident, $kind:ident, $run:ident, $channel:ident, $wrapper:ident) => {
        bench_channel!(@[1, 4] $c, $name, $kind, $run, $channel, $wrapper);
    };
    (@[$($n:literal),+] $c:ident, $name:ident, $kind:ident, $run:ident, $channel:ident, $runner:ident) => {$(
        bench_channel::<$runner<_>, [usize; $n], _, _>($c, stringify!($name), stringify!($kind), stringify!($run), chenal_bench::$name::$kind::$channel);
    )+};
}

fn bench(c: &mut Criterion) {
    bench_channel!(c, chenal, mpsc);
    bench_channel!(c, async_channel, mpmc(async));
    bench_channel!(c, chenal, mpmc, mpsc, spmc, spsc);
    bench_channel!(c, chenal_32, mpsc, spsc);
    bench_channel!(c, crossfire, mpmc, mpsc, spsc);
    bench_channel!(c, crossbeam, mpmc(blocking));
    bench_channel!(c, flume, mpmc);
    bench_channel!(c, kanal, mpmc);
    bench_channel!(c, postage, mpsc);
    bench_channel!(c, std, mpsc(blocking));
    bench_channel!(c, tokio, mpsc);
    bench_channel!(c, tachyonix, mpsc(async));
}

criterion_group!(benches, bench);
criterion_main!(benches);
