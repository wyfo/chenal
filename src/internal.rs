use alloc::sync::Arc;

use crate::{
    capacity::ConstCapacity,
    channel::Chan,
    errors::{SendError, TryAcquireError},
    rc::RefCount,
    waiter::Waiter,
};

pub(crate) trait Capacity: Copy + 'static {
    type Mask: Copy;
    fn get(self) -> usize;
    fn mask(self, compute_mask: impl FnOnce(usize) -> usize) -> Self::Mask;
    fn get_mask(mask: Self::Mask, compute_mask: impl FnOnce(usize) -> usize) -> usize;
}

impl Capacity for usize {
    type Mask = usize;
    #[inline(always)]
    fn get(self) -> usize {
        self
    }
    #[inline(always)]
    fn mask(self, compute_mask: impl FnOnce(usize) -> usize) -> Self::Mask {
        compute_mask(self)
    }
    #[inline(always)]
    fn get_mask(mask: Self::Mask, _compute_mask: impl FnOnce(usize) -> usize) -> usize {
        mask
    }
}

impl<const N: usize> Capacity for ConstCapacity<N> {
    type Mask = ();
    #[inline(always)]
    fn get(self) -> usize {
        const { assert!(N > 0) };
        N
    }
    #[inline(always)]
    fn mask(self, _compute_mask: impl FnOnce(usize) -> usize) -> Self::Mask {}
    #[inline(always)]
    fn get_mask(_mask: Self::Mask, compute_mask: impl FnOnce(usize) -> usize) -> usize {
        compute_mask(N)
    }
}

pub(crate) trait Channel: Sized + 'static {
    type Storage<T>;
    fn storage<T>(self) -> Self::Storage<T>;
    fn capacity<T>(storage: &Self::Storage<T>) -> Option<usize>;
    fn drop_storage<T>(chan: &mut Chan<T, Self>);
    fn close<T>(chan: &Chan<T, Self>);
    fn is_closed<T>(chan: &Chan<T, Self>) -> bool;
    type TxAtomicState<T>;
    type TxState<T>;
    type TxSlot<T>;
    type TxWaiter: Waiter;
    type TxRefCount: RefCount;
    fn tx_init_state<T>(storage: &Self::Storage<T>) -> Self::TxAtomicState<T>;
    fn tx_acquire_slot<T>(chan: &Chan<T, Self>) -> Result<Self::TxSlot<T>, Self::TxState<T>>;
    fn tx_acquire_slot_cold<T>(
        chan: &Chan<T, Self>,
        state: &mut Self::TxState<T>,
        first_call: bool,
    ) -> Result<Self::TxSlot<T>, TryAcquireError>;
    fn write_slot<T>(
        chan: &Chan<T, Self>,
        slot: Self::TxSlot<T>,
        msg: T,
    ) -> Result<(), SendError<T>>;
    type RxAtomicState<T>;
    type RxState<T>;
    type RxSlot<T>;
    type RxWaiter: Waiter;
    type RxRefCount: RefCount;
    const WAKE_TX_AFTER_READ: bool = true;
    fn rx_init_state<T>(storage: &Self::Storage<T>) -> Self::RxAtomicState<T>;
    fn rx_acquire_slot<T>(chan: &Chan<T, Self>) -> Result<Self::RxSlot<T>, Self::RxState<T>>;
    fn rx_acquire_slot_cold<T>(
        chan: &Chan<T, Self>,
        state: &mut Self::RxState<T>,
        first_call: bool,
    ) -> Result<Self::RxSlot<T>, TryAcquireError>;
    fn read_slot<T>(chan: &Chan<T, Self>, slot: Self::RxSlot<T>) -> T;
}

pub(crate) trait ChannelHalf<T, Ch: Channel> {
    #[cfg(feature = "weak")]
    const HALF: crate::channel::Half;
    fn new(chan: Arc<Chan<T, Ch>>) -> Self;
    fn chan(&self) -> &Arc<Chan<T, Ch>>;
    fn raw_clone(&self) -> Self
    where
        Self: Clone,
    {
        Self::new(self.chan().clone())
    }
}
