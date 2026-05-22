use std::{
    cell::Cell,
    fmt::Debug,
    hint::black_box,
    panic::{catch_unwind, AssertUnwindSafe},
    sync::{
        atomic::{AtomicUsize, Ordering::Relaxed}, Arc,
        Once,
    },
    thread,
    time::{Duration, Instant},
};

use chenal_bench::{AsyncReceiver, AsyncSender, BlockingReceiver, BlockingSender};
use criterion::{criterion_group, criterion_main, Criterion};
use futures::executor::block_on;

const CAPACITIES: &[usize] = &[8, 64, 256, 1024, 16384];
const PARALLELISM: &[usize] = &[1, 2, 4, 8];

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

#[derive(Clone)]
struct Blocking<T>(T);
#[derive(Clone)]
struct Async<T>(T);

trait Sender<T>: Send + 'static {
    fn run(self, count: usize);
    fn clone(&self) -> Self;
}

impl<S: BlockingSender<T>, T: Default + Debug + Unpin + 'static> Sender<T> for Blocking<S> {
    fn run(mut self, count: usize) {
        for _ in 0..count {
            self.0.send(black_box(T::default()));
        }
    }
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<S: AsyncSender<T>, T: Default + Debug + Unpin + 'static> Sender<T> for Async<S> {
    fn run(mut self, count: usize) {
        block_on(async move {
            for _ in 0..count {
                self.0.send(black_box(T::default())).await;
            }
        });
    }
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub trait Receiver<T>: Send + 'static {
    fn run(self, count: usize);
    fn clone(&self) -> Self;
}

impl<R: BlockingReceiver<T>, T: Default + Debug + Unpin + 'static> Receiver<T> for Blocking<R> {
    fn run(mut self, count: usize) {
        for _ in 0..count {
            black_box(self.0.recv());
        }
    }
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<R: AsyncReceiver<T>, T: Default + Debug + Unpin + 'static> Receiver<T> for Async<R> {
    fn run(mut self, count: usize) {
        block_on(async move {
            for _ in 0..count {
                black_box(self.0.recv().await);
            }
        });
    }
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

fn send_recv<T: Default + Debug + Unpin + 'static, S: Sender<T>, R: Receiver<T>>(
    channel: impl FnOnce(usize) -> (S, R),
    msg_count: usize,
    sender_count: usize,
    receiver_count: usize,
    capacity: usize,
) -> Duration {
    let total_msgs = msg_count * sender_count * receiver_count;
    let timer = Arc::new(Timer::new(sender_count + receiver_count));
    let (tx, rx) = channel(capacity);
    for _ in 0..sender_count - 1 {
        let tx = tx.clone();
        timer.run(move || tx.run(total_msgs / sender_count));
    }
    timer.run(move || tx.run(total_msgs / sender_count));
    for _ in 0..receiver_count - 1 {
        let rx = rx.clone();
        timer.run(move || rx.run(total_msgs / receiver_count));
    }
    timer.run(move || rx.run(total_msgs / receiver_count));
    timer.wait()
}

fn bench_channel<T: Default + Debug + Unpin + 'static, S: Sender<T>, R: Receiver<T>>(
    c: &mut Criterion,
    name: &str,
    kind: &str,
    run: &str,
    channel: impl Fn(usize) -> (S, R),
) {
    let msg_size = size_of::<T>() / 8;
    for &sender_count in PARALLELISM {
        for &receiver_count in &[1] {
            for &capacity in CAPACITIES {
                let check_channel = || {
                    let (tx, rx) = channel(capacity);
                    if sender_count > 1 {
                        tx.clone();
                    }
                    if receiver_count > 1 {
                        rx.clone();
                    }
                };
                std::panic::set_hook(Box::new(|_| ()));
                if catch_unwind(AssertUnwindSafe(check_channel)).is_err() {
                    continue;
                }
                let _ = std::panic::take_hook();
                c.bench_function(
                    &format!("{name}/{kind}/{run}/msg_size={msg_size}/senders={sender_count}/receivers={receiver_count}/capacity={capacity}"),
                    |b| {
                        b.iter_custom(|iters| {
                            send_recv(&channel, iters as _, sender_count, 1, capacity)
                        })
                    },
                );
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
    (@[$($n:literal),+] $c:ident, $name:ident, $kind:ident, $run:ident, $channel:ident, $wrapper:ident) => {$(
        bench_channel::<[usize; $n], _, _>($c, stringify!($name), stringify!($kind), stringify!($run), |capa| {
            let (tx, rx) = chenal_bench::$name::$kind::$channel(capa);
            ($wrapper(tx), $wrapper(rx))
        });
    )+};
}

fn bench(c: &mut Criterion) {
    bench_channel!(c, chenal, mpsc, spsc);
    bench_channel!(c, chenal_loop, mpsc, spsc);
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
