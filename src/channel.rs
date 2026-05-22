use core::{
    fmt, mem,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    panic::{RefUnwindSafe, UnwindSafe},
    pin::{Pin, pin},
    ptr::NonNull,
    sync::atomic::{
        AtomicUsize,
        Ordering::{AcqRel, Acquire, Relaxed},
    },
    task::{Context, Poll, ready},
};

use aiq::wait_queue::Wait;
use crossbeam_utils::CachePadded;

#[cfg(feature = "blocking")]
use crate::blocking::*;
use crate::{
    errors::{AcquireError, RecvError, SendError, TryAcquireError, TryRecvError, TrySendError},
    internal,
    loom::sync::Arc,
    rc::RefCount,
    waiter::Waiter,
};

/// A channel implementation.
#[expect(private_bounds)]
pub trait Channel: internal::Channel {
    /// Sender half of the channel.
    type TxHalf<T>: ChannelHalf<Msg = T, Channel = Self>;
    /// Receiver half of the channel.
    type RxHalf<T>: ChannelHalf<Msg = T, Channel = Self>;
    /// Creates a sender/receiver pair using this implementation.
    fn channel<T>(self) -> (Self::TxHalf<T>, Self::RxHalf<T>) {
        use internal::ChannelHalf;
        let chan = Arc::new(Chan::new(self));
        (Self::TxHalf::new(chan.clone()), Self::RxHalf::new(chan))
    }
}

/// A bounded channel.
pub trait BoundedChannel: Channel {}

/// A sender or receiver half of a channel.
#[expect(private_bounds)]
pub trait ChannelHalf: internal::ChannelHalf<Self::Msg, Self::Channel> {
    /// The message type of the channel.
    type Msg;
    /// The implementation of the channel.
    type Channel: Channel;
}

/// Opaque unique identifier for a channel instance. Two halves sharing the same `ChannelId`
/// belong to the same channel.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ChannelId(NonNull<()>);

trait Operation<T, Ch: internal::Channel>: 'static {
    type State;
    type Slot;
    type Waiter: Waiter;
    fn acquire_slot(chan: &Chan<T, Ch>) -> Result<Self::Slot, Self::State>;
    fn acquire_slot_cold(
        chan: &Chan<T, Ch>,
        state: &mut Self::State,
        first_call: bool,
    ) -> Result<Self::Slot, TryAcquireError>;
    fn waiter(chan: &Chan<T, Ch>) -> &Self::Waiter;
}

struct SendMsg;
impl<T, Ch: internal::Channel> Operation<T, Ch> for SendMsg {
    type State = Ch::TxState<T>;
    type Slot = Ch::TxSlot<T>;
    type Waiter = Ch::TxWaiter;
    fn acquire_slot(chan: &Chan<T, Ch>) -> Result<Self::Slot, Self::State> {
        Ch::tx_acquire_slot(chan)
    }
    fn acquire_slot_cold(
        chan: &Chan<T, Ch>,
        state: &mut Self::State,
        first_call: bool,
    ) -> Result<Self::Slot, TryAcquireError> {
        Ch::tx_acquire_slot_cold(chan, state, first_call)
    }
    fn waiter(chan: &Chan<T, Ch>) -> &Self::Waiter {
        &chan.tx_waiter
    }
}

struct RecvMsg;
impl<T, Ch: internal::Channel> Operation<T, Ch> for RecvMsg {
    type State = Ch::RxState<T>;
    type Slot = Ch::RxSlot<T>;
    type Waiter = Ch::RxWaiter;
    fn acquire_slot(chan: &Chan<T, Ch>) -> Result<Self::Slot, Self::State> {
        Ch::rx_acquire_slot(chan)
    }
    fn acquire_slot_cold(
        chan: &Chan<T, Ch>,
        state: &mut Self::State,
        first_call: bool,
    ) -> Result<Self::Slot, TryAcquireError> {
        Ch::rx_acquire_slot_cold(chan, state, first_call)
    }
    fn waiter(chan: &Chan<T, Ch>) -> &Self::Waiter {
        &chan.rx_waiter
    }
}

