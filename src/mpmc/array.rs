use core::{
    cmp,
    ptr::NonNull,
    sync::atomic::{
        Ordering::{AcqRel, Acquire, Relaxed, Release, SeqCst},
        fence,
    },
};

use aiq::WaitQueue;

use crate::{
    Channel, DEFAULT_UNBOUNDED_BACKOFF, MRx, MTx,
    array::{HB_SHIFT, LB, Slots},
    backoff::{Backoff, BackoffStrategy, NoBackoff},
    capacity::Capacity,
    channel::{BoundedChannel, Chan},
    errors::{SendError, TryAcquireError},
    internal,
    loom::{AtomicUsizeExt, RacyCell, sync::atomic::AtomicUsize},
    sync::SyncPrimitives,
};

/// Bounded MPMC channel implementation.
///
/// It allocates an array of `capacity` message slots.
///
/// # Soundness
///
/// Channel's algorithm relies on an **undefined behavior**, which is known to
/// [work in practice](https://github.com/rust-lang/unsafe-code-guidelines/blob/master/resources/deliberate-ub.md)
/// and is used in other widespread algorithms like SeqLocks.
///
/// Progress on a sound alternative is tracked in
/// [RFC 3301](https://github.com/rust-lang/rfcs/pull/3301)
pub struct Array<
    // Adding a BLOCK_SIZE parameter might be tempting to relax the SeqCst CAS on recv.
    // However, it would make stale SeqCst loads possible on send, invalidating the
    // synchronization.
    C: Capacity = usize,
    const UNBOUNDED_BACKOFF: bool = DEFAULT_UNBOUNDED_BACKOFF,
> {
    capacity: C,
}

impl<C: Capacity, const UNBOUNDED_BACKOFF: bool> Array<C, UNBOUNDED_BACKOFF> {
    /// Constructs a new `Array` with the specified capacity.
    pub fn new(capacity: C) -> Self {
        Self { capacity }
    }
}

impl<C: Capacity, const UNBOUNDED_BACKOFF: bool> Channel for Array<C, UNBOUNDED_BACKOFF> {
    type TxHalf<T, SP: SyncPrimitives> = MTx<T, Self, NoBackoff, SP>;
    type RxHalf<T, SP: SyncPrimitives> = MRx<T, Self, NoBackoff, SP>;
}

impl<C: Capacity, const UNBOUNDED_BACKOFF: bool> BoundedChannel for Array<C, UNBOUNDED_BACKOFF> {}

pub(crate) struct Slot<T> {
    msg: RacyCell<T>,
    stamp: AtomicUsize,
}

impl<C: Capacity, const UNBOUNDED_BACKOFF: bool> internal::Channel for Array<C, UNBOUNDED_BACKOFF> {
    type Storage<T> = Slots<Slot<T>, C>;

    fn storage<T>(self) -> Self::Storage<T> {
        Slots::new(self.capacity, |i, lap| Slot {
            msg: RacyCell::new(),
            stamp: AtomicUsize::new(i.wrapping_sub(lap) & LB),
        })
    }

    fn capacity<T>(storage: &Self::Storage<T>) -> Option<usize> {
        Some(storage.len())
    }

    fn drop_storage<T, SP: SyncPrimitives>(chan: &mut Chan<T, Self, SP>) {
        debug_assert!(chan.tx_state.load_mut() & chan.closed_flag() != 0);
        let tail = chan.tx_state.load_mut() & LB & !chan.closed_flag();
        let head = chan.rx_state.load_mut();
        for slot in chan.slots_between(head, tail) {
            unsafe { slot.msg.read_racy().assume_init_drop() };
        }
    }

    fn close<T, SP: SyncPrimitives>(chan: &Chan<T, Self, SP>) {
        let ordering = if UNBOUNDED_BACKOFF { SeqCst } else { AcqRel };
        chan.tx_state.fetch_or(chan.closed_flag(), ordering);
    }

    fn is_closed<T, SP: SyncPrimitives>(chan: &Chan<T, Self, SP>) -> bool {
        chan.tx_waiter.is_closed()
    }

    type TxAtomicState<T> = AtomicUsize;
    type TxState<T> = usize;
    type TxSlot<T> = (NonNull<Slot<T>>, usize);
    type TxWaiter<SP: SyncPrimitives> = WaitQueue<SP>;
    type TxRefCount = AtomicUsize;

    fn tx_init_state<T>(storage: &Self::Storage<T>) -> Self::TxAtomicState<T> {
        let tail = 0;
        let max_tail = storage.lap();
        AtomicUsize::new(tail | (max_tail << HB_SHIFT))
    }

