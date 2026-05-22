use core::{
    cell::UnsafeCell,
    cmp,
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

use crate::{
    Channel, DEFAULT_UNBOUNDED_BACKOFF, MRx, MTx,
    array::{HB_SHIFT, LB, Slots},
    capacity::Capacity,
    channel::{BoundedChannel, Chan},
    errors::{SendError, TryAcquireError},
    internal,
    loom::{AtomicUsizeExt, UnsafeCellExt},
    sync::{DefaultSyncPrimitives, SyncPrimitives},
};

/// Bounded channel implementation.
///
/// It allocates an array of `capacity` message slots.
///
/// Channel's algorithm uses [`atomic_memcpy`], which is not compatible with strict provenance.
/// The progress on a provenance-compatible alternative are tracked in
/// [RFC 3301](https://github.com/rust-lang/rfcs/pull/3301)
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
    /// Constructs a new `Array` with the specified capacity.
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
    type RxHalf<T> = MRx<T, Self>;
}

impl<C: Capacity, const UNBOUNDED_BACKOFF: bool, SP: SyncPrimitives> BoundedChannel
    for Array<C, UNBOUNDED_BACKOFF, SP>
{
}

pub(crate) struct Slot<T> {
    msg: UnsafeCell<MaybeUninit<T>>,
    stamp: AtomicUsize,
}

impl<T> Slot<T> {
    #[inline(always)]
    unsafe fn write(&self, msg: T) {
        unsafe { atomic_memcpy::atomic_store(self.msg.get().cast(), msg, Relaxed) }
    }

    #[inline(always)]
    unsafe fn read(&self) -> MaybeUninit<T> {
        unsafe { atomic_memcpy::atomic_load(self.msg.get().cast(), Relaxed) }
    }
}

