use std::{
    collections::{HashMap, HashSet},
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

const MESSAGE_COUNT: usize = 1024;
const MIN_SPIN: usize = 32;
const MAX_SPIN: usize = 64;

struct SpinBarrier(AtomicUsize);

impl SpinBarrier {
    fn new() -> Self {
        Self(AtomicUsize::new(1))
    }

    fn wait(&self) {
        self.0.fetch_sub(1, Relaxed);
        #[allow(clippy::missing_spin_loop)]
        while self.0.load(Relaxed) != 0 {}
    }

    fn wrap<R>(&self, f: impl FnOnce() -> R) -> impl FnOnce() -> R {
        self.0.fetch_add(1, Relaxed);
        || {
            self.wait();
            f()
        }
    }

    fn time<R>(&self, f: impl FnOnce() -> R) -> (R, Duration) {
        self.wait();
        let start = Instant::now();
        (f(), start.elapsed())
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

fn warm_channel<T: Default + Debug + Unpin + 'static, S: Sender<T>, R: Receiver<T>>(
    channel: impl Fn(usize) -> (S, R) + Copy + 'static,
) -> impl Fn(usize) -> (S, R) + Copy + 'static {
    move |capacity| {
        let (mut tx, mut rx) = channel(capacity);
        for _ in 0..capacity {
            tx.try_send(black_box(T::default()));
            black_box(rx.try_recv());
        }
        (tx, rx)
    }
}

fn bench_try_send<T: Default + Debug + Unpin + 'static, S: Sender<T>, R: Receiver<T>>(
    channel: impl Fn(usize) -> (S, R),
) -> Duration {
    let (mut tx, _rx) = channel(MESSAGE_COUNT);
    let start = Instant::now();
    for _ in 0..MESSAGE_COUNT {
        tx.try_send(black_box(T::default()));
        spin_loop();
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
        let rx = s.spawn(barrier.wrap(|| Run::run_recv(rx, MESSAGE_COUNT, MIN_SPIN)));
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
        let (_tx, res) = barrier.time(|| Run::run_send(tx, MESSAGE_COUNT, 0));
        for t in threads {
            t.join().unwrap();
        }
        rx.join().unwrap();
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
        let tx = s.spawn(barrier.wrap(|| Run::run_send(tx, MESSAGE_COUNT, MIN_SPIN)));
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
        let (_rx, res) = barrier.time(|| Run::run_recv(rx, MESSAGE_COUNT, 0));
        for t in threads {
            t.join().unwrap();
        }
        tx.join().unwrap();
        res
    })
}

type Group = String;
type Func = String;
type Benches = HashMap<Group, HashMap<Func, Box<dyn Fn() -> Duration>>>;

fn bench_channel<
    Run: Runner<S, R>,
    T: Default + Debug + Unpin + 'static,
    S: Sender<T>,
    R: Receiver<T>,
>(
    benches: &mut Benches,
    name: &str,
    kind: &str,
    run: &str,
    channel: impl Fn(usize) -> (S, R) + Copy + 'static,
) {
    let channel = warm_channel(channel);
    let mut insert = |group, f| {
        benches.entry(group).or_default().insert(name.into(), f);
    };
    let msg_size = size_of::<T>() / 8;
    static TRY: LazyLock<Mutex<HashSet<(String, String, usize)>>> = LazyLock::new(Default::default);
    let try_key = (name.to_string(), kind.to_string(), msg_size);
    if TRY.lock().unwrap().insert(try_key) {
        insert(
            format!("{kind}/try_send/msg_size={msg_size}"),
            Box::new(move || bench_try_send(channel)),
        );
        insert(
            format!("{kind}/try_recv/msg_size={msg_size}"),
            Box::new(move || bench_try_recv(channel)),
        );
    }
    if Run::ASYNC {
        insert(
            format!("{kind}/poll_send/msg_size={msg_size}"),
            Box::new(move || bench_poll_send::<Run, _, _, _>(channel)),
        );
        insert(
            format!("{kind}/poll_recv/msg_size={msg_size}"),
            Box::new(move || bench_poll_recv::<Run, _, _, _>(channel)),
        );
    }
    let contended: fn(bool) -> &'static [bool] = |cloneable| {
        if cloneable { &[false, true] } else { &[false] }
    };
    for &contended in contended(S::CLONEABLE) {
        insert(
            format!("{kind}/send_{run}/msg_size={msg_size}/contended={contended}"),
            Box::new(move || bench_send::<Run, _, _, _>(channel, contended)),
        );
    }
    for &contended in contended(R::CLONEABLE) {
        insert(
            format!("{kind}/recv_{run}/msg_size={msg_size}/contended={contended}"),
            Box::new(move || bench_recv::<Run, _, _, _>(channel, contended)),
        );
    }
}

