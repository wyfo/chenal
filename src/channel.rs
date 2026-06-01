// loom Arc cannot be used in ClosedHandle
use alloc::sync::Arc;
use core::{
    fmt,
    marker::PhantomData,
    mem,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    panic::{RefUnwindSafe, UnwindSafe},
    pin::{Pin, pin},
    ptr::NonNull,
    sync::atomic::Ordering::{AcqRel, Acquire, Relaxed},
    task::{Context, Poll, ready},
};

use aiq::wait_queue::Wait;
use crossbeam_utils::CachePadded;

#[cfg(feature = "blocking")]
use crate::blocking::*;
use crate::{
    backoff::{BackoffStrategy, NoBackoff},
    errors::{AcquireError, RecvError, SendError, TryAcquireError, TryRecvError, TrySendError},
    internal,
    loom::sync::atomic::AtomicUsize,
    rc::RefCount,
    sync::{DefaultSyncPrimitives, SyncPrimitives},
    waiter::Waiter,
};

/// A channel implementation.
#[expect(private_bounds)]
pub trait Channel: internal::Channel {
    /// Sender half of the channel.
    type TxHalf<T, SP: SyncPrimitives>: ChannelHalf<Msg = T, Channel = Self, SyncPrimitives = SP>;
    /// Receiver half of the channel.
    type RxHalf<T, SP: SyncPrimitives>: ChannelHalf<Msg = T, Channel = Self, SyncPrimitives = SP>;
    /// Creates a sender/receiver pair using this implementation, with the given
    /// [`SyncPrimitives`].
    fn channel_with_sync<T, SP: SyncPrimitives>(
        self,
    ) -> (Self::TxHalf<T, SP>, Self::RxHalf<T, SP>) {
        use internal::ChannelHalf;
        let chan = Arc::new(Chan::<T, Self, SP>::new(self));
        (Self::TxHalf::new(chan.clone()), Self::RxHalf::new(chan))
    }
    /// Creates a sender/receiver pair using this implementation.
    fn channel<T>(
        self,
    ) -> (
        Self::TxHalf<T, DefaultSyncPrimitives>,
        Self::RxHalf<T, DefaultSyncPrimitives>,
    ) {
        self.channel_with_sync::<T, DefaultSyncPrimitives>()
    }
}

/// A bounded channel.
pub trait BoundedChannel: Channel {}

/// A sender or receiver half of a channel.
#[expect(private_bounds)]
pub trait ChannelHalf:
    internal::ChannelHalf<Self::Msg, Self::Channel, Self::SyncPrimitives>
{
    /// The message type of the channel.
    type Msg;
    /// The implementation of the channel.
    type Channel: Channel;
    /// The [`SyncPrimitives`] used by the channel.
    type SyncPrimitives: SyncPrimitives;
}

/// Opaque unique identifier for a channel instance. Two halves sharing the same `ChannelId`
/// belong to the same channel.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ChannelId(NonNull<()>);

trait Operation<T, Ch: internal::Channel, SP: SyncPrimitives>: 'static {
    type State;
    type Slot;
    type Waiter: Waiter<SP>;
    fn acquire_slot(chan: &Chan<T, Ch, SP>) -> Result<Self::Slot, Self::State>;
    fn acquire_slot_cold<B: BackoffStrategy>(
        chan: &Chan<T, Ch, SP>,
        state: &mut Self::State,
        backoff: bool,
    ) -> Result<Self::Slot, TryAcquireError>;
    fn waiter(chan: &Chan<T, Ch, SP>) -> &Self::Waiter;
}

struct SendMsg;
impl<T, Ch: internal::Channel, SP: SyncPrimitives> Operation<T, Ch, SP> for SendMsg {
    type State = Ch::TxState<T>;
    type Slot = Ch::TxSlot<T>;
    type Waiter = Ch::TxWaiter<SP>;
    fn acquire_slot(chan: &Chan<T, Ch, SP>) -> Result<Self::Slot, Self::State> {
        Ch::tx_acquire_slot(chan)
    }
    fn acquire_slot_cold<B: BackoffStrategy>(
        chan: &Chan<T, Ch, SP>,
        state: &mut Self::State,
        backoff: bool,
    ) -> Result<Self::Slot, TryAcquireError> {
        Ch::tx_acquire_slot_cold::<T, B, SP>(chan, state, backoff)
    }
    fn waiter(chan: &Chan<T, Ch, SP>) -> &Self::Waiter {
        &chan.tx_waiter
    }
}

