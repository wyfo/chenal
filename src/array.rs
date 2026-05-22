use alloc::boxed::Box;
use core::{cmp, ops::Deref, ptr, ptr::NonNull, slice};

use crate::internal;

pub(crate) const HB_SHIFT: u32 = usize::BITS / 2;
pub(crate) const LB: usize = usize::MAX >> HB_SHIFT;
pub(crate) const HB: usize = usize::MAX << HB_SHIFT;

fn slot_mask(capacity: usize) -> usize {
    (capacity.next_power_of_two() << 1).wrapping_sub(1)
}

pub(crate) struct Slots<S, C: internal::Capacity> {
    slots: NonNull<S>,
    capacity: C,
    mask: C::Mask,
}

impl<S, C: internal::Capacity> Slots<S, C> {
    pub(crate) fn new(capacity: C, mut slot: impl FnMut(usize, usize) -> S) -> Self {
        assert!(capacity.get() > 0, "capacity must not be zero");
        assert!(capacity.get() <= (LB >> 1), "capacity overflow");
        let lap = slot_mask(capacity.get()).wrapping_add(1);
        let slots = (0..capacity.get())
            .map(|i| slot(i, lap))
            .collect::<Box<[_]>>();
        Self {
            slots: NonNull::from(Box::leak(slots)).cast(),
            capacity,
            mask: capacity.mask(slot_mask),
        }
    }

    #[inline(always)]
    pub(crate) fn capacity(&self) -> usize {
        self.capacity.get()
    }

    #[inline(always)]
    pub(crate) fn slot_mask(&self) -> usize {
        C::get_mask(self.mask, slot_mask)
    }

    #[inline(always)]
    pub(crate) fn lap(&self) -> usize {
        self.slot_mask().wrapping_add(1)
    }

    #[inline(always)]
    pub(crate) fn new_lap(&self, state: usize, keep_hb: bool) -> usize {
        let new_state = (state & !self.slot_mask()).wrapping_add(self.lap());
        if keep_hb {
            new_state & LB | state & HB
        } else {
            new_state
        }
    }

    #[inline(always)]
    pub(crate) fn wrap_around(&self, idx: usize, state: usize, keep_hb: bool) -> usize {
        if idx != self.capacity() - 1 {
            state + 1
        } else {
            self.new_lap(state, keep_hb)
        }
    }

    #[inline(always)]
    pub(crate) fn closed_flag(&self) -> usize {
        (self.slot_mask() >> 1) + 1
    }

    pub(crate) fn slots_between(&self, head: usize, tail: usize) -> impl Iterator<Item = &S> {
        let tail_idx = tail & self.slot_mask();
        let head_idx = head & self.slot_mask();
        let (r1, r2) = match tail_idx.cmp(&head_idx) {
            cmp::Ordering::Less => (0..tail_idx, head_idx..self.capacity()),
            cmp::Ordering::Equal if head != tail => (0..self.capacity(), 0..0),
            cmp::Ordering::Equal => (0..0, 0..0),
            cmp::Ordering::Greater => (head_idx..tail_idx, 0..0),
        };
        r1.chain(r2).map(|i| unsafe { self.get_unchecked(i) })
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
