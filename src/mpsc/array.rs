use core::{
    cmp,
    marker::PhantomData,
    mem::MaybeUninit,
    ptr::NonNull,
    sync::atomic::Ordering::{Acquire, Relaxed, Release, SeqCst},
};

use aiq::WaitQueue;
use spmc_waker::SpmcWaker;

use crate::{
    Channel, DEFAULT_UNBOUNDED_BACKOFF, MTx, Rx,
    array::{HB_SHIFT, LB, Slots},
    backoff::{Backoff, BackoffStrategy},
    capacity::Capacity,
    channel::{BoundedChannel, Chan},
    errors::{SendError, TryAcquireError},
    internal,
    loom::{AtomicUsizeExt, UnsafeCellExt, cell::UnsafeCell, sync::atomic::AtomicUsize},
    sync::{DefaultSyncPrimitives, SyncPrimitives},
};

/// Bounded MPSC channel implementation.
///
/// It allocates an array of `capacity` message slots. If `BLOCK_SIZE > 1` the array is fragmented
/// into blocks, which are released by the receiver only after their last slot has been read.
/// It means the exact capacity of the channel at one instant has a lower bound to
/// `capacity - BLOCK_SIZE`. As a consequence, every `recv` operation except the last in a
/// block uses relaxed synchronization, which is otherwise expensive on `x86_64` architecture.
pub struct Array<
    const BLOCK_SIZE: usize = 1,
    C: Capacity = usize,
    const UNBOUNDED_BACKOFF: bool = DEFAULT_UNBOUNDED_BACKOFF,
    SP: SyncPrimitives = DefaultSyncPrimitives,
> {
    capacity: C,
    sync: PhantomData<SP>,
}

impl<const BLOCK_SIZE: usize, C: Capacity, const UNBOUNDED_BACKOFF: bool, SP: SyncPrimitives>
    Array<BLOCK_SIZE, C, UNBOUNDED_BACKOFF, SP>
{
    /// Constructs a new `Array` with the specified capacity.
    pub fn new(capacity: C) -> Self {
        const {
            assert!(
                BLOCK_SIZE.is_power_of_two(),
                "`BLOCK_SIZE` must be a power of 2"
            );
        }
        assert!(
            capacity.get().is_multiple_of(BLOCK_SIZE),
            "capacity must be a multiple of `BLOCK_SIZE`"
        );
        Self {
            capacity,
            sync: PhantomData,
        }
    }
}

impl<const BLOCK_SIZE: usize, C: Capacity, const UNBOUNDED_BACKOFF: bool, SP: SyncPrimitives>
    Channel for Array<BLOCK_SIZE, C, UNBOUNDED_BACKOFF, SP>
{
    type TxHalf<T> = MTx<T, Self>;
    type RxHalf<T> = Rx<T, Self>;
}

impl<const BLOCK_SIZE: usize, C: Capacity, const UNBOUNDED_BACKOFF: bool, SP: SyncPrimitives>
    BoundedChannel for Array<BLOCK_SIZE, C, UNBOUNDED_BACKOFF, SP>
{
}

pub(crate) struct Slot<T> {
    msg: UnsafeCell<MaybeUninit<T>>,
    stamp: AtomicUsize,
}