struct RecvMsg;
impl<T, Ch: internal::Channel, SP: SyncPrimitives> Operation<T, Ch, SP> for RecvMsg {
    type State = Ch::RxState<T>;
    type Slot = Ch::RxSlot<T>;
    type Waiter = Ch::RxWaiter<SP>;
    fn acquire_slot(chan: &Chan<T, Ch, SP>) -> Result<Self::Slot, Self::State> {
        Ch::rx_acquire_slot(chan)
    }
    fn acquire_slot_cold<B: BackoffStrategy>(
        chan: &Chan<T, Ch, SP>,
        state: &mut Self::State,
        backoff: bool,
    ) -> Result<Self::Slot, TryAcquireError> {
        Ch::rx_acquire_slot_cold::<T, B, SP>(chan, state, backoff)
    }
    fn waiter(chan: &Chan<T, Ch, SP>) -> &Self::Waiter {
        &chan.rx_waiter
    }
}

// fields are duplicated instead of using an intermediate struct
// to keep them packed on the same cache line
pub(crate) struct Chan<T, Ch: internal::Channel, SP: SyncPrimitives = DefaultSyncPrimitives> {
    pub(crate) tx_state: CachePadded<Ch::TxAtomicState<T>>,
    pub(crate) rx_state: CachePadded<Ch::RxAtomicState<T>>,
    pub(crate) storage: Ch::Storage<T>,
    pub(crate) tx_waiter: Ch::TxWaiter<SP>,
    pub(crate) rx_waiter: Ch::RxWaiter<SP>,
    tx_cnt: Ch::TxRefCount,
    rx_cnt: Ch::RxRefCount,
    closed_waiter: aiq::WaitQueue<SP>,
    // In miri/loom MPMC tests with unbounded backoff, several receiver
    // might be blocked in a backoff loop, with the scheduler jumping
    // from one to another without ever running the sender to unblock
    // them. Use a global lock to fix the issue.
    #[cfg(any(all(miri, feature = "std"), loom))]
    pub(crate) lock: crate::loom::sync::Mutex<()>,
}

unsafe impl<T: Send, Ch: internal::Channel, SP: SyncPrimitives> Send for Chan<T, Ch, SP> {}
unsafe impl<T: Send, Ch: internal::Channel, SP: SyncPrimitives> Sync for Chan<T, Ch, SP> {}
impl<T, Ch: internal::Channel, SP: SyncPrimitives> UnwindSafe for Chan<T, Ch, SP> {}
impl<T, Ch: internal::Channel, SP: SyncPrimitives> RefUnwindSafe for Chan<T, Ch, SP> {}

impl<T, Ch: internal::Channel, SP: SyncPrimitives> Chan<T, Ch, SP> {
    pub(crate) fn new(channel: Ch) -> Self {
        let storage = channel.storage();
        Self {
            tx_state: Ch::tx_init_state(&storage).into(),
            rx_state: Ch::rx_init_state(&storage).into(),
            storage,
            tx_waiter: Default::default(),
            rx_waiter: Default::default(),
            tx_cnt: RefCount::one(),
            rx_cnt: RefCount::one(),
            closed_waiter: Default::default(),
            #[cfg(any(all(miri, feature = "std"), loom))]
            lock: crate::loom::sync::Mutex::new(()),
        }
    }

    fn id(&self) -> ChannelId {
        ChannelId(NonNull::from(self).cast())
    }

    fn close(&self) {
        Ch::close(self);
        // tx_waiter MUST be closed before rx_waiter as tx_waiter
        // can be used to check if closed in recv
        self.tx_waiter.close();
        self.rx_waiter.close();
        self.closed_waiter.close();
    }

    #[inline(always)]
    fn write_slot<E, F: From<(E, T)> + From<SendError<T>>>(
        &self,
        slot: Result<Ch::TxSlot<T>, E>,
        msg: T,
    ) -> Result<(), F> {
        match slot {
            Ok(slot) => Ch::write_slot(self, slot, msg)?,
            Err(err) => return Err((err, msg).into()),
        }
        Ok(())
    }

    #[inline(always)]
    fn read_slot<E>(&self, slot: Ch::RxSlot<T>) -> Result<T, E> {
        Ok(Ch::read_slot(self, slot))
    }

    #[inline(always)]
    fn try_acquire_slot<O: Operation<T, Ch, SP>, B: BackoffStrategy>(
        &self,
    ) -> Result<O::Slot, TryAcquireError> {
        O::acquire_slot(self).or_else(|state| self.try_acquire_slot_cold::<O, B>(state))
    }

    #[cold]
    fn try_acquire_slot_cold<O: Operation<T, Ch, SP>, B: BackoffStrategy>(
        &self,
        mut state: O::State,
    ) -> Result<O::Slot, TryAcquireError> {
        O::acquire_slot_cold::<B>(self, &mut state, true)
    }

    #[inline(always)]
    fn try_send<B: BackoffStrategy>(&self, msg: T) -> Result<(), TrySendError<T>> {
        self.write_slot(self.try_acquire_slot::<SendMsg, B>(), msg)
    }

