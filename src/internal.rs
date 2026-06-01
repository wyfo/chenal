use alloc::sync::Arc;

use crate::{
    backoff::BackoffStrategy,
    capacity::ConstCapacity,
    channel::Chan,
    errors::{SendError, TryAcquireError},
    rc::RefCount,
    sync::SyncPrimitives,
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
    fn drop_storage<T, SP: SyncPrimitives>(chan: &mut Chan<T, Self, SP>);
    fn close<T, SP: SyncPrimitives>(chan: &Chan<T, Self, SP>);
    fn is_closed<T, SP: SyncPrimitives>(chan: &Chan<T, Self, SP>) -> bool;
    type TxAtomicState<T>;
    type TxState<T>;
    type TxSlot<T>;
    type TxWaiter<SP: SyncPrimitives>: Waiter<SP>;
    type TxRefCount: RefCount;
    fn tx_init_state<T>(storage: &Self::Storage<T>) -> Self::TxAtomicState<T>;
    fn is_full<T, SP: SyncPrimitives>(chan: &Chan<T, Self, SP>) -> bool;
    fn tx_acquire_slot<T, SP: SyncPrimitives>(
        chan: &Chan<T, Self, SP>,
    ) -> Result<Self::TxSlot<T>, Self::TxState<T>>;
    fn tx_acquire_slot_cold<T, B: BackoffStrategy, SP: SyncPrimitives>(
        chan: &Chan<T, Self, SP>,
        state: &mut Self::TxState<T>,
        backoff: bool,
    ) -> Result<Self::TxSlot<T>, TryAcquireError>;
    fn write_slot<T, SP: SyncPrimitives>(
        chan: &Chan<T, Self, SP>,
        slot: Self::TxSlot<T>,
        msg: T,
    ) -> Result<(), SendError<T>>;
    type RxAtomicState<T>;
    type RxState<T>;
    type RxSlot<T>;
    type RxWaiter<SP: SyncPrimitives>: Waiter<SP>;
    type RxRefCount: RefCount;
    fn rx_init_state<T>(storage: &Self::Storage<T>) -> Self::RxAtomicState<T>;
    fn is_empty<T, SP: SyncPrimitives>(chan: &Chan<T, Self, SP>) -> bool;
    fn rx_acquire_slot<T, SP: SyncPrimitives>(
        chan: &Chan<T, Self, SP>,
    ) -> Result<Self::RxSlot<T>, Self::RxState<T>>;
    fn rx_acquire_slot_cold<T, B: BackoffStrategy, SP: SyncPrimitives>(
        chan: &Chan<T, Self, SP>,
        state: &mut Self::RxState<T>,
        backoff: bool,
    ) -> Result<Self::RxSlot<T>, TryAcquireError>;
    fn read_slot<T, SP: SyncPrimitives>(chan: &Chan<T, Self, SP>, slot: Self::RxSlot<T>) -> T;
}

pub(crate) trait ChannelHalf<T, Ch: Channel, SP: SyncPrimitives> {
    const HALF: crate::channel::Half;
    fn new(chan: Arc<Chan<T, Ch, SP>>) -> Self;
    fn chan(&self) -> &Arc<Chan<T, Ch, SP>>;
    fn raw_clone(&self) -> Self
    where
        Self: Clone,
    {
        Self::new(self.chan().clone())
    }
}