macro_rules! bench_channel {
    ($benches:ident, $name:ident($run:ident), $($kind:ident),+ $(,)?) => {$(
        bench_channel!(@kind $run, $benches, $name, $kind);
    )+};
    ($benches:ident, $name:ident, $($kind:ident),+ $(,)?) => {$(
        bench_channel!(@kind async, $benches, $name, $kind);
        bench_channel!(@kind blocking, $benches, $name, $kind);
    )+};
    (@kind $run:ident, $benches:ident, $name:ident, mpmc) => {
        bench_channel!(@run $benches, $name, mpmc, $run);
        bench_channel!(@run $benches, $name, mpsc, $run);
        bench_channel!(@run $benches, $name, spmc, $run);
        bench_channel!(@run $benches, $name, spsc, $run);
    };
    (@kind $run:ident, $benches:ident, $name:ident, mpsc) => {
        bench_channel!(@run $benches, $name, mpsc, $run);
        bench_channel!(@run $benches, $name, spsc, $run);
    };
    (@run $benches:ident, $name:ident, $kind:ident, async) => {
        bench_channel!(@ $benches, $name, $kind, async, async_channel, Async);
    };
    (@run $benches:ident, $name:ident, $kind:ident, blocking) => {
        bench_channel!(@ $benches, $name, $kind, blocking, blocking_channel, Blocking);
    };
    (@ $benches:ident, $name:ident, $kind:ident, $run:ident, $channel:ident, $wrapper:ident) => {
        bench_channel!(@[1, 4] $benches, $name, $kind, $run, $channel, $wrapper);
    };
    (@[$($n:literal),+] $benches:ident, $name:ident, $kind:ident, $run:ident, $channel:ident, $runner:ident) => {$(
        bench_channel::<$runner<_>, [usize; $n], _, _>(&mut $benches, stringify!($name), stringify!($kind), stringify!($run), chenal_bench::$name::$kind::$channel);
    )+};
}

fn bench(c: &mut Criterion) {
    let mut benches: Benches = Benches::new();
    // bench_channel!(benches, async_channel(async), mpmc);
    bench_channel!(benches, chenal, mpmc);
    // bench_channel!(benches, chenal_ub, mpmc);
    // bench_channel!(benches, chenal_32, mpsc);
    // bench_channel!(benches, chenal_32_ub, mpsc);
    // bench_channel!(benches, crossfire, mpmc);
    // bench_channel!(benches, crossbeam(blocking), mpmc);
    // bench_channel!(benches, flume, mpmc);
    // bench_channel!(benches, kanal, mpmc);
    // bench_channel!(benches, postage, mpsc);
    // bench_channel!(benches, std(blocking), mpsc);
    // bench_channel!(benches, tokio, mpsc);
    // bench_channel!(benches, tachyonix(async), mpsc);
    for (group, funcs) in benches {
        let mut g = c.benchmark_group(group);
        for (func, f) in funcs {
            g.bench_function(func, |b| {
                b.iter_custom(|iters| (0..iters).map(|_| f()).sum())
            });
        }
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);