    #[inline(always)]
    fn send_unbounded<B: BackoffStrategy>(&self, msg: T) -> Result<(), SendError<T>> {
        match self.try_send::<B>(msg) {
            Ok(()) => Ok(()),
            Err(TrySendError::Closed(msg)) => Err(SendError(msg)),
            Err(TrySendError::Full(_)) => unsafe { core::hint::unreachable_unchecked() },
        }
    }

    #[inline(always)]
    fn try_recv<B: BackoffStrategy>(&self) -> Result<T, TryRecvError> {
        self.read_slot(self.try_acquire_slot::<RecvMsg, B>()?)
    }

    #[inline(always)]
    fn poll_acquire_slot<'a, O: Operation<T, Ch, SP>, B: BackoffStrategy>(
        &'a self,
        cx: &mut Context<'_>,
        wait: Pin<&mut <O::Waiter as Waiter<SP>>::Wait<'a>>,
    ) -> Poll<Result<O::Slot, AcquireError>> {
        match O::acquire_slot(self) {
            Ok(slot) => Poll::Ready(Ok(slot)),
            Err(state) => self.poll_acquire_slot_cold::<O, B>(cx, wait, state),
        }
    }

    #[cold]
    fn poll_acquire_slot_cold<'a, O: Operation<T, Ch, SP>, B: BackoffStrategy>(
        &'a self,
        cx: &mut Context<'_>,
        mut wait: Pin<&mut <O::Waiter as Waiter<SP>>::Wait<'a>>,
        mut state: O::State,
    ) -> Poll<Result<O::Slot, AcquireError>> {
        let mut backoff = true;
        let mut waker_registered = false;
        loop {
            match O::acquire_slot_cold::<B>(self, &mut state, backoff) {
                Ok(slot) => {
                    if waker_registered {
                        unsafe { O::waiter(self).unregister() };
                    }
                    return Poll::Ready(Ok(slot));
                }
                Err(TryAcquireError::Closed) => return Poll::Ready(Err(AcquireError)),
                Err(TryAcquireError::Unavailable) if waker_registered => return Poll::Pending,
                Err(TryAcquireError::Unavailable) => {}
            }
            waker_registered = unsafe { O::waiter(self).register(wait.as_mut(), cx.waker()) };
            backoff = false;
        }
    }

    #[cfg(feature = "blocking")]
    fn acquire_slot_blocking<O: Operation<T, Ch, SP>, B: BackoffStrategy>(
        &self,
        parker: impl Into<Parker>,
    ) -> Result<O::Slot, TryAcquireError> {
        O::acquire_slot(self)
            .or_else(|state| self.acquire_slot_blocking_cold::<O, B>(state, parker.into()))
    }

    #[cfg(feature = "blocking")]
    #[cold]
    fn acquire_slot_blocking_cold<O: Operation<T, Ch, SP>, B: BackoffStrategy>(
        &self,
        mut state: O::State,
        mut parker: Parker,
    ) -> Result<O::Slot, TryAcquireError> {
        let mut wait = pin!(<O::Waiter as Waiter<SP>>::Wait::default());
        let mut backoff = true;
        let mut waker_registered = false;
        loop {
            match O::acquire_slot_cold::<B>(self, &mut state, backoff) {
                Ok(slot) => {
                    if waker_registered {
                        unsafe { O::waiter(self).unregister() };
                    }
                    return Ok(slot);
                }
                Err(TryAcquireError::Closed) => return Err(TryAcquireError::Closed),
                Err(TryAcquireError::Unavailable) => {}
            }
            waker_registered = if waker_registered {
                parker.park()?;
                false
            } else {
                unsafe { O::waiter(self).register(wait.as_mut(), &PARK_WAKER) }
            };
            backoff = false;
        }
    }

    #[cfg(feature = "blocking")]
    #[inline(always)]
    fn send_blocking<B: BackoffStrategy>(&self, msg: T) -> Result<(), SendError<T>> {
        let slot = self.acquire_slot_blocking::<SendMsg, B>(());
        self.write_slot(slot, msg)
    }

    #[cfg(feature = "blocking")]
    #[inline(always)]
    fn send_deadline<B: BackoffStrategy>(
        &self,
        msg: T,
        deadline: Instant,
    ) -> Result<(), SendTimeoutError<T>> {
        let slot = self.acquire_slot_blocking::<SendMsg, B>(deadline);
        self.write_slot(slot, msg)
    }

    #[cfg(feature = "blocking")]
    #[inline(always)]
    fn send_timeout<B: BackoffStrategy>(
        &self,
        msg: T,
        timeout: Duration,
    ) -> Result<(), SendTimeoutError<T>> {
        let slot = self.acquire_slot_blocking::<SendMsg, B>(timeout);
        self.write_slot(slot, msg)
    }

    #[cfg(feature = "blocking")]
    #[inline(always)]
    fn recv_blocking<B: BackoffStrategy>(&self) -> Result<T, RecvError> {
        let slot = self.acquire_slot_blocking::<RecvMsg, B>(())?;
        self.read_slot(slot)
    }

    #[cfg(feature = "blocking")]
    #[inline(always)]
    fn recv_deadline<B: BackoffStrategy>(&self, deadline: Instant) -> Result<T, RecvTimeoutError> {
        let slot = self.acquire_slot_blocking::<RecvMsg, B>(deadline)?;
        self.read_slot(slot)
    }

    #[cfg(feature = "blocking")]
    #[inline(always)]
    fn recv_timeout<B: BackoffStrategy>(&self, timeout: Duration) -> Result<T, RecvTimeoutError> {
        let slot = self.acquire_slot_blocking::<RecvMsg, B>(timeout)?;
        self.read_slot(slot)
    }

    fn ref_cnt(&self, half: Half) -> Option<&AtomicUsize> {
        match half {
            Half::Tx | Half::MTx | Half::UTx | Half::UMTx => self.tx_cnt.atomic(),
            Half::Rx | Half::MRx => self.rx_cnt.atomic(),
        }
    }

    fn drop_half(&self, half: Half) {
        if (self.ref_cnt(half)).is_none_or(|rc| rc.fetch_sub(1, AcqRel) == 1) {
            self.close();
        }
    }

    fn clone_half(&self, half: Half) {
        self.ref_cnt(half).unwrap().fetch_add(1, Relaxed);
    }
}

