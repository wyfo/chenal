use core::sync::atomic::AtomicUsize;

pub(crate) trait RefCount: 'static {
    fn one() -> Self;
    fn atomic(&self) -> Option<&AtomicUsize>;
}

impl RefCount for () {
    fn one() -> Self {}
    fn atomic(&self) -> Option<&AtomicUsize> {
        None
    }
}

impl RefCount for AtomicUsize {
    fn one() -> Self {
        AtomicUsize::new(1)
    }
    fn atomic(&self) -> Option<&AtomicUsize> {
        Some(self)
    }
}
