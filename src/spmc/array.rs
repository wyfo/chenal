use core::{
    cell::UnsafeCell,
    marker::PhantomData,
    mem::MaybeUninit,
    ops::Deref,
    sync::atomic::{
        AtomicUsize,
        Ordering::{Acquire, Relaxed, SeqCst},
    },
};

use aiq::WaitQueue;
use crossbeam_utils::Backoff;
use spmc_waker::SpmcWaker;

use crate::{
    Channel, MRx, Tx,
    array::{HB_SHIFT, LB, Slots},
    capacity::Capacity,
    channel::{BoundedChannel, Chan},
    errors::{SendError, TryAcquireError},
    internal,
    loom::{AtomicUsizeExt, RacyUnsafeCellExt, UnsafeCellExt},
    sync::{DefaultSyncPrimitives, SyncPrimitives},
};

/// Bounded SPMC channel implementation.
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
pub struct Array<C: Capacity = usize, SP: SyncPrimitives = DefaultSyncPrimitives> {
    capacity: C,
    sync: PhantomData<SP>,
}

impl<C: Capacity, SP: SyncPrimitives> Array<C, SP> {
    /// Constructs a new `Array` with the specified capacity.
    pub fn new(capacity: C) -> Self {
        Self {
            capacity,
            sync: PhantomData,
        }
    }
}

impl<C: Capacity, SP: SyncPrimitives> Channel for Array<C, SP> {
    type TxHalf<T> = Tx<T, Self>;
    type RxHalf<T> = MRx<T, Self>;
}

impl<C: Capacity, SP: SyncPrimitives> BoundedChannel for Array<C, SP> {}

type Slot<T> = UnsafeCell<MaybeUninit<T>>;

pub(crate) struct Storage<T, C: Capacity> {
    slots: Slots<Slot<T>, C>,
    closed: AtomicUsize,
}

impl<T, C: Capacity> Deref for Storage<T, C> {
    type Target = Slots<Slot<T>, C>;

    fn deref(&self) -> &Self::Target {
        &self.slots
    }
}

impl<C: Capacity, SP: SyncPrimitives> internal::Channel for Array<C, SP> {
    type Storage<T> = Storage<T, C>;

    fn storage<T>(self) -> Self::Storage<T> {
        Storage {
            slots: Slots::new(self.capacity, |_, _| UnsafeCell::new_racy()),
            closed: AtomicUsize::new(0),
        }
    }

    fn capacity<T>(storage: &Self::Storage<T>) -> Option<usize> {
        Some(storage.len())
    }

    fn drop_storage<T>(chan: &mut Chan<T, Self>) {
        let tail = chan.tx_state.load_mut() & LB;
        let head = chan.rx_state.load_mut() & LB;
        for slot in chan.slots_between(head, tail) {
            unsafe { slot.with_ref_mut(|m| m.assume_init_drop()) };
        }
    }

    fn close<T>(chan: &Chan<T, Self>) {
        let _ = chan.closed.compare_exchange(0, 1, SeqCst, Relaxed);
    }

    fn is_closed<T>(chan: &Chan<T, Self>) -> bool {
        chan.closed.load(Relaxed) != 0
    }