// fields are duplicated instead of using an intermediate struct
// to keep them packed on the same cache line
pub(crate) struct Chan<T, Ch: internal::Channel> {
    pub(crate) tx_state: CachePadded<Ch::TxAtomicState<T>>,
    pub(crate) rx_state: CachePadded<Ch::RxAtomicState<T>>,
    pub(crate) storage: Ch::Storage<T>,
    pub(crate) tx_waiter: Ch::TxWaiter,
    pub(crate) rx_waiter: Ch::RxWaiter,
    tx_cnt: Ch::TxRefCount,
    rx_cnt: Ch::RxRefCount,
    close_waiter: aiq::WaitQueue,
}

unsafe impl<T: Send, Ch: internal::Channel> Send for Chan<T, Ch> {}
unsafe impl<T: Send, Ch: internal::Channel> Sync for Chan<T, Ch> {}
impl<T, Ch: internal::Channel> UnwindSafe for Chan<T, Ch> {}
impl<T, Ch: internal::Channel> RefUnwindSafe for Chan<T, Ch> {}

impl<T, Ch: internal::Channel> Chan<T, Ch> {
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
            close_waiter: Default::default(),
        }
    }

    fn id(&self) -> ChannelId {
        ChannelId(NonNull::from(self).cast())
    }

    fn close(&self) {
        Ch::close(self);
        self.tx_waiter.close();
        self.rx_waiter.close();
        self.close_waiter.close();
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
        self.rx_waiter.wake();
        Ok(())
    }

    #[inline(always)]
    fn read_slot<E>(&self, slot: Ch::RxSlot<T>) -> Result<T, E> {
        let msg = Ch::read_slot(self, slot);
        if Ch::WAKE_TX_AFTER_READ {
            self.tx_waiter.wake();
        }
        Ok(msg)
    }

    #[inline(always)]
    fn try_acquire_slot<O: Operation<T, Ch>>(&self) -> Result<O::Slot, TryAcquireError> {
        O::acquire_slot(self).or_else(|state| self.try_acquire_slot_cold::<O>(state))
    }

    #[cold]
    fn try_acquire_slot_cold<O: Operation<T, Ch>>(
        &self,
        mut state: O::State,
    ) -> Result<O::Slot, TryAcquireError> {
        O::acquire_slot_cold(self, &mut state, true)
    }

    #[inline(always)]
    fn try_send(&self, msg: T) -> Result<(), TrySendError<T>> {
        self.write_slot(self.try_acquire_slot::<SendMsg>(), msg)
    }

    #[inline(always)]
    fn send_unbounded(&self, msg: T) -> Result<(), SendError<T>> {
        match self.try_send(msg) {
            Ok(()) => Ok(()),
            Err(TrySendError::Closed(msg)) => Err(SendError(msg)),
            Err(TrySendError::Full(_)) => unsafe { core::hint::unreachable_unchecked() },
        }
    }

    #[inline(always)]
    fn try_recv(&self) -> Result<T, TryRecvError> {
        self.read_slot(self.try_acquire_slot::<RecvMsg>()?)
    }

    #[inline(always)]
    fn poll_acquire_slot<'a, O: Operation<T, Ch>>(
        &'a self,
        cx: &mut Context<'_>,
        wait: Pin<&mut <O::Waiter as Waiter>::Wait<'a>>,
    ) -> Poll<Result<O::Slot, AcquireError>> {
        match O::acquire_slot(self) {
            Ok(slot) => Poll::Ready(Ok(slot)),
            Err(state) => self.poll_acquire_slot_cold::<O>(cx, wait, state),
        }
    }

    #[cold]
    fn poll_acquire_slot_cold<'a, O: Operation<T, Ch>>(
        &'a self,
        cx: &mut Context<'_>,
        mut wait: Pin<&mut <O::Waiter as Waiter>::Wait<'a>>,
        mut state: O::State,
    ) -> Poll<Result<O::Slot, AcquireError>> {
        let mut first_call = true;
        let mut waker_registered = false;
        loop {
            match O::acquire_slot_cold(self, &mut state, first_call) {
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
            first_call = false;
        }
    }

    #[cfg(feature = "blocking")]
    fn acquire_slot_blocking<O: Operation<T, Ch>>(
        &self,
        parker: impl Into<Parker>,
    ) -> Result<O::Slot, TryAcquireError> {
        O::acquire_slot(self)
            .or_else(|state| self.acquire_slot_blocking_cold::<O>(state, parker.into()))
    }

    #[cfg(feature = "blocking")]
    #[cold]
    fn acquire_slot_blocking_cold<O: Operation<T, Ch>>(
        &self,
        mut state: O::State,
        mut parker: Parker,
    ) -> Result<O::Slot, TryAcquireError> {
        let mut wait = core::pin::pin!(<O::Waiter as Waiter>::Wait::default());
        let mut first_call = true;
        let mut waker_registered = false;
        loop {
            match O::acquire_slot_cold(self, &mut state, first_call) {
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
            first_call = false;
        }
    }

    #[cfg(feature = "blocking")]
    #[inline(always)]
    fn send_blocking(&self, msg: T) -> Result<(), SendError<T>> {
        let slot = self.acquire_slot_blocking::<SendMsg>(());
        self.write_slot(slot, msg)
    }

    #[cfg(feature = "blocking")]
    #[inline(always)]
    fn send_deadline(&self, msg: T, deadline: Instant) -> Result<(), SendTimeoutError<T>> {
        let slot = self.acquire_slot_blocking::<SendMsg>(deadline);
        self.write_slot(slot, msg)
    }

    #[cfg(feature = "blocking")]
    #[inline(always)]
    fn send_timeout(&self, msg: T, timeout: Duration) -> Result<(), SendTimeoutError<T>> {
        let slot = self.acquire_slot_blocking::<SendMsg>(timeout);
        self.write_slot(slot, msg)
    }

    #[cfg(feature = "blocking")]
    #[inline(always)]
    fn recv_blocking(&self) -> Result<T, RecvError> {
        let slot = self.acquire_slot_blocking::<RecvMsg>(())?;
        self.read_slot(slot)
    }

    #[cfg(feature = "blocking")]
    #[inline(always)]
    fn recv_deadline(&self, deadline: Instant) -> Result<T, RecvTimeoutError> {
        let slot = self.acquire_slot_blocking::<RecvMsg>(deadline)?;
        self.read_slot(slot)
    }

    #[cfg(feature = "blocking")]
    #[inline(always)]
    fn recv_timeout(&self, timeout: Duration) -> Result<T, RecvTimeoutError> {
        let slot = self.acquire_slot_blocking::<RecvMsg>(timeout)?;
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

impl<T, Ch: internal::Channel> Drop for Chan<T, Ch> {
    fn drop(&mut self) {
        Ch::drop_storage(self);
    }
}

impl<T, Ch: internal::Channel> Deref for Chan<T, Ch> {
    type Target = Ch::Storage<T>;

    fn deref(&self) -> &Self::Target {
        &self.storage
    }
}

impl<T, Ch: internal::Channel> fmt::Debug for Chan<T, Ch> {
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
pub struct SendFuture<'a, T, Ch: Channel> {
    chan: &'a Chan<T, Ch>,
    msg: Option<T>,
    wait: <Ch::TxWaiter as Waiter>::Wait<'a>,
}

impl<'a, T, Ch: Channel> SendFuture<'a, T, Ch> {
    #[inline(always)]
    fn new(chan: &'a Chan<T, Ch>, msg: T) -> Self {
        Self {
            chan,
            msg: Some(msg),
            wait: Default::default(),
        }
    }

    /// Cancels the send, returning the message if it has not yet been sent.
    pub fn cancel(self: Pin<&mut Self>) -> Option<T> {
        unsafe { self.get_unchecked_mut() }.msg.take()
    }
}

impl<T, Ch: Channel> Future for SendFuture<'_, T, Ch> {
    type Output = Result<(), SendError<T>>;

    #[inline(always)]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let wait = unsafe { Pin::new_unchecked(&mut this.wait) };
        let slot = ready!(this.chan.poll_acquire_slot::<SendMsg>(cx, wait));
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
pub struct RecvFuture<'a, T, Ch: Channel> {
    chan: &'a Chan<T, Ch>,
    wait: <Ch::RxWaiter as Waiter>::Wait<'a>,
}

impl<'a, T, Ch: Channel> RecvFuture<'a, T, Ch> {
    #[inline(always)]
    fn new(chan: &'a Chan<T, Ch>) -> Self {
        Self {
            chan,
            wait: Default::default(),
        }
    }
}

impl<T, Ch: Channel> Future for RecvFuture<'_, T, Ch> {
    type Output = Result<T, RecvError>;

    #[inline(always)]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let wait = unsafe { Pin::new_unchecked(&mut this.wait) };
        let slot = ready!(this.chan.poll_acquire_slot::<RecvMsg>(cx, wait))?;
        Poll::Ready(this.chan.read_slot(slot))
    }
}

