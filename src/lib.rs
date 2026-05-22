//! # chenal
//!
//! Performant channel implementations.
#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![no_std]
extern crate alloc;

mod array;
#[cfg(feature = "blocking")]
mod blocking;
pub mod capacity;
mod channel;
#[cfg(any(feature = "compat-tokio", feature = "compat-std",))]
pub mod compat;
pub mod errors;
mod internal;
mod loom;
#[cfg(feature = "mpmc")]
pub mod mpmc;
pub mod mpsc;
mod rc;
pub mod spmc;
pub mod spsc;
mod waiter;

/// Reexport of [`aiq::sync`].
pub use aiq::sync;
pub use channel::{
    BoundedChannel, Channel, ChannelHalf, ChannelId, CloseGuard, CloseHandle, MRx, MTx, Rx, Tx,
    UMTx, UTx, Weak,
};

/// Future types.
pub mod futures {
    pub use crate::channel::{ClosedFuture, RecvFuture, SendFuture};
}

/// Default value for the `UNBOUNDED_BACKOFF` MPSC/MPMC channel parameter.
///
/// On some platforms like `x86_64`, `SeqCst` stores are significantly more expensive than
/// `Release` stores. That's why MPSC and MPMC channels use by default a modified algorithm
/// where `SeqCst` stores are replaced by `Release` stores, at the cost of having an
/// unbounded backoff loop in `recv` operation. It means that under specific circumstances,
/// i.e., `recv` called while the next message is being sent, [`spin_loop`] and even
/// [`yield_now`] are called repeatedly until the `send` operation finishes.
///
/// If this behavior is not desired, channel's `UNBOUNDED_BACKOFF` parameter might be set
/// to `false`, trading performance for scheduler friendliness.
///
/// [`spin_loop`]: core::hint::spin_loop
/// [`yield_now`]: blocking::std::thread::yield_now
pub const DEFAULT_UNBOUNDED_BACKOFF: bool = cfg!(any(
    target_arch = "x86_64",
    target_arch = "x86",
    target_arch = "riscv64",
    target_arch = "riscv32",
    target_arch = "powerpc64",
    target_arch = "powerpc",
    target_arch = "arm",
));