    type TxAtomicState<T> = AtomicUsize;
    type TxState<T> = usize;
    type TxSlot<T> = usize;
    type TxWaiter = SpmcWaker<false>;
    type TxRefCount = ();

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
        if tail == max_tail || chan.closed.load(SeqCst) != 0 {
            return Err(state);
        }
        Ok(state)
    }

    fn tx_acquire_slot_cold<T>(
        chan: &Chan<T, Self>,
        state: &mut Self::TxState<T>,
        _first_call: bool,
    ) -> Result<Self::TxSlot<T>, TryAcquireError> {
        if chan.closed.load(SeqCst) != 0 {
            return Err(TryAcquireError::Closed);
        }
        let tail = *state & LB;
        let head = chan.rx_state.load(SeqCst);
        let max_tail = head.wrapping_add(chan.lap()) & LB;
        if max_tail == tail {
            return Err(TryAcquireError::Unavailable);
        }
        Ok(tail | max_tail << HB_SHIFT)
    }

    fn write_slot<T>(
        chan: &Chan<T, Self>,
        state: Self::TxSlot<T>,
        msg: T,
    ) -> Result<(), SendError<T>> {
        let tail_idx = state & chan.slot_mask();
        let slot = unsafe { chan.get_unchecked(tail_idx) };
        unsafe { slot.write_racy(msg) };
        let new_state = chan.wrap_around(tail_idx, state, true);
        chan.tx_state.store(new_state, SeqCst);
        if chan.closed.load(SeqCst) != 0 {
            #[cold]
            #[inline(never)]
            fn handle_closed<C: Capacity, SP: SyncPrimitives, T>(
                chan: &Chan<T, Array<C, SP>>,
                state: usize,
            ) -> Result<(), SendError<T>> {
                let new_tail = chan.wrap_around(state & chan.slot_mask(), state, true) & LB;
                if let Err(closed) =
                    (chan.closed).compare_exchange(1, 2 | (new_tail << 2), SeqCst, Relaxed)
                    && closed >> 2 != new_tail
                {
                    debug_assert!(closed >> 2 == state & LB);
                    let slot = unsafe { chan.get_unchecked(state & chan.slot_mask()) };
                    let msg = unsafe { slot.read_racy().assume_init() };
                    return Err(SendError(msg));
                }
                Ok(())
            }
            return handle_closed(chan, state);
        }
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
        let head = chan.rx_state.load(Relaxed) & LB;
        let tail = chan.tx_state.load(Relaxed) & LB;
        head == tail
    }

    fn rx_acquire_slot<T>(chan: &Chan<T, Self>) -> Result<Self::RxSlot<T>, Self::RxState<T>> {
        let state = chan.rx_state.load(Acquire);
        let head = state & LB;
        let tail = state >> HB_SHIFT;
        if head == tail {
            return Err(state);
        }
        let head_idx = state & chan.slot_mask();
        let slot = unsafe { chan.get_unchecked(head_idx) };
        let msg = unsafe { slot.read_racy() };
        let new_state = chan.wrap_around(head_idx, state, true);
        chan.rx_state
            .compare_exchange_weak(state, new_state, SeqCst, Acquire)
            .map(|_| unsafe { msg.assume_init() })
    }

    fn rx_acquire_slot_cold<T>(
        chan: &Chan<T, Self>,
        state: &mut Self::RxState<T>,
        first_call: bool,
    ) -> Result<Self::RxSlot<T>, TryAcquireError> {
        let backoff = Backoff::new();
        let mut spin = first_call;
        loop {
            let head = *state & LB;
            let tail = *state >> HB_SHIFT;
            let head_idx = *state & chan.slot_mask();
            let mut new_state = chan.wrap_around(head_idx, *state, true);
            if head == tail {
                let state_reload = chan.rx_state.load(SeqCst);
                if state_reload != *state {
                    *state = state_reload;
                    continue;
                }
                let closed = chan.closed.load(SeqCst);
                let mut tail = chan.tx_state.load(SeqCst) & LB;
                if closed != 0
                    && let Err(closed) =
                        (chan.closed).compare_exchange(1, 2 | (tail << 2), SeqCst, SeqCst)
                {
                    tail = closed >> 2;
                }
                if head == tail {
                    return Err(if closed != 0 {
                        TryAcquireError::Closed
                    } else {
                        TryAcquireError::Unavailable
                    });
                }
                new_state = (new_state & LB) | (tail << HB_SHIFT);
            }
            let slot = unsafe { chan.get_unchecked(head_idx) };
            let msg = unsafe { slot.read_racy() };
            if spin {
                backoff.spin();
            }
            match (chan.rx_state).compare_exchange_weak(*state, new_state, SeqCst, SeqCst) {
                Ok(_) => return Ok(unsafe { msg.assume_init() }),
                Err(s) => *state = s,
            }
            spin = true;
        }
    }

    fn read_slot<T>(_chan: &Chan<T, Self>, msg: Self::RxSlot<T>) -> T {
        msg
    }
}
