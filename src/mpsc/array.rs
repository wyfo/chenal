use core::{
    cmp, iter,
    marker::PhantomData,
    mem::MaybeUninit,
    ptr::NonNull,
    sync::atomic::{
        AtomicUsize,
        Ordering::{Acquire, Relaxed, Release, SeqCst},
    },
};

use aiq::WaitQueue;
use crossbeam_utils::Backoff;
use spmc_waker::SpmcWaker;

use crate::{
    Channel, DEFAULT_UNBOUNDED_BACKOFF, MTx, Rx,
    capacity::{Capacity, Slots},
    channel::{BoundedChannel, Chan},
    errors::{SendError, TryAcquireError},
    internal,
    loom::{AtomicUsizeExt, UnsafeCellExt, cell::UnsafeCell},
    sync::{DefaultSyncPrimitives, SyncPrimitives},
};

const LB: usize = usize::MAX >> HB_SHIFT;
const HB_SHIFT: u32 = usize::BITS / 2;

pub struct Array<
    C: Capacity = usize,
    const UNBOUNDED_BACKOFF: bool = DEFAULT_UNBOUNDED_BACKOFF,
    SP: SyncPrimitives = DefaultSyncPrimitives,
> {
    capacity: C,
    sync: PhantomData<SP>,
}

impl<C: Capacity, const UNBOUNDED_BACKOFF: bool, SP: SyncPrimitives>
    Array<C, UNBOUNDED_BACKOFF, SP>
{
    pub fn new(capacity: C) -> Self {
        Self {
            capacity,
            sync: PhantomData,
        }
    }
}

impl<C: Capacity, const UNBOUNDED_BACKOFF: bool, SP: SyncPrimitives> Channel
    for Array<C, UNBOUNDED_BACKOFF, SP>
{
    type TxHalf<T> = MTx<T, Self>;
    type RxHalf<T> = Rx<T, Self>;
}

impl<C: Capacity, const UNBOUNDED_BACKOFF: bool, SP: SyncPrimitives> BoundedChannel
    for Array<C, UNBOUNDED_BACKOFF, SP>
{
}

fn slot_mask(capacity: usize) -> usize {
    (capacity.next_power_of_two() << 1).wrapping_sub(1)
}

pub(crate) struct Slot<T> {
    msg: UnsafeCell<MaybeUninit<T>>,
    stamp: AtomicUsize,
}

pub(crate) struct Storage<T, C: Capacity> {
    slots: Slots<Slot<T>, C>,
    slot_mask: C::Mask,
}

impl<T, C: Capacity> Storage<T, C> {
    fn capacity(&self) -> usize {
        self.slots.capacity()
    }

    fn slot_mask(&self) -> usize {
        C::get_mask(self.slot_mask, slot_mask)
    }

    fn closed_flag(&self) -> usize {
        (self.slot_mask() >> 1) + 1
    }
}

