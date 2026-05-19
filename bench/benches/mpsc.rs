use std::{
    fmt::Debug,
    hint::black_box,
    sync::{
        atomic::{AtomicUsize, Ordering::Relaxed},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use criterion::{
    criterion_group, criterion_main, measurement::WallTime, BenchmarkGroup, Criterion,
};
use futures::executor::block_on;
use mpsc_bench::{AsyncReceiver, AsyncSender, BlockingReceiver, BlockingSender};

const CAPACITIES: &[usize] = &[8, 64, 256, 1024, 16384];

fn sender_counts() -> Vec<usize> {
    let mut sender_counts = vec![1, 2, 4];
    if let Ok(n) = thread::available_parallelism()
        && n.get() > 4
    {
        sender_counts.push(n.get() - 1);
    }
    sender_counts
}

pub(crate) struct SpinBarrier(AtomicUsize);

impl SpinBarrier {
    pub(crate) fn new(n: usize) -> Self {
        Self(AtomicUsize::new(n))
    }

    pub(crate) fn wait(&self) {
        self.0.fetch_sub(1, Relaxed);
        while self.0.load(Relaxed) != 0 {}
    }
}

#[derive(Clone)]
struct Blocking<T>(T);
#[derive(Clone)]
struct Async<T>(T);

trait Sender<T>: Clone + Send + 'static {
    fn run(self, count: usize);
}

impl<S: BlockingSender<T>, T: Default + Debug + Unpin + 'static> Sender<T> for Blocking<S> {
    fn run(mut self, count: usize) {
        for _ in 0..count {
            self.0.send(black_box(T::default()));
        }
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
}

pub trait Receiver<T> {
    fn run(self, count: usize);
}

impl<R: BlockingReceiver<T>, T: Default + Debug + Unpin + 'static> Receiver<T> for Blocking<R> {
    fn run(mut self, count: usize) {
        for _ in 0..count {
            black_box(self.0.recv());
        }
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
}

fn send_recv<T: Default + Debug + Unpin + 'static, S: Sender<T>, R: Receiver<T>>(
    channel: impl FnOnce(usize) -> (S, R),
    msg_count: usize,
    sender_count: usize,
    capacity: usize,
) -> Duration {
    let barrier = Arc::new(SpinBarrier::new(sender_count + 1));
    let (tx, rx) = channel(capacity);
    for _ in 0..sender_count {
        let tx = tx.clone();
        let barrier = barrier.clone();
        thread::spawn(move || {
            barrier.wait();
            tx.run(msg_count);
        });
    }
    let start = Instant::now();
    barrier.wait();
    rx.run(msg_count * sender_count);
    start.elapsed()
}

fn bench_channel<T: Default + Debug + Unpin + 'static, S: Sender<T>, R: Receiver<T>>(
    group: &mut BenchmarkGroup<WallTime>,
    name: &str,
    channel: impl Fn(usize) -> (S, R),
) {
    let capacities = if name.starts_with("fast") {
        CAPACITIES.iter().flat_map(|&c| [c / 2, c]).collect()
    } else {
        CAPACITIES.to_vec()
    };
    let msg_size = size_of::<T>() / 8;
    for sender_count in sender_counts() {
        for &capacity in &capacities {
            group.bench_function(
                format!("{name}/msg_size={msg_size}/senders={sender_count}/capacity={capacity}"),
                |b| b.iter_custom(|iters| send_recv(&channel, iters as _, sender_count, capacity)),
            );
        }
    }
}

macro_rules! bench_channel {
    ($group:ident, $($channel:ident),* $(,)?) => {$(
        bench_channel!(@ $group, $group, $channel);
    )*};
    (@ blocking_group, $group:ident, $channel:ident) => {
        bench_channel!(@ $group, $channel, blocking_channel, Blocking);
    };
    (@ async_group, $group:ident, $channel:ident) => {
        bench_channel!(@ $group, $channel, async_channel, Async);
    };
    (@ $group:ident, $channel:ident, $init:ident, $wrapper:ident) => {
        bench_channel!(@[0, 1, 2, 4, 8, 16] $group, $channel, $init, $wrapper);
    };
    (@[$($n:literal),+] $group:ident, $channel:ident, $init:ident, $wrapper:ident) => {$(
        bench_channel::<[usize; $n], _, _>(&mut $group, stringify!($channel), |capa| {
            let (tx, rx) = mpsc_bench::$channel::$init(capa);
            ($wrapper(tx), $wrapper(rx))
        });
    )+};
}

fn bench(c: &mut Criterion) {
    let mut blocking_group = c.benchmark_group("blocking");
    bench_channel!(
        blocking_group,
        chenal,
        chenal_loop,
        crossbeam,
        crossfire,
        flume,
        kanal,
        postage,
        std,
        tokio
    );
    blocking_group.finish();
    let mut async_group = c.benchmark_group("async");
    bench_channel!(
        async_group,
        async_channel,
        chenal,
        chenal_loop,
        crossfire,
        flume,
        futures,
        kanal,
        postage,
        tachyonix,
        tokio
    );
    async_group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