impl<C: Capacity, const UNBOUNDED_BACKOFF: bool, SP: SyncPrimitives> internal::Channel
    for Array<C, UNBOUNDED_BACKOFF, SP>
{
    type Storage<T> = Slots<Slot<T>, C>;

    fn storage<T>(self) -> Self::Storage<T> {
        Slots::new(self.capacity, |i, lap| Slot {
            msg: UnsafeCell::new(MaybeUninit::uninit()),
            stamp: AtomicUsize::new(i.wrapping_sub(lap) & LB),
        })
    }

    fn capacity<T>(storage: &Self::Storage<T>) -> Option<usize> {
        Some(storage.len())
    }

    fn drop_storage<T>(chan: &mut Chan<T, Self>) {
        debug_assert!(chan.tx_state.load_mut() & chan.closed_flag() != 0);
        let tail = chan.tx_state.load_mut() & LB & !chan.closed_flag();
        let head = chan.rx_state.load_mut();
        for slot in chan.slots_between(head, tail) {
            unsafe { slot.msg.with_ref_mut(|m| m.assume_init_drop()) };
        }
    }

    fn close<T>(chan: &Chan<T, Self>) {
        chan.tx_state.fetch_or(chan.closed_flag(), SeqCst);
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
        let max_tail = storage.lap();
        AtomicUsize::new(tail | (max_tail << HB_SHIFT))
    }

    fn is_full<T>(chan: &Chan<T, Self>) -> bool {
        let tail = chan.tx_state.load(Relaxed) & LB;
        let head = chan.rx_state.load(Relaxed) & LB;
        let max_tail = head.wrapping_add(chan.lap()) & LB;
        tail == max_tail
    }

    fn tx_acquire_slot<T>(chan: &Chan<T, Self>) -> Result<Self::TxSlot<T>, Self::TxState<T>> {
        let state = chan.tx_state.load(Relaxed);
        let tail = state & LB;
        let max_tail = state >> HB_SHIFT;
        let tail_idx = state & chan.slot_mask();
        if tail == max_tail || tail_idx >= chan.capacity() - 1 {
            return Err(state);
        }
        chan.tx_state
            .compare_exchange_weak(state, state + 1, SeqCst, Relaxed)?;
        let slot = unsafe { chan.get_unchecked(tail_idx) }.into();
        Ok((slot, tail))
    }

    fn tx_acquire_slot_cold<T>(
        chan: &Chan<T, Self>,
        state: &mut Self::TxState<T>,
        first_call: bool,
    ) -> Result<Self::TxSlot<T>, TryAcquireError> {
        let mut backoff = (!first_call).then(Backoff::new);
        loop {
            let tail_idx = *state & chan.slot_mask();
            let mut next_state = match tail_idx.cmp(&(chan.capacity() - 1)) {
                cmp::Ordering::Less => *state + 1,
                cmp::Ordering::Equal => chan.new_lap(*state, true),
                cmp::Ordering::Greater => return Err(TryAcquireError::Closed),
            };
            let tail = *state & LB;
            let max_tail = *state >> HB_SHIFT;
            if max_tail == tail {
                let state_reload = chan.tx_state.load(SeqCst);
                if state_reload != *state {
                    *state = state_reload;
                    continue;
                }
                let head = chan.rx_state.load(SeqCst);
                let max_tail = head.wrapping_add(chan.lap()) & LB;
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
                .tx_state
                .compare_exchange_weak(*state, next_state, SeqCst, Relaxed)
            {
                Ok(_) => {
                    let slot = unsafe { chan.get_unchecked(tail_idx) }.into();
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
        unsafe { slot.as_ref().write(msg) };
        let order = if UNBOUNDED_BACKOFF { Release } else { SeqCst };
        unsafe { slot.as_ref().stamp.store(tail, order) };
        Ok(())
    }

    type RxAtomicState<T> = AtomicUsize;
    type RxState<T> = usize;
    type RxSlot<T> = T;
    type RxWaiter = WaitQueue<SP>;
    type RxRefCount = AtomicUsize;

    fn rx_init_state<T>(_storage: &Self::Storage<T>) -> Self::RxAtomicState<T> {
        AtomicUsize::new(0)
    }

    fn is_empty<T>(chan: &Chan<T, Self>) -> bool {
        let head = chan.rx_state.load(Relaxed);
        let head_idx = head & chan.slot_mask();
        let slot = unsafe { chan.get_unchecked(head_idx) };
        slot.stamp.load(Relaxed) != head
    }

    fn rx_acquire_slot<T>(chan: &Chan<T, Self>) -> Result<Self::RxSlot<T>, Self::RxState<T>> {
        let head = chan.rx_state.load(Relaxed);
        let head_idx = head & chan.slot_mask();
        let slot = unsafe { chan.get_unchecked(head_idx) };
        if slot.stamp.load(Acquire) != head {
            return Err(head);
        }
        let msg = unsafe { slot.read() };
        let new_head = if head_idx == chan.capacity() - 1 {
            chan.new_lap(head, false)
        } else {
            head + 1
        };
        chan.rx_state
            .compare_exchange_weak(head, new_head, SeqCst, Relaxed)
            .map(|_| unsafe { msg.assume_init() })
    }

    fn rx_acquire_slot_cold<T>(
        chan: &Chan<T, Self>,
        head: &mut Self::RxState<T>,
        first_call: bool,
    ) -> Result<Self::RxSlot<T>, TryAcquireError> {
        let mut backoff = (!first_call).then(Backoff::new);
        loop {
            let head_idx = *head & chan.slot_mask();
            let slot = unsafe { chan.get_unchecked(head_idx) };
            let ordering = if UNBOUNDED_BACKOFF { Acquire } else { SeqCst };
            if slot.stamp.load(ordering) != *head {
                let head_reload = chan.rx_state.load(SeqCst);
                if head_reload != *head {
                    *head = head_reload;
                    continue;
                }
                if UNBOUNDED_BACKOFF {
                    let tail = chan.tx_state.load(SeqCst) & LB;
                    let closed_flag = chan.closed_flag();
                    if *head == tail & !closed_flag {
                        return Err(if *head == tail {
                            TryAcquireError::Unavailable
                        } else {
                            TryAcquireError::Closed
                        });
                    }
                    let backoff = Backoff::new();
                    while slot.stamp.load(Acquire) != *head {
                        backoff.snooze();
                    }
                } else {
                    if chan.tx_waiter.is_closed() {
                        let tail = chan.tx_state.load(SeqCst) & LB;
                        debug_assert!(tail & chan.closed_flag() != 0);
                        if *head == tail & !chan.closed_flag() {
                            return Err(TryAcquireError::Closed);
                        }
                    }
                    return Err(TryAcquireError::Unavailable);
                }
            }
            let msg = unsafe { slot.read() };
            let new_head = if head_idx == chan.capacity() - 1 {
                chan.new_lap(*head, false)
            } else {
                *head + 1
            };
            if let Some(backoff) = backoff.as_ref() {
                backoff.spin();
            } else {
                backoff = Some(Backoff::new());
            }
            match chan
                .rx_state
                .compare_exchange_weak(*head, new_head, SeqCst, Relaxed)
            {
                Ok(_) => return Ok(unsafe { msg.assume_init() }),
                Err(h) => *head = h,
            }
        }
    }

    fn read_slot<T>(_chan: &Chan<T, Self>, msg: Self::RxSlot<T>) -> T {
        msg
    }
}
