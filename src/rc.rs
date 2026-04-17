use core::sync::atomic::AtomicUsize;

pub(crate) trait RefCount: 'static {
    #[cfg(feature = "weak")]
    fn zero() -> Self;
    fn one() -> Self;
    fn atomic(&self) -> Option<&AtomicUsize>;
}

impl RefCount for () {
    #[cfg(feature = "weak")]
    fn zero() -> Self {}
    fn one() -> Self {}
    fn atomic(&self) -> Option<&AtomicUsize> {
        None
    }
}

impl RefCount for AtomicUsize {
    #[cfg(feature = "weak")]
    fn zero() -> Self {
        AtomicUsize::new(0)
    }
    fn one() -> Self {
        AtomicUsize::new(1)
    }
    fn atomic(&self) -> Option<&AtomicUsize> {
        Some(self)
    }
}