impl<const BLOCK_SIZE: usize, C: Capacity, const UNBOUNDED_BACKOFF: bool, SP: SyncPrimitives>
    internal::Channel for Array<BLOCK_SIZE, C, UNBOUNDED_BACKOFF, SP>
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
        let max_tail = head.wrapping_add(chan.lap()) & LB & !(BLOCK_SIZE - 1);
        tail == max_tail
    }

    #[inline(always)]
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

    fn tx_acquire_slot_cold<T, B: BackoffStrategy>(
        chan: &Chan<T, Self>,
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
                let state_reload = chan.tx_state.load(SeqCst);
                if state_reload != *state {
                    *state = state_reload;
                    continue;
                }
                let head = chan.rx_state.load(SeqCst);
                let max_tail = head.wrapping_add(chan.lap()) & LB & !(BLOCK_SIZE - 1);
                if max_tail == tail {
                    return Err(TryAcquireError::Unavailable);
                }
                next_state = (next_state & LB) | (max_tail << HB_SHIFT);
            }
            if backoff.backoff(state, || chan.tx_state.load(Relaxed)) {
                continue;
            }
            match (chan.tx_state).compare_exchange_weak(*state, next_state, SeqCst, SeqCst) {
                Ok(_) => {
                    let slot = unsafe { chan.get_unchecked(tail_idx) }.into();
                    return Ok((slot, tail));
                }
                Err(s) => *state = s,
            }
        }
    }

    #[inline(always)]
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
    type RxWaiter = SpmcWaker;
    type RxRefCount = ();
    const WAKE_TX_AFTER_READ: bool = false;

    fn rx_init_state<T>(_storage: &Self::Storage<T>) -> Self::RxAtomicState<T> {
        AtomicUsize::new(0)
    }

    fn is_empty<T>(chan: &Chan<T, Self>) -> bool {
        let head = chan.rx_state.load(Relaxed);
        let head_idx = head & chan.slot_mask();
        let slot = unsafe { chan.get_unchecked(head_idx) };
        slot.stamp.load(Relaxed) != head
    }

    #[inline(always)]
    fn rx_acquire_slot<T>(chan: &Chan<T, Self>) -> Result<Self::RxSlot<T>, Self::RxState<T>> {
        let head = chan.rx_state.load(Relaxed);
        let head_idx = head & chan.slot_mask();
        let slot = unsafe { chan.get_unchecked(head_idx) };
        if slot.stamp.load(Acquire) != head {
            return Err((slot.into(), head));
        }
        Ok((slot.into(), head))
    }

    fn rx_acquire_slot_cold<T, B: BackoffStrategy>(
        chan: &Chan<T, Self>,
        &mut (slot, head): &mut Self::RxState<T>,
        backoff: bool,
    ) -> Result<Self::RxSlot<T>, TryAcquireError> {
        if !backoff {
            let ordering = if UNBOUNDED_BACKOFF { Acquire } else { SeqCst };
            if unsafe { slot.as_ref() }.stamp.load(ordering) == head {
                return Ok((slot, head));
            }
        }
        if UNBOUNDED_BACKOFF {
            let tail = chan.tx_state.load(SeqCst) & LB;
            let closed_flag = chan.closed_flag();
            if head == tail & !closed_flag {
                return Err(if head == tail {
                    TryAcquireError::Unavailable
                } else {
                    TryAcquireError::Closed
                });
            }
            #[cfg(not(loom))]
            let backoff = crossbeam_utils::Backoff::new();
            while unsafe { slot.as_ref() }.stamp.load(Acquire) != head {
                #[cfg(not(loom))]
                backoff.snooze();
                #[cfg(loom)]
                loom::hint::spin_loop();
            }
            return Ok((slot, head));
        }
        if chan.tx_waiter.is_closed() {
            let tail = chan.tx_state.load(SeqCst) & LB;
            debug_assert!(tail & chan.closed_flag() != 0);
            if head == tail & !chan.closed_flag() {
                return Err(TryAcquireError::Closed);
            }
        }
        Err(TryAcquireError::Unavailable)
    }

    #[inline(always)]
    fn read_slot<T>(chan: &Chan<T, Self>, (slot, head): Self::RxSlot<T>) -> T {
        let msg = unsafe { slot.as_ref().msg.with_ref(|m| m.assume_init_read()) };
        let new_head = chan.wrap_around(head & chan.slot_mask(), head, false);
        if new_head.is_multiple_of(BLOCK_SIZE) {
            chan.rx_state.store(new_head, SeqCst);
            chan.tx_waiter.notify_many_const::<BLOCK_SIZE>();
        } else {
            chan.rx_state.store(new_head, Relaxed);
        }
        msg
    }
}
