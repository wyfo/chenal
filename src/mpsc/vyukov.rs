use core::{
    cmp,
    mem::MaybeUninit,
    ptr::NonNull,
    sync::atomic::Ordering::{Acquire, Relaxed, SeqCst},
};

use aiq::WaitQueue;
use spmc_waker::SpmcWaker;

use crate::{
    array::Slots, backoff::{Backoff, BackoffStrategy}, capacity::Capacity, channel::{BoundedChannel, Chan},
    errors::{SendError, TryAcquireError},
    internal,
    loom::{cell::UnsafeCell, sync::atomic::AtomicUsize, AtomicUsizeExt, UnsafeCellExt},
    Channel,
    MTx,
    Rx,
    DEFAULT_UNBOUNDED_BACKOFF,
};

/// Bounded MPSC channel implementation.
///
/// It allocates an array of `capacity` message slots. If `BLOCK_SIZE > 1` the array is fragmented
/// into blocks, which are released by the receiver only after their last slot has been read.
/// It means the exact capacity of the channel at one instant has a lower bound to
/// `capacity - BLOCK_SIZE`. As a consequence, every `recv` operation except the last in a
/// block uses relaxed synchronization, which is otherwise expensive on `x86_64` architecture.
pub struct VyukovMpsc<
    C: Capacity = usize,
    const UNBOUNDED_BACKOFF: bool = DEFAULT_UNBOUNDED_BACKOFF,
> {
    capacity: C,
}

impl<C: Capacity, const UNBOUNDED_BACKOFF: bool> VyukovMpsc<C, UNBOUNDED_BACKOFF> {
    /// Constructs a new `Array` with the specified capacity.
    pub fn new(capacity: C) -> Self {
        Self { capacity }
    }
}

impl<C: Capacity, const UNBOUNDED_BACKOFF: bool> Channel for VyukovMpsc<C, UNBOUNDED_BACKOFF> {
    type TxHalf<T> = MTx<T, Self>;
    type RxHalf<T> = Rx<T, Self>;
}

impl<C: Capacity, const UNBOUNDED_BACKOFF: bool> BoundedChannel
    for VyukovMpsc<C, UNBOUNDED_BACKOFF>
{
}

pub(crate) struct Slot<T> {
    msg: UnsafeCell<MaybeUninit<T>>,
    stamp: AtomicUsize,
}

impl<C: Capacity, const UNBOUNDED_BACKOFF: bool> internal::Channel
    for VyukovMpsc<C, UNBOUNDED_BACKOFF>
{
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
        chan.tx_state
            .compare_exchange_weak(tail, tail + 1, Relaxed, Relaxed)?;
        Ok((slot.into(), tail))
    }

    fn tx_acquire_slot_cold<T, B: BackoffStrategy>(
        chan: &Chan<T, Self>,
        tail: &mut Self::TxState<T>,
        backoff: bool,
    ) -> Result<Self::TxSlot<T>, TryAcquireError> {
        let mut backoff = Backoff::<B>::new(backoff);
        loop {
            let tail_idx = *tail & chan.slot_mask();
            let next_tail = match tail_idx.cmp(&(chan.capacity() - 1)) {
                cmp::Ordering::Less => *tail + 1,
                // `chan.wrap_around` is not inlined properly compared to new_lap
                cmp::Ordering::Equal => chan.new_lap(*tail, false),
                cmp::Ordering::Greater => return Err(TryAcquireError::Closed),
            };
            let slot = unsafe { chan.get_unchecked(tail_idx) };
            let delta = slot.stamp.load(SeqCst).wrapping_sub(*tail) as isize;
            match delta.cmp(&0) {
                cmp::Ordering::Equal => {
                    if backoff.backoff(tail, || chan.tx_state.load(Relaxed)) {
                        continue;
                    }
                    match (chan.tx_state).compare_exchange_weak(*tail, next_tail, Relaxed, Relaxed)
                    {
                        Ok(_) => {
                            let slot = unsafe { chan.get_unchecked(tail_idx) }.into();
                            return Ok((slot, *tail));
                        }
                        Err(s) => *tail = s,
                    }
                }
                cmp::Ordering::Less => return Err(TryAcquireError::Unavailable),
                cmp::Ordering::Greater => *tail = chan.tx_state.load(Relaxed),
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
        unsafe { slot.as_ref().stamp.store(tail + 1, SeqCst) };
        chan.rx_waiter.wake_cold();
        Ok(())
    }

    type RxAtomicState<T> = AtomicUsize;
    type RxState<T> = (NonNull<Slot<T>>, usize);
    type RxSlot<T> = (NonNull<Slot<T>>, usize);
    type RxWaiter = SpmcWaker;
    type RxRefCount = ();

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
        if slot.stamp.load(Acquire) == head {
            return Err((slot.into(), head));
        }
        Ok((slot.into(), head))
    }

    fn rx_acquire_slot_cold<T, B: BackoffStrategy>(
        chan: &Chan<T, Self>,
        &mut (slot, head): &mut Self::RxState<T>,
        backoff: bool,
    ) -> Result<Self::RxSlot<T>, TryAcquireError> {
        if !backoff && unsafe { slot.as_ref() }.stamp.load(SeqCst) != head {
            return Ok((slot, head));
        }
        if chan.tx_waiter.is_closed() {
            let tail = chan.tx_state.load(Relaxed);
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
        let new_stamp = head.wrapping_add(chan.lap());
        unsafe { slot.as_ref().stamp.store(new_stamp, SeqCst) };
        let new_head = chan.wrap_around(head & chan.slot_mask(), head, false);
        chan.rx_state.store(new_head, Relaxed);
        chan.tx_waiter.notify_one();
        msg
    }
}