impl<T, Ch: internal::Channel, SP: SyncPrimitives> Drop for Chan<T, Ch, SP> {
    fn drop(&mut self) {
        Ch::drop_storage(self);
    }
}

impl<T, Ch: internal::Channel, SP: SyncPrimitives> Deref for Chan<T, Ch, SP> {
    type Target = Ch::Storage<T>;

    fn deref(&self) -> &Self::Target {
        &self.storage
    }
}

impl<T, Ch: internal::Channel, SP: SyncPrimitives> fmt::Debug for Chan<T, Ch, SP> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let half_kind = |rc_size| if rc_size > 0 { "M" } else { "S" };
        let tx_kind = half_kind(size_of_val(&self.tx_cnt));
        let rx_kind = half_kind(size_of_val(&self.rx_cnt));
        let mut builder = f.debug_struct("Chan");
        builder
            .field("id", &self.id())
            .field("kind", &alloc::format!("{tx_kind}P{rx_kind}C",));
        if let Some(capacity) = Ch::capacity(&self.storage) {
            builder.field("capacity", &capacity);
        }
        builder.finish_non_exhaustive()
    }
}

/// Future returned by [`Tx::send`]/[`MTx::send`].
///
/// Resolves once the message has been written in the channel, or the channel is closed.
///
/// Message being sent can be retrieved with [`cancel`](Self::cancel), cancelling the future.
pub struct SendFuture<
    'a,
    T,
    Ch: Channel,
    B: BackoffStrategy = NoBackoff,
    SP: SyncPrimitives = DefaultSyncPrimitives,
> {
    chan: &'a Chan<T, Ch, SP>,
    msg: Option<T>,
    wait: <Ch::TxWaiter<SP> as Waiter<SP>>::Wait<'a>,
    _backoff: PhantomData<B>,
}

impl<'a, T, Ch: Channel, B: BackoffStrategy, SP: SyncPrimitives> SendFuture<'a, T, Ch, B, SP> {
    #[inline(always)]
    fn new(chan: &'a Chan<T, Ch, SP>, msg: T) -> Self {
        Self {
            chan,
            msg: Some(msg),
            wait: Default::default(),
            _backoff: PhantomData,
        }
    }

    /// Cancels the send, returning the message if it has not yet been sent.
    pub fn cancel(self: Pin<&mut Self>) -> Option<T> {
        unsafe { self.get_unchecked_mut() }.msg.take()
    }
}

impl<T, Ch: Channel, B: BackoffStrategy, SP: SyncPrimitives> Future
    for SendFuture<'_, T, Ch, B, SP>
{
    type Output = Result<(), SendError<T>>;

    #[inline(always)]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let wait = unsafe { Pin::new_unchecked(&mut this.wait) };
        let slot = ready!(this.chan.poll_acquire_slot::<SendMsg, B>(cx, wait));
        let Some(msg) = this.msg.take() else {
            #[cold]
            #[inline(never)]
            fn polled_after_completion() -> ! {
                panic!("future polled after completion; channel will be blocked");
            }
            polled_after_completion();
        };
        Poll::Ready(this.chan.write_slot(slot, msg))
    }
}

