use std::{
    cell::Cell,
    collections::HashSet,
    fmt::Debug,
    hint::black_box,
    marker::PhantomData,
    sync::{
        atomic::{AtomicUsize, Ordering::Relaxed}, Arc, LazyLock, Mutex,
        Once,
    },
    thread,
    time::{Duration, Instant},
};

use chenal_bench::{
    AsyncReceiver, AsyncSender, BlockingReceiver, BlockingSender, Receiver, Sender,
};
use criterion::{criterion_group, criterion_main, Criterion};
use futures::executor::block_on;

const CAPACITIES: &[usize] = &[8, 64, 256, 1024, 16384];
const PARALLELISM: &[usize] = &[1, 2, 4, 8];
const MESSAGE_COUNT: usize = 1 << 16;

pub(crate) struct Timer {
    threads: usize,
    started: AtomicUsize,
    ended: AtomicUsize,
    start: Cell<Option<Instant>>,
    end: Cell<Option<Instant>>,
    finished: Once,
}
unsafe impl Send for Timer {}
unsafe impl Sync for Timer {}

impl Timer {
    fn new(threads: usize) -> Self {
        Self {
            threads,
            started: AtomicUsize::new(0),
            ended: AtomicUsize::new(0),
            start: Cell::new(None),
            end: Cell::new(None),
            finished: Once::new(),
        }
    }

    fn run(self: &Arc<Self>, f: impl FnOnce() + Send + 'static) {
        let this = self.clone();
        thread::spawn(move || {
            if this.started.fetch_add(1, Relaxed) == this.threads - 1 {
                this.start.set(Some(Instant::now()))
            } else {
                while this.started.load(Relaxed) != this.threads {}
            }
            f();
            if this.ended.fetch_add(1, Relaxed) == this.threads - 1 {
                this.end.set(Some(Instant::now()));
                this.finished.call_once(|| ());
            }
        });
    }

    fn wait(&self) -> Duration {
        self.finished.wait();
        let end = self.end.get().unwrap();
        let start = self.start.get().unwrap();
        end.duration_since(start)
    }
}

trait Runner<S, R> {
    fn run_send(sender: S, msg_count: usize);
    fn run_recv(receiver: R, msg_count: usize);
}

struct Blocking<T>(PhantomData<T>);
struct Async<T>(PhantomData<T>);

impl<T: Default, S: BlockingSender<T>, R: BlockingReceiver<T>> Runner<S, R> for Blocking<T> {
    fn run_send(mut sender: S, msg_count: usize) {
        let atomic = AtomicUsize::new(0);
        for _ in 0..msg_count {
            sender.send(black_box(T::default()));
            atomic.fetch_add(1, Relaxed);
        }
    }

    fn run_recv(mut receiver: R, msg_count: usize) {
        let atomic = AtomicUsize::new(0);
        for _ in 0..msg_count {
            black_box(receiver.recv());
            atomic.fetch_add(1, Relaxed);
        }
    }
}

impl<T: Default, S: AsyncSender<T>, R: AsyncReceiver<T>> Runner<S, R> for Async<T> {
    fn run_send(mut sender: S, msg_count: usize) {
        let atomic = AtomicUsize::new(0);
        block_on(async move {
            for _ in 0..msg_count {
                sender.send(black_box(T::default())).await;
                atomic.fetch_add(1, Relaxed);
            }
        })
    }

    fn run_recv(mut receiver: R, msg_count: usize) {
        let atomic = AtomicUsize::new(0);
        block_on(async move {
            for _ in 0..msg_count {
                black_box(receiver.recv().await);
                atomic.fetch_add(1, Relaxed);
            }
        })
    }
}

fn parallel<
    Run: Runner<S, R>,
    T: Default + Debug + Unpin + 'static,
    S: Sender<T>,
    R: Receiver<T>,