#[cfg(feature = "stream")]
impl<T, Ch: Channel> futures_core::Stream for RecvFuture<'_, T, Ch> {
    type Item = T;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.poll(cx).map(Result::ok)
    }
}

/// Future returned by [`CloseHandle::closed`] and similar methods.
///
/// Resolves once the channel is closed.
pub struct ClosedFuture<'a>(Wait<&'a aiq::WaitQueue>);

impl<'a> ClosedFuture<'a> {
    fn new<T, Ch: internal::Channel>(chan: &'a Chan<T, Ch>) -> Self {
        Self(chan.close_waiter.wait())
    }
}

impl Future for ClosedFuture<'_> {
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

impl<T: Send, Ch: internal::Channel> Close for Chan<T, Ch> {
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
    pub(crate) fn new<T: Send + 'a, Ch: Channel>(chan: Arc<Chan<T, Ch>>) -> Self {
        #[cfg(not(loom))]
        return Self(chan as _);
        #[cfg(loom)]
        unimplemented!("{chan:?}");
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
    ($ty:ident) => {
        impl<T, Ch: Channel> Drop for $ty<T, Ch> {
            fn drop(&mut self) {
                self.chan.drop_half(Half::$ty);
            }
        }
        impl<T, Ch: Channel> internal::ChannelHalf<T, Ch> for $ty<T, Ch> {
            const HALF: Half = Half::$ty;
            fn new(chan: Arc<Chan<T, Ch>>) -> Self {
                Self { chan }
            }
            fn chan(&self) -> &Arc<Chan<T, Ch>> {
                &self.chan
            }
        }
        impl<T, Ch: Channel> ChannelHalf for $ty<T, Ch> {
            type Msg = T;
            type Channel = Ch;
        }
        unsafe impl<T: Send, Ch: Channel> Send for $ty<T, Ch> {}
        unsafe impl<T: Send, Ch: Channel> Sync for $ty<T, Ch> {}
        impl<T, Ch: Channel> UnwindSafe for $ty<T, Ch> {}
        impl<T, Ch: Channel> RefUnwindSafe for $ty<T, Ch> {}
        impl<T, Ch: Channel> $ty<T, Ch> {
            /// Returns `true` if the channel is closed.
            pub fn is_closed(&self) -> bool {
                Ch::is_closed(&self.chan)
            }
            /// Closes the channel, preventing new messages to be sent.
            pub fn close(&self) {
                self.chan.close();
            }
            /// Returns a [`CloseHandle`] that can close the channel or observe its closure
            /// without holding this half.
            pub fn close_handle<'a>(&self) -> CloseHandle<'a>
            where
                T: Send + 'a,
            {
                CloseHandle::new(self.chan.clone())
            }
            /// Returns a unique identifier for the underlying channel.
            pub fn channel_id(&self) -> ChannelId {
                self.chan.id()
            }
            /// Returns a future that resolves once the channel is closed.
            pub fn closed(&self) -> ClosedFuture<'_> {
                ClosedFuture::new(&self.chan)
            }
            /// Returns the capacity of the bounded channel.
            pub fn capacity(&self) -> usize
            where
                Ch: BoundedChannel,
            {
                Ch::capacity(&self.chan).unwrap()
            }
        }
        impl<T, Ch: Channel> core::fmt::Debug for $ty<T, Ch> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_tuple(stringify!($ty)).field(&self.chan).finish()
            }
        }
    };
}
macro_rules! tx_half {
    ($ty:ident $(, $mut:tt)?) => {
        impl<T, Ch: Channel> $ty<T, Ch> {
            /// Sends a message, waiting asynchronously if the channel is full.
            #[inline]
            pub fn send(&$($mut)? self, msg: T) -> SendFuture<'_, T, Ch> {
                SendFuture::new(&self.chan, msg)
            }
            /// Sends a message, blocking the current thread if the channel is full.
            #[cfg(feature = "blocking")]
            #[inline]
            pub fn send_blocking(&$($mut)? self, msg: T) -> Result<(), SendError<T>> {
                self.chan.send_blocking(msg)
            }
            /// Sends a message, blocking until `deadline` if the channel is full.
            #[cfg(feature = "blocking")]
            #[inline]
            pub fn send_deadline(&$($mut)? self, msg: T, deadline: Instant) -> Result<(), SendTimeoutError<T>> {
                self.chan.send_deadline(msg, deadline)
            }
            /// Sends a message, blocking for up to `timeout` if the channel is full.
            #[cfg(feature = "blocking")]
            #[inline]
            pub fn send_timeout(&$($mut)? self, msg: T, timeout: Duration) -> Result<(), SendTimeoutError<T>> {
                self.chan.send_timeout(msg, timeout)
            }
            /// Attempts to send a message without waiting.
            #[inline]
            pub fn try_send(&$($mut)? self, msg: T) -> Result<(), TrySendError<T>> {
                self.chan.try_send(msg)
            }
            /// Returns `true` if the channel is full.
            pub fn is_full(&self) -> bool {
                Ch::is_full(&self.chan)
            }
        }
    };
}
macro_rules! utx_methods {
    ($ty:ident $(, $mut:tt)?) => {
        impl<T, Ch: Channel> $ty<T, Ch> {
            /// Sends a message. Never blocks or returns [`TrySendError::Full`] since the channel
            /// is unbounded.
            #[inline]
            pub fn send_unbounded(&$($mut)? self, msg: T) -> Result<(), SendError<T>> {
                self.chan.send_unbounded(msg)
            }
        }
    };
}
macro_rules! rx_half {
    ($ty:ident $(, $mut:tt)?) => {
        impl<T, Ch: Channel> $ty<T, Ch> {
            /// Receives a message, waiting asynchronously if the channel is empty.
            #[inline]
            pub fn recv(&$($mut)? self) -> RecvFuture<'_, T, Ch> {
                RecvFuture::new(&self.chan)
            }
            /// Receives a message, blocking the current thread if the channel is empty.
            #[cfg(feature = "blocking")]
            #[inline]
            pub fn recv_blocking(&$($mut)? self) -> Result<T, RecvError> {
                self.chan.recv_blocking()
            }
            /// Receives a message, blocking until `deadline` if the channel is empty.
            #[cfg(feature = "blocking")]
            #[inline]
            pub fn recv_deadline(&$($mut)? self, deadline: Instant) -> Result<T, RecvTimeoutError> {
                self.chan.recv_deadline(deadline)
            }
            /// Receives a message, blocking for up to `timeout` if the channel is empty.
            #[cfg(feature = "blocking")]
            #[inline]
            pub fn recv_timeout(&$($mut)? self, timeout: Duration) -> Result<T, RecvTimeoutError> {
                self.chan.recv_timeout(timeout)
            }
            /// Returns a blocking iterator that yields messages until the channel is closed.
            #[cfg(feature = "blocking")]
            pub fn iter_blocking(&$($mut)? self) -> impl Iterator<Item = T> {
                core::iter::from_fn(|| self.recv_blocking().ok())
            }
            /// Attempts to receive a message without waiting.
            #[inline]
            pub fn try_recv(&$($mut)? self) -> Result<T, TryRecvError> {
                self.chan.try_recv()
            }
            /// Returns `true` if the channel is empty.
            pub fn is_empty(&self) -> bool {
                Ch::is_empty(&self.chan)
            }
        }
    };
}
macro_rules! cloneable_half {
    ($ty:ident) => {
        impl<T, Ch: Channel> Clone for $ty<T, Ch> {
            fn clone(&self) -> Self {
                self.chan.clone_half(Half::$ty);
                internal::ChannelHalf::raw_clone(self)
            }
        }
        impl<T, Ch: Channel> $ty<T, Ch> {
            /// Returns a [`Weak`] reference that does not prevent the channel from being closed.
            pub fn downgrade(&self) -> Weak<Self> {
                Weak::new(self)
            }
        }
    };
}