/// Future returned by [`Rx::recv`]/[`MRx::recv`].
///
/// Resolves once a message is available, or the channel is closed.
///
/// The future can be reused to receive subsequent messages, as a [`Stream`](futures_core::stream::Stream).
pub struct RecvFuture<
    'a,
    T,
    Ch: Channel,
    B: BackoffStrategy = NoBackoff,
    SP: SyncPrimitives = DefaultSyncPrimitives,
> {
    chan: &'a Chan<T, Ch, SP>,
    wait: <Ch::RxWaiter<SP> as Waiter<SP>>::Wait<'a>,
    _backoff: PhantomData<B>,
}

impl<'a, T, Ch: Channel, B: BackoffStrategy, SP: SyncPrimitives> RecvFuture<'a, T, Ch, B, SP> {
    #[inline(always)]
    fn new(chan: &'a Chan<T, Ch, SP>) -> Self {
        Self {
            chan,
            wait: Default::default(),
            _backoff: PhantomData,
        }
    }
}

impl<T, Ch: Channel, B: BackoffStrategy, SP: SyncPrimitives> Future
    for RecvFuture<'_, T, Ch, B, SP>
{
    type Output = Result<T, RecvError>;

    #[inline(always)]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let wait = unsafe { Pin::new_unchecked(&mut this.wait) };
        let slot = ready!(this.chan.poll_acquire_slot::<RecvMsg, B>(cx, wait))?;
        Poll::Ready(this.chan.read_slot(slot))
    }
}

#[cfg(feature = "stream")]
impl<T, Ch: Channel, B: BackoffStrategy, SP: SyncPrimitives> futures_core::Stream
    for RecvFuture<'_, T, Ch, B, SP>
{
    type Item = T;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.poll(cx).map(Result::ok)
    }
}

/// Future returned by [`CloseHandle::closed`] and similar methods.
///
/// Resolves once the channel is closed.
pub struct ClosedFuture<'a, SP: SyncPrimitives = DefaultSyncPrimitives>(
    Wait<&'a aiq::WaitQueue<SP>, SP>,
);

impl<'a, SP: SyncPrimitives> ClosedFuture<'a, SP> {
    fn new<T, Ch: internal::Channel>(chan: &'a Chan<T, Ch, SP>) -> Self {
        Self(chan.closed_waiter.wait())
    }
}

impl<SP: SyncPrimitives> Future for ClosedFuture<'_, SP> {
    type Output = ();

    #[inline(always)]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        unsafe { self.map_unchecked_mut(|this| &mut this.0) }.poll(cx)
    }
}

trait Close: Send + Sync + fmt::Debug {
    fn id(&self) -> ChannelId;
    fn close(&self);
    fn closed(&self) -> ClosedFuture<'_>;
}

impl<T: Send, Ch: internal::Channel> Close for Chan<T, Ch, DefaultSyncPrimitives> {
    fn id(&self) -> ChannelId {
        self.id()
    }
    fn close(&self) {
        self.close();
    }
    fn closed(&self) -> ClosedFuture<'_> {
        ClosedFuture::new(self)
    }
}

/// A cloneable handle for closing a channel or observing its closure, without holding a sender
/// or receiver half.
#[derive(Debug, Clone)]
pub struct CloseHandle<'a>(Arc<dyn Close + 'a>);

impl<'a> CloseHandle<'a> {
    pub(crate) fn new<T: Send + 'a, Ch: Channel>(
        chan: Arc<Chan<T, Ch, DefaultSyncPrimitives>>,
    ) -> Self {
        Self(chan as _)
    }

    /// Returns the channel's unique identifier.
    pub fn channel_id(&self) -> ChannelId {
        self.0.id()
    }

    /// Closes the channel, preventing new messages to be sent.
    pub fn close(&self) {
        self.0.close();
    }

    /// Converts this handle into a [`CloseGuard`] that closes the channel when dropped.
    pub fn close_on_drop(self) -> CloseGuard<'a> {
        CloseGuard(self.0)
    }

    /// Returns a future that resolves once the channel is closed.
    pub fn closed(&self) -> ClosedFuture<'_> {
        self.0.closed()
    }
}

/// A RAII guard that closes the channel when dropped. Obtained via [`CloseHandle::close_on_drop`].
#[derive(Debug, Clone)]
pub struct CloseGuard<'a>(Arc<dyn Close + 'a>);

impl<'a> CloseGuard<'a> {
    /// Returns the channel's unique identifier.
    pub fn channel_id(&self) -> ChannelId {
        self.0.id()
    }

    /// Disarms the guard without closing the channel.
    pub fn forget(self) {
        mem::forget(self);
    }
}

impl Drop for CloseGuard<'_> {
    fn drop(&mut self) {
        self.0.close();
    }
}

#[derive(Copy, Clone)]
pub(crate) enum Half {
    Tx,
    MTx,
    UTx,
    UMTx,
    Rx,
    MRx,
}