>(
    channel: impl Fn(usize) -> (S, R),
    sender_count: usize,
    receiver_count: usize,
    capacity: usize,
) -> Duration {
    let timer = Arc::new(Timer::new(sender_count + receiver_count));
    let (tx, rx) = channel(capacity);
    for _ in 0..sender_count - 1 {
        let tx = tx.clone();
        timer.run(move || Run::run_send(tx, MESSAGE_COUNT / sender_count));
    }
    timer.run(move || Run::run_send(tx, MESSAGE_COUNT / sender_count));
    for _ in 0..receiver_count - 1 {
        let rx = rx.clone();
        timer.run(move || Run::run_recv(rx, MESSAGE_COUNT / receiver_count));
    }
    timer.run(move || Run::run_recv(rx, MESSAGE_COUNT / receiver_count));
    timer.wait()
}

fn seq<T: Default + Debug + Unpin + 'static, S: Sender<T>, R: Receiver<T>>(
    channel: impl Fn(usize) -> (S, R),
    send: bool,
    recv: bool,
) -> Duration {
    let (mut tx, mut rx) = channel(MESSAGE_COUNT);
    let mut start = Instant::now();
    let atomic = AtomicUsize::new(0);
    for _ in 0..MESSAGE_COUNT {
        tx.try_send(black_box(T::default()));
        atomic.fetch_add(1, Relaxed);
    }
    if !recv {
        assert!(send);
        return start.elapsed();
    } else if !send {
        start = Instant::now();
    }
    for _ in 0..MESSAGE_COUNT {
        black_box(rx.try_recv());
        atomic.fetch_add(1, Relaxed);
    }
    start.elapsed()
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
    let msg_size = size_of::<T>() / 8;
    for (op, send, recv) in [
        ("seq", true, true),
        ("send", true, false),
        ("recv", false, true),
    ] {
        static SEQ: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(Default::default);
        let seq_bench = format!("{name}/{kind}/{op}/msg_size={msg_size}");
        if SEQ.lock().unwrap().insert(seq_bench.clone()) {
            c.bench_function(&seq_bench, |b| {
                b.iter_custom(|iters| (0..iters).map(|_| seq(&channel, send, recv)).sum())
            });
        }
    }
    let parallelism = |cloneable| if cloneable { PARALLELISM } else { &[1] };
    for &sender_count in parallelism(S::CLONEABLE) {
        for &receiver_count in parallelism(R::CLONEABLE) {
            for &capacity in CAPACITIES {
                let bench = format!(
                    "{name}/{kind}/{run}/msg_size={msg_size}/senders={sender_count}/receivers={receiver_count}/capacity={capacity}"
                );
                c.bench_function(&bench, |b| {
                    b.iter_custom(|iters| {
                        (0..iters)
                            .map(|_| parallel::<Run, T, S, R>(&channel, sender_count, 1, capacity))
                            .sum()
                    })
                });
            }
        }
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
        bench_channel!(@[0, 1, 2, 4, 8, 16] $c, $name, $kind, $run, $channel, $wrapper);
    };
    (@[$($n:literal),+] $c:ident, $name:ident, $kind:ident, $run:ident, $channel:ident, $runner:ident) => {$(
        bench_channel::<$runner<_>, [usize; $n], _, _>($c, stringify!($name), stringify!($kind), stringify!($run), chenal_bench::$name::$kind::$channel);
    )+};
}

fn bench(c: &mut Criterion) {
    bench_channel!(c, async_channel, mpmc(async));
    bench_channel!(c, chenal, mpmc, mpsc, spmc, spsc);
    bench_channel!(c, chenal_32, mpsc, spsc);
    bench_channel!(c, chenal_loop, mpmc, mpsc, spmc, spsc);
    bench_channel!(c, chenal_32, mpsc, spsc);
    bench_channel!(c, crossfire, mpmc, mpsc, spsc);
    bench_channel!(c, crossbeam, mpmc(blocking));
    bench_channel!(c, flume, mpmc);
    bench_channel!(c, kanal, mpmc);
    bench_channel!(c, postage, mpsc);
    bench_channel!(c, std, mpsc(blocking));
    bench_channel!(c, tokio, mpsc);
}

criterion_group!(benches, bench);
criterion_main!(benches);