/// Single-producer sending half. Requires exclusive access (`&mut self`) to send.
pub struct Tx<T, Ch: Channel> {
    chan: Arc<Chan<T, Ch>>,
}
tx_half!(Tx, mut);
channel_half!(Tx);

/// Multi-producer cloneable sending half.
pub struct MTx<T, Ch: Channel> {
    chan: Arc<Chan<T, Ch>>,
}
tx_half!(MTx);
channel_half!(MTx);
cloneable_half!(MTx);

/// Single-producer unbounded sending half, which never blocks. Requires exclusive access (`&mut self`) to send.
pub struct UTx<T, Ch: Channel> {
    chan: Arc<Chan<T, Ch>>,
}
utx_methods!(UTx, mut);
channel_half!(UTx);

/// Multi-producer cloneable unbounded sending half, which never blocks.
pub struct UMTx<T, Ch: Channel> {
    chan: Arc<Chan<T, Ch>>,
}
utx_methods!(UMTx);
channel_half!(UMTx);
cloneable_half!(UMTx);

/// Single-consumer receiving half. Requires exclusive access (`&mut self`) to receive.
pub struct Rx<T, Ch: Channel> {
    chan: Arc<Chan<T, Ch>>,
}
rx_half!(Rx, mut);
channel_half!(Rx);

/// Multi-consumer cloneable receiving half.
pub struct MRx<T, Ch: Channel> {
    chan: Arc<Chan<T, Ch>>,
}
rx_half!(MRx);
channel_half!(MRx);
cloneable_half!(MRx);

impl<T, Ch: Channel> Rx<T, Ch> {
    /// Poll-based receive for use in manual [`Future`] or
    /// [`Stream`](futures_core::stream::Stream) implementations.
    pub fn poll_recv(&mut self, cx: &mut Context<'_>) -> Poll<Result<T, RecvError>> {
        const { assert!(size_of::<<Ch::RxWaiter as Waiter>::Wait<'static>>() == 0) };
        pin!(self.recv()).poll(cx)
    }
}

#[cfg(feature = "stream")]
impl<T, Ch: Channel> futures_core::Stream for Rx<T, Ch> {
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
