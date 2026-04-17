#![no_std]
extern crate alloc;

#[cfg(feature = "blocking")]
mod blocking;
pub mod capacity;
pub mod channel;
pub mod errors;
mod loom;
mod macros;
pub mod mpmc;
pub mod mpsc;
mod private;
pub mod rx;
pub mod spmc;
pub mod spsc;
pub mod tx;
#[cfg(feature = "weak")]
pub mod weak;

pub use aiq::sync;
