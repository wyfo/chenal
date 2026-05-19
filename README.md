# chenal

[![crates.io](https://img.shields.io/crates/v/chenal.svg)](https://crates.io/crates/chenal)
[![docs.rs](https://docs.rs/chenal/badge.svg)](https://docs.rs/chenal)
[![License](https://img.shields.io/crates/l/chenal.svg)](LICENSE-MIT)
[![MSRV](https://img.shields.io/badge/rustc-1.87+-blue.svg)](https://blog.rust-lang.org/2025/05/15/Rust-1.87.0/)

Performant channel implementations.

*chenal is a French noun which translates to 'channel' in English.*

## *Work in progress*

*The crate is still at an early development stage. Only a bounded MPSC algorithm is provided for the moment, but others will follow soon.*

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

### [`crossbeam_channel`](https://docs.rs/crossbeam-channel/latest/crossbeam_channel/) / [`std::sync::mpsc`](https://doc.rust-lang.org/std/sync/mpsc/)

While exposed as an MPSC channel in the standard library, the underlying `crossbeam_channel` crate is an MPMC, so it's not fair to compare it to `chenal` until its MPMC is released.

One notable difference from `std::sync::mpsc` is in the API; while `chenal` and most other MPSCs use `&mut self` methods on the receiver, `std::sync::mpsc::Receiver` uses `&self` but is `!Sync` in exchange.

### [`tokio::sync::mpsc`](https://docs.rs/tokio/latest/tokio/sync/mpsc/)

Tokio's MPSC has notable differences from most other MPSC implementations such as `chenal`:
- it is de facto provided by the most ubiquitous asynchronous runtime of the Rust ecosystem
- it uses a lazily-allocated block-based algorithm
- it has a permit reservation system which provides fairness

On the other hand, it's one of the least performant MPSC, especially due to a high number of atomic RMW operations.

### [`tachyonix`](https://docs.rs/tachyonix/latest/tachyonix/)

`tachyonix` is a standalone async-only MPSC "which only claim to fame is to be extremely fast". In `tachyonix`'s own benchmark, `chenal` is between 1.5x and 10x faster.

### [`kanal`](https://docs.rs/kanal/latest/kanal/)

`kanal` is an MPMC channel which pretends to be faster than any other competitor. However, it uses a lock-based algorithm, which means every operation performs at least two atomic RMW operations, vs. one for `chenal`, not to mention other (spin-)lock drawbacks. In `kanal`'s own benchmark, `chenal` is 3x faster on average.

`kanal` is also not async-friendly, as its operations are not [cancel-safe](https://github.com/fereidani/kanal/issues/24).

### [`crossfire`](https://docs.rs/crossfire/latest/crossfire/)

`crossfire`'s API is radically different from other channel crates, as it is heavily generic, supports multiple kinds of channels, and uses `MTx`/`Rx`/etc. instead of `Sender`/`Receiver`. It is obviously the main inspiration behind `chenal`'s API.

However, while `chenal` is carefully designed around hot path inlining, `crossfire` overuses inlining: while `let _ = tx.send(msg)` compiles to 56 assembly lines with `chenal`, the same code compiles to 2148 assembly lines with `crossfire`.

`crossfire` also overuses backoff loops: calling `recv` on an empty channel will spin and yield to the OS many times before giving up and parking. While it *might* be good for highly contended benchmarks, it adds latency and disrupts the scheduler on each operation.

Speaking of benchmarks, `crossfire` claims to have "pushed the speed to a level no one has gone before". But this was before `chenal`, as adding the same `try_xxx` backoff loops before blocking operations makes it overtake `crossfire` in all highly contended benchmarks. And it performs significantly better without the loops when the capacity is large enough. `tachyonix`'s benchmarks, which are more realistic, also give a clear advantage to `chenal` compared to `crossfire`.

Last but not least, while both algorithms are similar and use an unbounded backoff loop — because `SeqCst` stores are too expensive on x86_64 — `chenal`'s algorithm is optimized to not use this unbounded backoff loop on other architectures like aarch64.

## What makes `chenal` blazingly fast™?

This crate is built on top of two waiting primitives:
- [`aiq`](https://github.com/wyfo/aiq): an intrusive list with lock-free insertion
- [`spmc-waker`](https://github.com/wyfo/spmc-waker): a SPMC atomic waker with caching

When no waker is registered, both primitives require a single atomic load, with their waking path outlined in a cold function.

`chenal` code (as well as `aiq`'s and `spmc-waker`'s code) is carefully designed around hot path inlining. Each operation is compiled to a few dozen assembly instructions, while the outlined cold path has its generic parameters erased to be reused by different generic instances.

The algorithms are optimized to minimize contention and depend on the CPU architecture. Unlike the classical channel algorithm by Dmitry Vyukov (used in `crossbeam_channel` or `tachyonix`), channel slots are never written by the receiver, reducing contention on their cache lines.

On x86_64 where `SeqCst` stores are expensive, they are downgraded to `Release` by adding an unbounded backoff loop on the receiver side. On aarch64, the channel's tail is also never accessed by the receiver, putting no contention on the senders.

## Safety

Like any other lock-free channel implementation, this crate uses unsafe code. It is [extensively tested](https://github.com/wyfo/chenal/actions/runs/26091006905/job/76716454402) with [miri](https://github.com/rust-lang/miri) to ensure its safety.

## License

Licensed under either of

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT license](LICENSE-MIT)

at your option.