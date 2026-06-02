use crate::{
    Channel, DEFAULT_UNBOUNDED_BACKOFF, MRx, Tx,
    backoff::BackoffStrategy,
    capacity::Capacity,
    channel::{BoundedChannel, Chan},
    errors::{SendError, TryAcquireError},
    internal,
};

/// Bounded SPMC channel implementation.
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

type Inner<C, const UNBOUNDED_BACKOFF: bool> = crate::mpmc::Array<C, UNBOUNDED_BACKOFF>;

/// Reinterprets a `Chan` over the SPMC channel as a `Chan` over the inner MPMC channel.
///
/// Both channels have identical `internal::Channel` associated types, so `Chan<T, Array<..>>`
/// and `Chan<T, Inner<..>>` have the same layout.
#[inline(always)]
fn as_mpmc<T, C: Capacity, const UNBOUNDED_BACKOFF: bool>(
    chan: &Chan<T, Array<C, UNBOUNDED_BACKOFF>>,
) -> &Chan<T, Inner<C, UNBOUNDED_BACKOFF>> {
    unsafe {
        &*(chan as *const Chan<T, Array<C, UNBOUNDED_BACKOFF>>
            as *const Chan<T, Inner<C, UNBOUNDED_BACKOFF>>)
    }
}

#[inline(always)]
fn as_mpmc_mut<T, C: Capacity, const UNBOUNDED_BACKOFF: bool>(
    chan: &mut Chan<T, Array<C, UNBOUNDED_BACKOFF>>,
) -> &mut Chan<T, Inner<C, UNBOUNDED_BACKOFF>> {
    unsafe {
        &mut *(chan as *mut Chan<T, Array<C, UNBOUNDED_BACKOFF>>
            as *mut Chan<T, Inner<C, UNBOUNDED_BACKOFF>>)
    }
}

impl<C: Capacity, const UNBOUNDED_BACKOFF: bool> Channel for Array<C, UNBOUNDED_BACKOFF> {
    type TxHalf<T> = Tx<T, Self>;
    type RxHalf<T> = MRx<T, Self>;
}

impl<C: Capacity, const UNBOUNDED_BACKOFF: bool> BoundedChannel for Array<C, UNBOUNDED_BACKOFF> {}

impl<C: Capacity, const UNBOUNDED_BACKOFF: bool> internal::Channel for Array<C, UNBOUNDED_BACKOFF> {
    type Storage<T> = <Inner<C, UNBOUNDED_BACKOFF> as internal::Channel>::Storage<T>;

    fn storage<T>(self) -> Self::Storage<T> {
        Inner::<C, UNBOUNDED_BACKOFF>::new(self.capacity).storage()
    }

    fn capacity<T>(storage: &Self::Storage<T>) -> Option<usize> {
        Inner::<C, UNBOUNDED_BACKOFF>::capacity(storage)
    }

    fn drop_storage<T>(chan: &mut Chan<T, Self>) {
        Inner::<C, UNBOUNDED_BACKOFF>::drop_storage(as_mpmc_mut(chan))
    }

    fn close<T>(chan: &Chan<T, Self>) {
        Inner::<C, UNBOUNDED_BACKOFF>::close(as_mpmc(chan))
    }

    fn is_closed<T>(chan: &Chan<T, Self>) -> bool {
        Inner::<C, UNBOUNDED_BACKOFF>::is_closed(as_mpmc(chan))
    }

    type TxAtomicState<T> = <Inner<C, UNBOUNDED_BACKOFF> as internal::Channel>::TxAtomicState<T>;
    type TxState<T> = <Inner<C, UNBOUNDED_BACKOFF> as internal::Channel>::TxState<T>;
    type TxSlot<T> = <Inner<C, UNBOUNDED_BACKOFF> as internal::Channel>::TxSlot<T>;
    type TxWaiter = <Inner<C, UNBOUNDED_BACKOFF> as internal::Channel>::TxWaiter;
    type TxRefCount = <Inner<C, UNBOUNDED_BACKOFF> as internal::Channel>::TxRefCount;

    fn tx_init_state<T>(storage: &Self::Storage<T>) -> Self::TxAtomicState<T> {
        Inner::<C, UNBOUNDED_BACKOFF>::tx_init_state(storage)
    }

    fn is_full<T>(chan: &Chan<T, Self>) -> bool {
        Inner::<C, UNBOUNDED_BACKOFF>::is_full(as_mpmc(chan))
    }

    #[inline(always)]
    fn tx_acquire_slot<T>(chan: &Chan<T, Self>) -> Result<Self::TxSlot<T>, Self::TxState<T>> {
        Inner::<C, UNBOUNDED_BACKOFF>::tx_acquire_slot(as_mpmc(chan))
    }

    fn tx_acquire_slot_cold<T, B: BackoffStrategy>(
        chan: &Chan<T, Self>,
        tail: &mut Self::TxState<T>,
        backoff: bool,
    ) -> Result<Self::TxSlot<T>, TryAcquireError> {
        Inner::<C, UNBOUNDED_BACKOFF>::tx_acquire_slot_cold::<T, B>(as_mpmc(chan), tail, backoff)
    }

    #[inline(always)]
    fn write_slot<T>(
        chan: &Chan<T, Self>,
        slot: Self::TxSlot<T>,
        msg: T,
    ) -> Result<(), SendError<T>> {
        Inner::<C, UNBOUNDED_BACKOFF>::write_slot(as_mpmc(chan), slot, msg)
    }

    type RxAtomicState<T> = <Inner<C, UNBOUNDED_BACKOFF> as internal::Channel>::RxAtomicState<T>;
    type RxState<T> = <Inner<C, UNBOUNDED_BACKOFF> as internal::Channel>::RxState<T>;
    type RxSlot<T> = <Inner<C, UNBOUNDED_BACKOFF> as internal::Channel>::RxSlot<T>;
    type RxWaiter = <Inner<C, UNBOUNDED_BACKOFF> as internal::Channel>::RxWaiter;
    type RxRefCount = <Inner<C, UNBOUNDED_BACKOFF> as internal::Channel>::RxRefCount;

    fn rx_init_state<T>(storage: &Self::Storage<T>) -> Self::RxAtomicState<T> {
        Inner::<C, UNBOUNDED_BACKOFF>::rx_init_state(storage)
    }

    fn is_empty<T>(chan: &Chan<T, Self>) -> bool {
        Inner::<C, UNBOUNDED_BACKOFF>::is_empty(as_mpmc(chan))
    }

    #[inline(always)]
    fn rx_acquire_slot<T>(chan: &Chan<T, Self>) -> Result<Self::RxSlot<T>, Self::RxState<T>> {
        Inner::<C, UNBOUNDED_BACKOFF>::rx_acquire_slot(as_mpmc(chan))
    }

    fn rx_acquire_slot_cold<T, B: BackoffStrategy>(
        chan: &Chan<T, Self>,
        state: &mut Self::RxState<T>,
        backoff: bool,
    ) -> Result<Self::RxSlot<T>, TryAcquireError> {
        Inner::<C, UNBOUNDED_BACKOFF>::rx_acquire_slot_cold::<T, B>(as_mpmc(chan), state, backoff)
    }

    #[inline(always)]
    fn read_slot<T>(chan: &Chan<T, Self>, slot: Self::RxSlot<T>) -> T {
        Inner::<C, UNBOUNDED_BACKOFF>::read_slot(as_mpmc(chan), slot)
    }
}