    fn is_full<T, SP: SyncPrimitives>(chan: &Chan<T, Self, SP>) -> bool {
        let tail = chan.tx_state.load(Relaxed) & LB;
        let head = chan.rx_state.load(Relaxed) & LB;
        let max_tail = head.wrapping_add(chan.lap()) & LB;
        tail == max_tail
    }

    #[inline(always)]
    fn tx_acquire_slot<T, SP: SyncPrimitives>(
        chan: &Chan<T, Self, SP>,
    ) -> Result<Self::TxSlot<T>, Self::TxState<T>> {
        let state = chan.tx_state.load(Relaxed);
        let tail = state & LB;
        let max_tail = state >> HB_SHIFT;
        let tail_idx = state & chan.slot_mask();
        if tail == max_tail || tail_idx >= chan.capacity() - 1 {
            return Err(state);
        }
        let ordering = if UNBOUNDED_BACKOFF { SeqCst } else { Relaxed };
        chan.tx_state
            .compare_exchange_weak(state, state + 1, ordering, Relaxed)?;
        let slot = unsafe { chan.get_unchecked(tail_idx) }.into();
        Ok((slot, tail))
    }

    fn tx_acquire_slot_cold<T, B: BackoffStrategy, SP: SyncPrimitives>(
        chan: &Chan<T, Self, SP>,
        state: &mut Self::TxState<T>,
        backoff: bool,
    ) -> Result<Self::TxSlot<T>, TryAcquireError> {
        let mut backoff = Backoff::<B>::new(backoff);
        loop {
            let tail_idx = *state & chan.slot_mask();
            let mut next_state = match tail_idx.cmp(&(chan.capacity() - 1)) {
                cmp::Ordering::Less => *state + 1,
                // `chan.wrap_around` is not inlined properly compared to new_lap
                cmp::Ordering::Equal => chan.new_lap(*state, true),
                cmp::Ordering::Greater => return Err(TryAcquireError::Closed),
            };
            let tail = *state & LB;
            let max_tail = *state >> HB_SHIFT;
            if max_tail == tail {
                let state_reload = chan.tx_state.load(Relaxed);
                if state_reload != *state {
                    *state = state_reload;
                    continue;
                }
                let head = chan.rx_state.load(SeqCst);
                let max_tail = head.wrapping_add(chan.lap()) & LB;
                if max_tail == tail {
                    return Err(TryAcquireError::Unavailable);
                }
                if !UNBOUNDED_BACKOFF {
                    fence(Release);
                }
                next_state = (next_state & LB) | (max_tail << HB_SHIFT);
            }
            if backoff.backoff(state, || chan.tx_state.load(Relaxed)) {
                continue;
            }
            let ordering = if UNBOUNDED_BACKOFF { SeqCst } else { Relaxed };
            match (chan.tx_state).compare_exchange_weak(*state, next_state, ordering, Relaxed) {
                Ok(_) => {
                    let slot = unsafe { chan.get_unchecked(tail_idx) }.into();
                    return Ok((slot, tail));
                }
                Err(s) => *state = s,
            }
        }
    }

    #[inline(always)]
    fn write_slot<T, SP: SyncPrimitives>(
        chan: &Chan<T, Self, SP>,
        (slot, tail): Self::TxSlot<T>,
        msg: T,
    ) -> Result<(), SendError<T>> {
        if !UNBOUNDED_BACKOFF {
            fence(Acquire);
        }
        unsafe { slot.as_ref().msg.write_racy(msg) };
        let order = if UNBOUNDED_BACKOFF { Release } else { SeqCst };
        unsafe { slot.as_ref().stamp.store(tail, order) };
        if UNBOUNDED_BACKOFF {
            chan.rx_waiter.notify_one();
        } else if !chan.rx_waiter.is_empty() {
            // A sender writing to a later slot may have completed and notified
            // a receiver before this slot has been written. In that case, the
            // notification might have been lost as the channel would still
            // appear empty for the waken receiver. That's why if `tx_state` has
            // already advanced, we wake all receivers to ensure any lost
            // notification is recovered.
            #[cold]
            #[inline(never)]
            fn notify_receivers<C, const UB: bool, SP, T>(
                chan: &Chan<T, Array<C, UB>, SP>,
                tail: usize,
            ) where
                C: Capacity,
                SP: SyncPrimitives,
            {
                // A lost notification means that WaitQueue state has been updated.
                // As `is_empty` synchronizes with `notify_one`/`notify_all`,
                // `tx_state` modification which occurs before the notification
                // is visible to a Relaxed load.
                let state_reload = chan.tx_state.load(Relaxed);
                let next_tail = chan.wrap_around(tail & chan.slot_mask(), tail, false);
                if state_reload & LB == next_tail {
                    chan.rx_waiter.notify_one();
                } else {
                    chan.rx_waiter.notify_all();
                }
            }
            notify_receivers(chan, tail);
        }
        Ok(())
    }

