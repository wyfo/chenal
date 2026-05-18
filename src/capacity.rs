//! Bounded channel capacity abstraction.
use alloc::boxed::Box;
use core::{ops::Deref, ptr, ptr::NonNull, slice};

use crate::internal;

/// The capacity of a bounded channel.
///
/// Either a runtime `usize` or a compile-time [`ConstCapacity<N>`].
#[expect(private_bounds)]
pub trait Capacity: internal::Capacity {}

impl Capacity for usize {}

/// A compile-time capacity, enabling more aggressive optimizations.
#[derive(Clone, Copy)]
pub struct ConstCapacity<const N: usize>;

impl<const N: usize> Capacity for ConstCapacity<N> {}

pub(crate) struct Slots<S, C: internal::Capacity> {
    slots: NonNull<S>,
    capacity: C,
}

impl<S, C: internal::Capacity> Slots<S, C> {
    pub(crate) fn new(capacity: C, slot: impl FnMut(usize) -> S) -> Self {
        assert_ne!(capacity.get(), 0, "capacity must be greater than 0");
        let slots = (0..capacity.get()).map(slot).collect::<Box<[_]>>();
        Self {
            capacity,
            slots: NonNull::from(Box::leak(slots)).cast(),
        }
    }

    #[inline(always)]
    pub(crate) fn capacity(&self) -> usize {
        self.capacity.get()
    }
}

impl<S, C: internal::Capacity> Deref for Slots<S, C> {
    type Target = [S];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.slots.as_ptr(), self.capacity.get()) }
    }
}

impl<S, C: internal::Capacity> Drop for Slots<S, C> {
    fn drop(&mut self) {
        let slots = ptr::slice_from_raw_parts_mut(self.slots.as_ptr(), self.capacity.get());
        drop(unsafe { Box::from_raw(slots) });
    }
}
