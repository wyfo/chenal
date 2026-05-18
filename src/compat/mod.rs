#[cfg(feature = "compat-tokio")]
pub mod tokio {
    pub mod mpsc;
}
#[cfg(feature = "compat-std")]
pub mod std {
    pub mod mpsc;
}
#[cfg(feature = "compat-tachyonix")]
pub mod tachyonix;
