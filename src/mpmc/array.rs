use core::{
    cmp,
    mem::MaybeUninit,
    ptr::NonNull,
    sync::atomic::Ordering::{Acquire, Relaxed, Release, SeqCst},
};

use aiq::WaitQueue;

use crate::{
    Channel, DEFAULT_UNBOUNDED_BACKOFF, MRx, MTx,
    array::Slots,
    backoff::{Backoff, BackoffStrategy},
    capacity::Capacity,
    channel::{BoundedChannel, Chan},
    errors::{SendError, TryAcquireError},
    internal,
    loom::{AtomicUsizeExt, UnsafeCellExt, cell::UnsafeCell, sync::atomic::AtomicUsize},
};

/// Bounded MPMC channel implementation.
///
/// It allocates an array of `capacity` message slots.
pub struct Array<C: Capacity = usize, const UNBOUNDED_BACKOFF: bool = DEFAULT_UNBOUNDED_BACKOFF> {
    capacity: C,
}

impl<C: Capacity, const UNBOUNDED_BACKOFF: bool> Array<C, UNBOUNDED_BACKOFF> {
    /// Constructs a new `Array` with the specified capacity.
    pub fn new(capacity: C) -> Self {
        Self { capacity }
    }
}

impl<C: Capacity, const UNBOUNDED_BACKOFF: bool> Channel for Array<C, UNBOUNDED_BACKOFF> {
    type TxHalf<T> = MTx<T, Self>;
    type RxHalf<T> = MRx<T, Self>;
}

impl<C: Capacity, const UNBOUNDED_BACKOFF: bool> BoundedChannel for Array<C, UNBOUNDED_BACKOFF> {}

pub(crate) struct Slot<T> {
    msg: UnsafeCell<MaybeUninit<T>>,
    stamp: AtomicUsize,
}

impl<C: Capacity, const UNBOUNDED_BACKOFF: bool> internal::Channel for Array<C, UNBOUNDED_BACKOFF> {
    type Storage<T> = Slots<Slot<T>, C>;

    fn storage<T>(self) -> Self::Storage<T> {
        Slots::new(self.capacity, |i, _| Slot {
            msg: UnsafeCell::new(MaybeUninit::uninit()),
            stamp: AtomicUsize::new(i),
        })
    }

    fn capacity<T>(storage: &Self::Storage<T>) -> Option<usize> {
        Some(storage.len())
    }

    fn drop_storage<T>(chan: &mut Chan<T, Self>) {
        debug_assert!(chan.tx_state.load_mut() & chan.closed_flag() != 0);
        let tail = chan.tx_state.load_mut() & !chan.closed_flag();
        let head = chan.rx_state.load_mut();
        for slot in chan.slots_between(head, tail) {
            unsafe { slot.msg.with_ref_mut(|m| m.assume_init_drop()) };
        }
    }

    fn close<T>(chan: &Chan<T, Self>) {
        let ordering = if UNBOUNDED_BACKOFF { SeqCst } else { Relaxed };
        chan.tx_state.fetch_or(chan.closed_flag(), ordering);
    }

    fn is_closed<T>(chan: &Chan<T, Self>) -> bool {
        chan.tx_waiter.is_closed()
    }

    type TxAtomicState<T> = AtomicUsize;
    type TxState<T> = usize;
    type TxSlot<T> = (NonNull<Slot<T>>, usize);
    type TxWaiter = WaitQueue;
    type TxRefCount = AtomicUsize;

    fn tx_init_state<T>(_storage: &Self::Storage<T>) -> Self::TxAtomicState<T> {
        AtomicUsize::new(0)
    }

    fn is_full<T>(chan: &Chan<T, Self>) -> bool {
        let tail = chan.tx_state.load(Relaxed);
        let tail_idx = tail & chan.slot_mask();
        if tail_idx >= chan.capacity() {
            return false;
        }
        let slot = unsafe { chan.get_unchecked(tail_idx) };
        slot.stamp.load(Relaxed) != tail
    }

    #[inline(always)]
    fn tx_acquire_slot<T>(chan: &Chan<T, Self>) -> Result<Self::TxSlot<T>, Self::TxState<T>> {
        let tail = chan.tx_state.load(Relaxed);
        let tail_idx = tail & chan.slot_mask();
        if tail_idx >= chan.capacity() - 1 {
            return Err(tail);
        }
        let slot = unsafe { chan.get_unchecked(tail_idx) };
        let stamp = slot.stamp.load(Acquire);
        if stamp != tail {
            return Err(tail);
        }
        let ordering = if UNBOUNDED_BACKOFF { SeqCst } else { Relaxed };
        (chan.tx_state).compare_exchange_weak(tail, tail + 1, ordering, Relaxed)?;
        Ok((slot.into(), tail))
    }