impl<C: Capacity, const UNBOUNDED_BACKOFF: bool, SP: SyncPrimitives> internal::Channel
    for Array<C, UNBOUNDED_BACKOFF, SP>
{
    type Storage<T> = Storage<T, C>;

    fn storage<T>(self) -> Self::Storage<T> {
        let lap = slot_mask(self.capacity.get()).wrapping_add(1);
        Storage {
            slots: Slots::new(self.capacity, |i| Slot {
                msg: UnsafeCell::new(MaybeUninit::uninit()),
                stamp: AtomicUsize::new(i.wrapping_sub(lap) & LB),
            }),
            slot_mask: self.capacity.mask(slot_mask),
        }
    }

    fn capacity<T>(storage: &Self::Storage<T>) -> Option<usize> {
        Some(storage.slots.len())
    }

    fn drop_storage<T>(chan: &mut Chan<T, Self>) {
        debug_assert!(chan.tx_shared_state.load_mut() & chan.closed_flag() != 0);
        let tail = chan.tx_shared_state.load_mut() & LB & !chan.closed_flag();
        let tail_idx = tail & chan.slot_mask();
        let head = chan.rx_shared_state.load_mut();
        let head_idx = head & chan.slot_mask();
        let (r1, r2) = match tail_idx.cmp(&head_idx) {
            cmp::Ordering::Less => (0..tail_idx, head_idx..chan.capacity()),
            cmp::Ordering::Equal if head != tail => (0..chan.capacity(), 0..0),
            cmp::Ordering::Equal => (0..0, 0..0),
            cmp::Ordering::Greater => (head_idx..tail_idx, 0..0),
        };
        for idx in iter::chain(r1, r2) {
            let slot = unsafe { chan.slots.get_unchecked(idx) };
            unsafe { slot.msg.with_ref_mut(|m| m.assume_init_drop()) };
        }
    }

    fn close<T>(chan: &Chan<T, Self>) {
        chan.tx_shared_state.fetch_or(chan.closed_flag(), SeqCst);
    }

    fn is_closed<T>(chan: &Chan<T, Self>) -> bool {
        chan.tx_waiter.is_closed()
    }

    type TxAtomicState<T> = AtomicUsize;
    type TxState<T> = usize;
    type TxSlot<T> = (NonNull<Slot<T>>, usize);
    type TxWaiter = WaitQueue<SP>;
    type TxRefCount = AtomicUsize;

    fn tx_init_state<T>(storage: &Self::Storage<T>) -> Self::TxAtomicState<T> {
        let tail = 0;
        let max_tail = storage.slot_mask().wrapping_add(1);
        AtomicUsize::new(tail | (max_tail << HB_SHIFT))
    }

    fn is_full<T>(chan: &Chan<T, Self>) -> bool {
        let state = chan.tx_shared_state.load(Relaxed);
        let tail = state & LB;
        let max_tail = state >> HB_SHIFT;
        tail == max_tail
    }

    fn tx_acquire_slot<T>(chan: &Chan<T, Self>) -> Result<Self::TxSlot<T>, Self::TxState<T>> {
        let state = chan.tx_shared_state.load(Relaxed);
        let tail = state & LB;
        let max_tail = state >> HB_SHIFT;
        let tail_idx = state & chan.slot_mask();
        if tail == max_tail || tail_idx >= chan.capacity() - 1 {
            return Err(state);
        }
        chan.tx_shared_state
            .compare_exchange_weak(state, state + 1, SeqCst, Relaxed)?;
        let slot = unsafe { chan.slots.get_unchecked(tail_idx) }.into();
        Ok((slot, tail))
    }

    fn tx_acquire_slot_cold<T>(
        chan: &Chan<T, Self>,
        state: &mut Self::TxState<T>,
        first_call: bool,
    ) -> Result<Self::TxSlot<T>, TryAcquireError> {
        let slot_mask = chan.slot_mask();
        let lap = || slot_mask.wrapping_add(1);
        let mut backoff = (!first_call).then(Backoff::new);
        loop {
            let tail_idx = *state & slot_mask;
            let mut next_state = match tail_idx.cmp(&(chan.capacity() - 1)) {
                cmp::Ordering::Less => *state + 1,
                cmp::Ordering::Equal => (*state & !slot_mask).wrapping_add(lap()),
                cmp::Ordering::Greater => return Err(TryAcquireError::Closed),
            };
            let tail = *state & LB;
            let max_tail = *state >> HB_SHIFT;
            if max_tail == tail {
                let cur_state = chan.tx_shared_state.load(SeqCst);
                if cur_state != *state {
                    *state = cur_state;
                    continue;
                }
                let head = chan.rx_shared_state.load(SeqCst);
                let max_tail = head.wrapping_add(lap());
                if max_tail == tail {
                    return Err(TryAcquireError::Unavailable);
                }
                next_state = (next_state & LB) | (max_tail << HB_SHIFT);
            }
            if let Some(backoff) = backoff.as_ref() {
                backoff.spin();
            } else {
                backoff = Some(Backoff::new());
            }
            match chan
                .tx_shared_state
                .compare_exchange_weak(*state, next_state, SeqCst, Relaxed)
            {
                Ok(_) => {
                    let slot = unsafe { chan.slots.get_unchecked(tail_idx) }.into();
                    return Ok((slot, tail));
                }
                Err(s) => *state = s,
            }
        }
    }

    fn write_slot<T>(
        _chan: &Chan<T, Self>,
        (slot, tail): Self::TxSlot<T>,
        msg: T,
    ) -> Result<(), SendError<T>> {
        unsafe { slot.as_ref().msg.with_ref_mut(|m| m.write(msg)) };
        let order = if UNBOUNDED_BACKOFF { Release } else { SeqCst };
        unsafe { slot.as_ref().stamp.store(tail, order) };
        Ok(())
    }

    type RxAtomicState<T> = AtomicUsize;
    type RxState<T> = (NonNull<Slot<T>>, usize);
    type RxSlot<T> = (NonNull<Slot<T>>, usize);
    type RxWaiter = SpmcWaker<false>;
    type RxRefCount = ();

    fn rx_init_state<T>(_storage: &Self::Storage<T>) -> Self::RxAtomicState<T> {
        AtomicUsize::new(0)
    }

    fn is_empty<T>(chan: &Chan<T, Self>) -> bool {
        let head = chan.rx_shared_state.load(Relaxed);
        let head_idx = head & chan.slot_mask();
        let slot = unsafe { chan.slots.get_unchecked(head_idx) };
        slot.stamp.load(Acquire) != head
    }

    fn rx_acquire_slot<T>(chan: &Chan<T, Self>) -> Result<Self::RxSlot<T>, Self::RxState<T>> {
        let head = chan.rx_shared_state.load(Relaxed);
        let head_idx = head & chan.slot_mask();
        let slot = unsafe { chan.slots.get_unchecked(head_idx) };
        if slot.stamp.load(Acquire) != head {
            return Err((slot.into(), head));
        }
        Ok((slot.into(), head))
    }

    fn rx_acquire_slot_cold<T>(
        chan: &Chan<T, Self>,
        &mut (slot, head): &mut Self::RxState<T>,
        first_call: bool,
    ) -> Result<Self::RxSlot<T>, TryAcquireError> {
        if !first_call {
            let ordering = if UNBOUNDED_BACKOFF { Acquire } else { SeqCst };
            if unsafe { slot.as_ref() }.stamp.load(ordering) == head {
                return Ok((slot, head));
            }
        }
        if UNBOUNDED_BACKOFF {
            let tail = chan.tx_shared_state.load(SeqCst) & LB;
            let closed_flag = chan.closed_flag();
            if head == tail & !closed_flag {
                return Err(if head == tail {
                    TryAcquireError::Unavailable
                } else {
                    TryAcquireError::Closed
                });
            }
            let backoff = Backoff::new();
            while unsafe { slot.as_ref() }.stamp.load(Acquire) != head {
                backoff.snooze();
            }
            return Ok((slot, head));
        }
        if chan.tx_waiter.is_closed() {
            let tail = chan.tx_shared_state.load(SeqCst) & LB;
            debug_assert!(tail & chan.closed_flag() != 0);
            if head == tail & !chan.closed_flag() {
                return Err(TryAcquireError::Closed);
            }
        }
        Err(TryAcquireError::Unavailable)
    }

    fn read_slot<T>(chan: &Chan<T, Self>, (slot, head): Self::RxSlot<T>) -> T {
        let msg = unsafe { slot.as_ref().msg.with_ref(|m| m.assume_init_read()) };
        let slot_mask = chan.slot_mask();
        let new_head = if head & slot_mask == chan.capacity() - 1 {
            let lap = slot_mask.wrapping_add(1);
            (head & !slot_mask).wrapping_add(lap)
        } else {
            head + 1
        };
        chan.rx_shared_state.store(new_head, SeqCst);
        msg
    }
}