    type RxAtomicState<T> = AtomicUsize;
    type RxState<T> = usize;
    type RxSlot<T> = T;
    type RxWaiter<SP: SyncPrimitives> = WaitQueue<SP>;
    type RxRefCount = AtomicUsize;

    fn rx_init_state<T>(_storage: &Self::Storage<T>) -> Self::RxAtomicState<T> {
        AtomicUsize::new(0)
    }

    fn is_empty<T, SP: SyncPrimitives>(chan: &Chan<T, Self, SP>) -> bool {
        let head = chan.rx_state.load(Relaxed);
        let head_idx = head & chan.slot_mask();
        let slot = unsafe { chan.get_unchecked(head_idx) };
        slot.stamp.load(Relaxed) != head
    }

    #[inline(always)]
    fn rx_acquire_slot<T, SP: SyncPrimitives>(
        chan: &Chan<T, Self, SP>,
    ) -> Result<Self::RxSlot<T>, Self::RxState<T>> {
        let head = chan.rx_state.load(Relaxed);
        let head_idx = head & chan.slot_mask();
        let slot = unsafe { chan.get_unchecked(head_idx) };
        if slot.stamp.load(Acquire) != head {
            return Err(head);
        }
        let msg = unsafe { slot.msg.read_racy() };
        let new_head = chan.wrap_around(head_idx, head, false);
        chan.rx_state
            .compare_exchange_weak(head, new_head, SeqCst, Relaxed)
            .map(|_| unsafe { msg.assume_init() })
    }

    fn rx_acquire_slot_cold<T, B: BackoffStrategy, SP: SyncPrimitives>(
        chan: &Chan<T, Self, SP>,
        head: &mut Self::RxState<T>,
        backoff: bool,
    ) -> Result<Self::RxSlot<T>, TryAcquireError> {
        let mut backoff = Backoff::<B>::new(backoff);
        'outer: loop {
            let head_idx = *head & chan.slot_mask();
            let slot = unsafe { chan.get_unchecked(head_idx) };
            let ordering = if UNBOUNDED_BACKOFF { Acquire } else { SeqCst };
            if slot.stamp.load(ordering) != *head {
                let head_reload = chan.rx_state.load(Relaxed);
                if head_reload != *head {
                    *head = head_reload;
                    continue;
                }
                if UNBOUNDED_BACKOFF {
                    let tail = chan.tx_state.load(SeqCst) & LB;
                    if *head == tail & !chan.closed_flag() {
                        return Err(if *head == tail {
                            TryAcquireError::Unavailable
                        } else {
                            TryAcquireError::Closed
                        });
                    }
                    #[cfg(all(miri, not(feature = "std")))]
                    panic!("this miri test requires feature \"std\"");
                    #[cfg(any(all(miri, feature = "std"), loom))]
                    let _guard = chan.lock.lock().unwrap();
                    #[cfg(not(loom))]
                    let backoff = crossbeam_utils::Backoff::new();
                    loop {
                        #[cfg(not(loom))]
                        backoff.snooze();
                        #[cfg(loom)]
                        loom::hint::spin_loop();
                        // Head can advance concurrently and slot.stamp can be
                        // one lap forward, so it's necessary to reload the head
                        // to not loop forever
                        let head_reload = chan.rx_state.load(SeqCst);
                        if head_reload != *head {
                            *head = head_reload;
                            continue 'outer;
                        } else if slot.stamp.load(Acquire) == *head {
                            break;
                        }
                    }
                } else {
                    if chan.tx_waiter.is_closed() {
                        let tail = chan.tx_state.load(Relaxed) & LB;
                        debug_assert!(tail & chan.closed_flag() != 0);
                        if *head == tail & !chan.closed_flag() {
                            return Err(TryAcquireError::Closed);
                        }
                    }
                    return Err(TryAcquireError::Unavailable);
                }
            }
            let msg = unsafe { slot.msg.read_racy() };
            let new_head = chan.wrap_around(head_idx, *head, false);
            if backoff.backoff(head, || chan.rx_state.load(Relaxed)) {
                continue;
            }
            match (chan.rx_state).compare_exchange_weak(*head, new_head, SeqCst, Relaxed) {
                Ok(_) => return Ok(unsafe { msg.assume_init() }),
                Err(h) => *head = h,
            }
        }
    }

    #[inline(always)]
    fn read_slot<T, SP: SyncPrimitives>(chan: &Chan<T, Self, SP>, msg: Self::RxSlot<T>) -> T {
        chan.tx_waiter.notify_one();
        msg
    }
}
