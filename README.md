# chenal

[![crates.io](https://img.shields.io/crates/v/chenal.svg)](https://crates.io/crates/chenal)
[![docs.rs](https://docs.rs/chenal/badge.svg)](https://docs.rs/chenal)
[![License](https://img.shields.io/crates/l/chenal.svg)](LICENSE-MIT)
[![MSRV](https://img.shields.io/badge/rustc-1.87+-blue.svg)](https://blog.rust-lang.org/2025/05/15/Rust-1.87.0/)

Performant channel implementations.

*chenal is a French noun which translates to 'channel' in English.*

## *Work in progress*

*The crate is still at an early development stage. Only bounded channels are provided for the moment, but unbounded channels might be added in the future.*

## Example

```rust
use chenal::{errors::TryRecvError, mpsc};
use futures::executor::block_on;

let (tx, mut rx) = mpsc::channel(42);
std::thread::scope(|s| {
    s.spawn(|| block_on(tx.send(0)).unwrap());
    s.spawn(|| tx.send_blocking(1).unwrap());
    assert!(block_on(rx.recv()).unwrap() < 2);
    assert!(rx.recv_blocking().unwrap() < 2);
    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
});
```

## Comparison with other channel crates

### [`std::sync::mpsc`](https://doc.rust-lang.org/std/sync/mpsc/)

`std::sync::mpsc` wraps `crossbeam-channel` behind the scene, so it is discussed in the dedicated section.

### [`crossbeam_channel`](https://docs.rs/crossbeam-channel/latest/crossbeam_channel/)

`crossbeam_channel` is a synchronous MPMC channel and surely the most known in the ecosystem. It implements Dmitry Vyukov MPMC 
On its own benchmark, `chenal` is 2x faster than `crossbeam_channel` on average.

### [`tokio::sync::mpsc`](https://docs.rs/tokio/latest/tokio/sync/mpsc/)

Tokio's MPSC has notable differences from most other MPSC implementations such as `chenal`:
- it is de facto provided by the most ubiquitous asynchronous runtime of the Rust ecosystem
- it uses a lazily-allocated block-based algorithm
- it has a permit reservation system which provides fairness
- TODO budget

On the other hand, it's one of the least performant MPSC, especially due to a high number of atomic RMW operations.

### [`tachyonix`](https://docs.rs/tachyonix/latest/tachyonix/)

`tachyonix` is a standalone async-only MPSC. In `tachyonix`'s own benchmark, `chenal` is between 1.5x and 10x faster.

### [`kanal`](https://docs.rs/kanal/latest/kanal/) and [`flume`](https://docs.rs/flume/latest/flume/)

`kanal` is an MPMC channel inspired from Go. However, it uses a lock-based algorithm, which means every operation performs at least two atomic RMW operations, vs. one for `chenal`, not to mention other (spin-)lock drawbacks. In `kanal`'s own benchmark, `chenal` is 3x faster on average.

`kanal` is also not async-friendly, as its operations are not [cancel-safe](https://github.com/fereidani/kanal/issues/24).

### [`flume`](https://docs.rs/flume/latest/flume/)

`flume` is also a lock-based MPMC channel, but using 100% safe code. 

### [`crossfire`](https://docs.rs/crossfire/latest/crossfire/)

`crossfire`'s API is radically different from other channel crates, as it is heavily generic, supports multiple kinds of channels, and uses `MTx`/`Rx`/etc. instead of `Sender`/`Receiver`. It is obviously the main inspiration behind `chenal`'s API.

However, while `chenal` is carefully designed around hot path inlining, `crossfire` overuses inlining: `let _ = tx.send_blocking(msg)` compiles to 56 assembly lines with `chenal`, against 2148 assembly lines with `crossfire`.

`crossfire` also overuses backoff loops: calling `recv` on an empty channel will spin and yield to the OS many times before giving up and parking. While it *might* be good for throughput in highly contended benchmarks, it adds latency and disrupts the scheduler on each operation.

On highly contented benchmarks with small capacity, `crossfire` performs well, at the cost of calling 200k times `yield_now`. Yet, adding `try_xxx` backoff loops before blocking operations (roughly what `crossfire` does) makes `chenal` overtake `crossfire` in these benchmarks. And it performs significantly better without the backoff loops when the capacity is large enough. `tachyonix`'s benchmarks, which are more realistic, also give a clear advantage to `chenal` compared to `crossfire`.

Last but not least, `crossfire` doesn't allow closing a channel without dropping the receiver, preventing graceful termination without message loss. `chenal` not only allows it, but also every channel, even the SPSC one, can be closed from the receiver, the sender, or a dedicated `CloseHandle`.

## What makes `chenal` blazingly fast™?

This crate is built on top of two waiting primitives:
- [`aiq`](https://github.com/wyfo/aiq): an intrusive list with lock-free insertion
- [`spmc-waker`](https://github.com/wyfo/spmc-waker): a SPMC atomic waker with caching

When no waker is registered, both primitives have their hot path reduced to a single atomic load.

`chenal` code (as well as `aiq`'s and `spmc-waker`'s code) is carefully designed around hot path inlining. Each operation is compiled to a few dozen assembly instructions, while the outlined cold path has its generic parameters erased to be reused by different generic instances.

The algorithms are optimized to minimize atomic operations and contention. Each `send`/`recv` operation uses at most 1 atomic RMW operation in hot path. On x86_64 where `SeqCst` stores are expensive, they are downgraded to `Release` by adding an unbounded backoff loop on the receiver side.

Unlike the classical MPMC channel algorithm by Dmitry Vyukov (used in `crossbeam_channel` or `tachyonix`), channel slots are never written by the receiver, reducing contention on their cache lines. As a tradeoff, channel's capacity is limited to 2^31 on 64-bit platforms, which should be enough for most use cases.

## About benchmarks

Benchmarking can be hard, and benchmarking concurrent algorithms is notoriously hard. Good result in a benchmark necessarily doesn't necessarily mean the algorithm will be performant in real situation. On the other hand, an algorithm can perform badly in a benchmark because it is more performant.

## Safety and soundness

Like any other lock-free channel implementation, this crate uses unsafe code. It is [extensively tested](https://github.com/wyfo/chenal/actions/runs/26644156265) with [miri](https://github.com/rust-lang/miri) and [loom](https://github.com/tokio-rs/loom) to ensure its safety.

MPMC and SPMC channels' algorithms rely on an **undefined behavior** in the Rust memory model (not LLVM's one), which is known to [work in practice](https://github.com/rust-lang/unsafe-code-guidelines/blob/master/resources/deliberate-ub.md) and is used in other widespread algorithms like SeqLocks.

Progress on a sound alternative is tracked in [RFC 3301](https://github.com/rust-lang/rfcs/pull/3301)

## License

Licensed under either of

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT license](LICENSE-MIT)

at your option.