macro_rules! channel_half {
    ($ty:ident $(<$B:ident>)?) => {
        impl<T, Ch: Channel, $($B: BackoffStrategy,)? SP: SyncPrimitives> Drop for $ty<T, Ch, $($B,)? SP> {
            fn drop(&mut self) {
                self.chan.drop_half(Half::$ty);
            }
        }
        impl<T, Ch: Channel, $($B: BackoffStrategy,)? SP: SyncPrimitives> internal::ChannelHalf<T, Ch, SP> for $ty<T, Ch, $($B,)? SP> {
            const HALF: Half = Half::$ty;
            fn new(chan: Arc<Chan<T, Ch, SP>>) -> Self {
                Self { chan, $(_backoff: PhantomData::<$B>)? }
            }
            fn chan(&self) -> &Arc<Chan<T, Ch, SP>> {
                &self.chan
            }
        }
        impl<T, Ch: Channel, $($B: BackoffStrategy,)? SP: SyncPrimitives> ChannelHalf for $ty<T, Ch, $($B,)? SP> {
            type Msg = T;
            type Channel = Ch;
            type SyncPrimitives = SP;
        }
        unsafe impl<T: Send, Ch: Channel, $($B: BackoffStrategy,)? SP: SyncPrimitives> Send for $ty<T, Ch, $($B,)? SP> {}
        unsafe impl<T: Send, Ch: Channel, $($B: BackoffStrategy,)? SP: SyncPrimitives> Sync for $ty<T, Ch, $($B,)? SP> {}
        impl<T, Ch: Channel, $($B: BackoffStrategy,)? SP: SyncPrimitives> UnwindSafe for $ty<T, Ch, $($B,)? SP> {}
        impl<T, Ch: Channel, $($B: BackoffStrategy,)? SP: SyncPrimitives> RefUnwindSafe for $ty<T, Ch, $($B,)? SP> {}
        impl<T, Ch: Channel, $($B: BackoffStrategy,)? SP: SyncPrimitives> $ty<T, Ch, $($B,)? SP> {
            /// Returns `true` if the channel is closed.
            pub fn is_closed(&self) -> bool {
                Ch::is_closed(&self.chan)
            }
            /// Closes the channel, preventing new messages to be sent.
            pub fn close(&self) {
                self.chan.close();
            }
            /// Returns a unique identifier for the underlying channel.
            pub fn channel_id(&self) -> ChannelId {
                self.chan.id()
            }
            /// Returns a future that resolves once the channel is closed.
            pub fn closed(&self) -> ClosedFuture<'_, SP> {
                ClosedFuture::new(&self.chan)
            }
            /// Returns the capacity of the bounded channel.
            pub fn capacity(&self) -> usize
            where
                Ch: BoundedChannel,
            {
                Ch::capacity(&self.chan).unwrap()
            }
            $(
            /// Updates the backoff strategy.
            pub fn with_backoff<B2: BackoffStrategy>(self) -> $ty<T, Ch, B2, SP> {
                let _ = PhantomData::<$B>;
                let this = ManuallyDrop::new(self);
                internal::ChannelHalf::new(unsafe {Arc::from_raw(Arc::as_ptr(&this.chan))})
            }
            )?
        }
        impl<T, Ch: Channel, $($B: BackoffStrategy)?> $ty<T, Ch, $($B,)? DefaultSyncPrimitives> {
            /// Returns a [`CloseHandle`] that can close the channel or observe its closure
            /// without holding this half.
            pub fn close_handle<'a>(&self) -> CloseHandle<'a>
            where
                T: Send + 'a,
            {
                CloseHandle::new(self.chan.clone())
            }
        }
        impl<T, Ch: Channel, $($B: BackoffStrategy,)? SP: SyncPrimitives> core::fmt::Debug for $ty<T, Ch, $($B,)? SP> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_tuple(stringify!($ty)).field(&self.chan).finish()
            }
        }
    };
}
macro_rules! tx_half {
    ($ty:ident $(<$B:ident>)? $(, $mut:tt)?) => {
        impl<T, Ch: Channel, $($B: BackoffStrategy,)? SP: SyncPrimitives> $ty<T, Ch, $($B,)? SP> {
            /// Sends a message, waiting asynchronously if the channel is full.
            #[inline]
            pub fn send(&$($mut)? self, msg: T) -> SendFuture<'_, T, Ch, backoff!($($B)?), SP> {
                SendFuture::new(&self.chan, msg)
            }
            /// Sends a message, blocking the current thread if the channel is full.
            #[cfg(feature = "blocking")]
            #[inline]
            pub fn send_blocking(&$($mut)? self, msg: T) -> Result<(), SendError<T>> {
                self.chan.send_blocking::<backoff!($($B)?)>(msg)
            }
            /// Sends a message, blocking until `deadline` if the channel is full.
            #[cfg(feature = "blocking")]
            #[inline]
            pub fn send_deadline(&$($mut)? self, msg: T, deadline: Instant) -> Result<(), SendTimeoutError<T>> {
                self.chan.send_deadline::<backoff!($($B)?)>(msg, deadline)
            }
            /// Sends a message, blocking for up to `timeout` if the channel is full.
            #[cfg(feature = "blocking")]
            #[inline]
            pub fn send_timeout(&$($mut)? self, msg: T, timeout: Duration) -> Result<(), SendTimeoutError<T>> {
                self.chan.send_timeout::<backoff!($($B)?)>(msg, timeout)
            }
            /// Attempts to send a message without waiting.
            #[inline]
            pub fn try_send(&$($mut)? self, msg: T) -> Result<(), TrySendError<T>> {
                self.chan.try_send::<backoff!($($B)?)>(msg)
            }
            /// Returns `true` if the channel is full.
            pub fn is_full(&self) -> bool {
                Ch::is_full(&self.chan)
            }
        }
    };
}
macro_rules! utx_methods {
    ($ty:ident $(<$B:ident>)? $(, $mut:tt)?) => {
        impl<T, Ch: Channel, $($B: BackoffStrategy,)? SP: SyncPrimitives> $ty<T, Ch, $($B,)? SP> {
            /// Sends a message. Never blocks or returns [`TrySendError::Full`] since the channel
            /// is unbounded.
            #[inline]
            pub fn send_unbounded(&$($mut)? self, msg: T) -> Result<(), SendError<T>> {
                self.chan.send_unbounded::<backoff!($($B)?)>(msg)
            }
        }
    };
}
macro_rules! rx_half {
    ($ty:ident $(<$B:ident>)? $(, $mut:tt)?) => {
        impl<T, Ch: Channel, $($B: BackoffStrategy,)? SP: SyncPrimitives> $ty<T, Ch, $($B,)? SP> {
            /// Receives a message, waiting asynchronously if the channel is empty.
            #[inline]
            pub fn recv(&$($mut)? self) -> RecvFuture<'_, T, Ch, backoff!($($B)?), SP> {
                RecvFuture::new(&self.chan)
            }
            /// Receives a message, blocking the current thread if the channel is empty.
            #[cfg(feature = "blocking")]
            #[inline]
            pub fn recv_blocking(&$($mut)? self) -> Result<T, RecvError> {
                self.chan.recv_blocking::<backoff!($($B)?)>()
            }
            /// Receives a message, blocking until `deadline` if the channel is empty.
            #[cfg(feature = "blocking")]
            #[inline]
            pub fn recv_deadline(&$($mut)? self, deadline: Instant) -> Result<T, RecvTimeoutError> {
                self.chan.recv_deadline::<backoff!($($B)?)>(deadline)
            }
            /// Receives a message, blocking for up to `timeout` if the channel is empty.
            #[cfg(feature = "blocking")]
            #[inline]
            pub fn recv_timeout(&$($mut)? self, timeout: Duration) -> Result<T, RecvTimeoutError> {
                self.chan.recv_timeout::<backoff!($($B)?)>(timeout)
            }
            /// Returns a blocking iterator that yields messages until the channel is closed.
            #[cfg(feature = "blocking")]
            pub fn iter_blocking(&$($mut)? self) -> impl Iterator<Item = T> {
                core::iter::from_fn(|| self.recv_blocking().ok())
            }
            /// Attempts to receive a message without waiting.
            #[inline]
            pub fn try_recv(&$($mut)? self) -> Result<T, TryRecvError> {
                self.chan.try_recv::<backoff!($($B)?)>()
            }
            /// Returns `true` if the channel is empty.
            pub fn is_empty(&self) -> bool {
                Ch::is_empty(&self.chan)
            }
        }
    };
}
macro_rules! cloneable_half {
    ($ty:ident $(<$B:ident>)?) => {
        impl<T, Ch: Channel, $($B: BackoffStrategy,)? SP: SyncPrimitives> Clone for $ty<T, Ch, $($B,)? SP> {
            fn clone(&self) -> Self {
                self.chan.clone_half(Half::$ty);
                internal::ChannelHalf::raw_clone(self)
            }
        }
        impl<T, Ch: Channel, $($B: BackoffStrategy,)? SP: SyncPrimitives> $ty<T, Ch, $($B,)? SP> {
            /// Returns a [`Weak`] reference that does not prevent the channel from being closed.
            pub fn downgrade(&self) -> Weak<Self> {
                Weak::new(self)
            }
        }
    };
}
macro_rules! backoff {
    ($B:ident) => {
        $B
    };
    () => {
        NoBackoff
    };
}