    fn tx_acquire_slot_cold<T, B: BackoffStrategy>(
        chan: &Chan<T, Self>,
        tail: &mut Self::TxState<T>,
        backoff: bool,
    ) -> Result<Self::TxSlot<T>, TryAcquireError> {
        let mut backoff = Backoff::<B>::new(backoff);
        'outer: loop {
            let tail_idx = *tail & chan.slot_mask();
            let next_tail = match tail_idx.cmp(&(chan.capacity() - 1)) {
                cmp::Ordering::Less => *tail + 1,
                // `chan.wrap_around` is not inlined properly compared to new_lap
                cmp::Ordering::Equal => chan.new_lap(*tail, false),
                cmp::Ordering::Greater => return Err(TryAcquireError::Closed),
            };
            let slot = unsafe { chan.get_unchecked(tail_idx) };
            let ordering = if UNBOUNDED_BACKOFF { Acquire } else { SeqCst };
            let delta = slot.stamp.load(ordering).wrapping_sub(*tail) as isize;
            match delta.cmp(&0) {
                cmp::Ordering::Equal => {
                    if backoff.backoff(tail, || chan.tx_state.load(Relaxed)) {
                        continue;
                    }
                }
                cmp::Ordering::Less if UNBOUNDED_BACKOFF => {
                    let tail_reload = chan.tx_state.load(Relaxed);
                    if tail_reload != *tail {
                        *tail = tail_reload;
                        continue;
                    }
                    let head = chan.rx_state.load(SeqCst);
                    let max_tail = head.wrapping_add(chan.lap());
                    if *tail == max_tail {
                        return Err(TryAcquireError::Unavailable);
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
                        // Tail can advance concurrently and slot.stamp can be
                        // one lap forward, so it's necessary to reload the tail
                        // to not loop forever
                        let tail_reload = chan.tx_state.load(SeqCst);
                        if tail_reload != *tail {
                            *tail = tail_reload;
                            continue 'outer;
                        } else if slot.stamp.load(Acquire) == *tail {
                            break;
                        }
                    }
                }
                cmp::Ordering::Less => {
                    let tail_reload = chan.tx_state.load(Relaxed);
                    if tail_reload != *tail {
                        *tail = tail_reload;
                        continue;
                    }
                    return Err(TryAcquireError::Unavailable);
                }
                cmp::Ordering::Greater => {
                    *tail = chan.tx_state.load(Relaxed);
                    continue;
                }
            }
            let ordering = if UNBOUNDED_BACKOFF { SeqCst } else { Relaxed };
            match (chan.tx_state).compare_exchange_weak(*tail, next_tail, ordering, Relaxed) {
                Ok(_) => return Ok((slot.into(), *tail)),
                Err(t) => *tail = t,
            }
        }
    }

    #[inline(always)]
    fn write_slot<T>(
        chan: &Chan<T, Self>,
        (slot, tail): Self::TxSlot<T>,
        msg: T,
    ) -> Result<(), SendError<T>> {
        unsafe { slot.as_ref().msg.with_ref_mut(|m| m.write(msg)) };
        let ordering = if UNBOUNDED_BACKOFF { Release } else { SeqCst };
        unsafe { slot.as_ref().stamp.store(tail + 1, ordering) };
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
            fn notify_receivers<C: Capacity, const UB: bool, T>(
                chan: &Chan<T, Array<C, UB>>,
                tail: usize,
            ) {
                // A lost notification means that WaitQueue state has been updated.
                // As `is_empty` synchronizes with `notify_one`/`notify_all`,
                // `tx_state` modification which occurs before the notification
                // is visible to a Relaxed load.
                let tail_reload = chan.tx_state.load(Relaxed);
                let next_tail = chan.wrap_around(tail & chan.slot_mask(), tail, false);
                if tail_reload == next_tail {
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
    type RxSlot<T> = (NonNull<Slot<T>>, usize);
    type RxWaiter = WaitQueue;
    type RxRefCount = AtomicUsize;

    fn rx_init_state<T>(_storage: &Self::Storage<T>) -> Self::RxAtomicState<T> {
        AtomicUsize::new(0)
    }

    fn is_empty<T>(chan: &Chan<T, Self>) -> bool {
        let head = chan.rx_state.load(Relaxed);
        let head_idx = head & chan.slot_mask();
        let slot = unsafe { chan.get_unchecked(head_idx) };
        slot.stamp.load(Relaxed) == head
    }

    #[inline(always)]
    fn rx_acquire_slot<T>(chan: &Chan<T, Self>) -> Result<Self::RxSlot<T>, Self::RxState<T>> {
        let head = chan.rx_state.load(Relaxed);
        let head_idx = head & chan.slot_mask();
        let slot = unsafe { chan.get_unchecked(head_idx) };
        if slot.stamp.load(Acquire) != head + 1 {
            return Err(head);
        }
        let new_head = chan.wrap_around(head_idx, head, false);
        let ordering = if UNBOUNDED_BACKOFF { SeqCst } else { Relaxed };
        (chan.rx_state).compare_exchange_weak(head, new_head, ordering, Relaxed)?;
        Ok((slot.into(), head))
    }

    fn rx_acquire_slot_cold<T, B: BackoffStrategy>(
        chan: &Chan<T, Self>,
        head: &mut Self::RxState<T>,
        backoff: bool,
    ) -> Result<Self::RxSlot<T>, TryAcquireError> {
        let mut backoff = Backoff::<B>::new(backoff);
        'outer: loop {
            let head_idx = *head & chan.slot_mask();
            let slot = unsafe { chan.get_unchecked(head_idx) };
            let ordering = if UNBOUNDED_BACKOFF { Acquire } else { SeqCst };
            if slot.stamp.load(ordering) != *head + 1 {
                let head_reload = chan.rx_state.load(Relaxed);
                if head_reload != *head {
                    *head = head_reload;
                    continue;
                }
                if UNBOUNDED_BACKOFF {
                    let tail = chan.tx_state.load(SeqCst);
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
                        } else if slot.stamp.load(Acquire) == *head + 1 {
                            break;
                        }
                    }
                } else {
                    if chan.tx_waiter.is_closed() {
                        let tail = chan.tx_state.load(Relaxed);
                        debug_assert!(tail & chan.closed_flag() != 0);
                        if *head == tail & !chan.closed_flag() {
                            return Err(TryAcquireError::Closed);
                        }
                    }
                    return Err(TryAcquireError::Unavailable);
                }
            }
            if backoff.backoff(head, || chan.rx_state.load(Relaxed)) {
                continue;
            }
            let new_head = chan.wrap_around(head_idx, *head, false);
            match (chan.rx_state).compare_exchange_weak(*head, new_head, SeqCst, Relaxed) {
                Ok(_) => return Ok((slot.into(), *head)),
                Err(h) => *head = h,
            }
        }
    }

    #[inline(always)]
    fn read_slot<T>(chan: &Chan<T, Self>, (slot, head): Self::RxSlot<T>) -> T {
        let msg = unsafe { slot.as_ref().msg.with_ref(|m| m.assume_init_read()) };
        let new_stamp = head.wrapping_add(chan.lap());
        let ordering = if UNBOUNDED_BACKOFF { Release } else { SeqCst };
        unsafe { slot.as_ref().stamp.store(new_stamp, ordering) };
        if UNBOUNDED_BACKOFF {
            chan.tx_waiter.notify_one();
        } else if !chan.tx_waiter.is_empty() {
            // A receiver reading to a later slot may have completed and notified
            // a sender before this slot has been read. In that case, the
            // notification might have been lost as the channel would still
            // appear full for the waken sender. That's why if `rx_state` has
            // already advanced, we wake all senders to ensure any lost
            // notification is recovered.
            #[cold]
            #[inline(never)]
            fn notify_senders<C: Capacity, const UB: bool, T>(
                chan: &Chan<T, Array<C, UB>>,
                head: usize,
            ) {
                // A lost notification means that WaitQueue state has been updated.
                // As `is_empty` synchronizes with `notify_one`/`notify_all`,
                // `rx_state` modification which occurs before the notification
                // is visible to a Relaxed load.
                let head_reload = chan.rx_state.load(Relaxed);
                let next_head = chan.wrap_around(head & chan.slot_mask(), head, false);
                if head_reload == next_head {
                    chan.tx_waiter.notify_one();
                } else {
                    chan.tx_waiter.notify_all();
                }
            }
            notify_senders(chan, head);
        }
        msg
    }
}
