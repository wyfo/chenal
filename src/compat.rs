#![allow(missing_docs)]
//! Compatibility wrappers to be used in place of other populars channel implementation.

#[cfg(feature = "compat-tokio")]
pub mod tokio {
    pub mod mpsc;
}
#[cfg(feature = "compat-std")]
pub mod std {
    pub mod mpsc;
}