/// Single-producer sending half. Requires exclusive access (`&mut self`) to send.
pub struct Tx<T, Ch: Channel, SP: SyncPrimitives = DefaultSyncPrimitives> {
    chan: Arc<Chan<T, Ch, SP>>,
}
tx_half!(Tx, mut);
channel_half!(Tx);

/// Multi-producer cloneable sending half.
pub struct MTx<
    T,
    Ch: Channel,
    B: BackoffStrategy = NoBackoff,
    SP: SyncPrimitives = DefaultSyncPrimitives,
> {
    chan: Arc<Chan<T, Ch, SP>>,
    _backoff: PhantomData<B>,
}
tx_half!(MTx<B>);
channel_half!(MTx<B>);
cloneable_half!(MTx<B>);

/// Single-producer unbounded sending half, which never blocks. Requires exclusive access (`&mut self`) to send.
pub struct UTx<T, Ch: Channel, SP: SyncPrimitives = DefaultSyncPrimitives> {
    chan: Arc<Chan<T, Ch, SP>>,
}
utx_methods!(UTx, mut);
channel_half!(UTx);

/// Multi-producer cloneable unbounded sending half, which never blocks.
pub struct UMTx<
    T,
    Ch: Channel,
    B: BackoffStrategy = NoBackoff,
    SP: SyncPrimitives = DefaultSyncPrimitives,
> {
    chan: Arc<Chan<T, Ch, SP>>,
    _backoff: PhantomData<B>,
}
utx_methods!(UMTx<B>);
channel_half!(UMTx<B>);
cloneable_half!(UMTx<B>);

/// Single-consumer receiving half. Requires exclusive access (`&mut self`) to receive.
pub struct Rx<T, Ch: Channel, SP: SyncPrimitives = DefaultSyncPrimitives> {
    chan: Arc<Chan<T, Ch, SP>>,
}
rx_half!(Rx, mut);
channel_half!(Rx);

/// Multi-consumer cloneable receiving half.
pub struct MRx<
    T,
    Ch: Channel,
    B: BackoffStrategy = NoBackoff,
    SP: SyncPrimitives = DefaultSyncPrimitives,
> {
    chan: Arc<Chan<T, Ch, SP>>,
    _backoff: PhantomData<B>,
}
rx_half!(MRx<B>);
channel_half!(MRx<B>);
cloneable_half!(MRx<B>);

impl<T, Ch: Channel, SP: SyncPrimitives> Rx<T, Ch, SP> {
    /// Poll-based receive for use in manual [`Future`] or
    /// [`Stream`](futures_core::stream::Stream) implementations.
    pub fn poll_recv(&mut self, cx: &mut Context<'_>) -> Poll<Result<T, RecvError>> {
        const { assert!(size_of::<<Ch::RxWaiter<SP> as Waiter<SP>>::Wait<'static>>() == 0) };
        pin!(self.recv()).poll(cx)
    }
}

#[cfg(feature = "stream")]
impl<T, Ch: Channel, SP: SyncPrimitives> futures_core::Stream for Rx<T, Ch, SP> {
    type Item = T;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.poll_recv(cx).map(Result::ok)
    }
}

/// A non-owning reference to a cloneable channel half. Does not prevent the channel from closing.
pub struct Weak<H: ChannelHalf + Clone>(pub(crate) ManuallyDrop<H>);

impl<H: ChannelHalf + Clone> Deref for Weak<H> {
    type Target = H;

    fn deref(&self) -> &H {
        &self.0
    }
}

impl<H: ChannelHalf + Clone> DerefMut for Weak<H> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<H: ChannelHalf + Clone> Weak<H> {
    pub(crate) fn new(end: &H) -> Self {
        Weak(ManuallyDrop::new(end.raw_clone()))
    }

    /// Attempts to obtain a strong reference. Returns `None` if all strong handles have been
    /// dropped and the channel is closed.
    pub fn upgrade(&self) -> Option<H> {
        let incr = |c| (c != 0).then(|| c + 1);
        let strong = self.0.chan().ref_cnt(H::HALF).unwrap();
        #[allow(deprecated)]
        strong.fetch_update(Relaxed, Acquire, incr).ok()?;
        Some(self.0.raw_clone())
    }
}

impl<H: ChannelHalf + Clone> Drop for Weak<H> {
    fn drop(&mut self) {
        drop(unsafe { Arc::from_raw(Arc::as_ptr(self.0.chan())) });
    }
}

impl<H: ChannelHalf + Clone> Clone for Weak<H> {
    fn clone(&self) -> Self {
        Self(ManuallyDrop::new(self.0.raw_clone()))
    }
}

impl<H: ChannelHalf + fmt::Debug + Clone> fmt::Debug for Weak<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Weak").field(&self.0).finish()
    }
}
