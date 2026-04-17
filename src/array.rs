// use alloc::alloc::{alloc, alloc_zeroed, dealloc, handle_alloc_error};
// use core::{
//     alloc::Layout,
//     marker::PhantomData,
//     mem::MaybeUninit,
//     ops::Range,
//     ptr::NonNull,
//     sync::atomic::Ordering::{Acquire, Relaxed, Release, SeqCst},
// };
//
// use aiq::{
//     sync::{DefaultSyncPrimitives, SyncPrimitives},
//     WaitQueue,
// };
// use spmc_waker::SpmcWaker;
//
// use crate::{
//     capacity::Capacity,
//     channel,
//     channel::{Channel, ChannelHalf, MRx, MTx, Multiple, Rx, Single, Tx},
//     errors::{SendError, TryAcquireError},
//     loom::{
//         cell::{Cell, UnsafeCell}, sync::atomic::{AtomicBool, AtomicUsize},
//         AtomicUsizeExt,
//         UnsafeCellExt,
//     },
//     private,
//     state::{HalfState, RxState, TxState},
//     waiter::Waiter,
// };
//
// #[allow(private_bounds)]
// pub trait Multiplicity {
//     const CONCURRENT: bool;
//     type ArrayTxHalf<T, Ch: Channel>: ChannelHalf<Msg = T, Channel = Ch>;
//     type ArrayRxHalf<T, Ch: Channel>: ChannelHalf<Msg = T, Channel = Ch>;
//     type ArrayIndex: ArrayIndex;
//     type Waiter: Waiter;
//     type StateValue;
//     fn state(index: usize) -> Self::StateValue;
//     fn index(state: Self::StateValue) -> Option<usize>;
// }
//
// pub struct Array<
//     PM: channel::Multiplicity,
//     CM: channel::Multiplicity,
//     C: Capacity = usize,
//     const UNBOUNDED_BACKOFF: bool = false,
//     SP: SyncPrimitives = DefaultSyncPrimitives,
// > {
//     capacity: C,
//     _kind: PhantomData<(PM, CM)>,
//     _sync: PhantomData<SP>,
// }
//
// impl<
//     PM: channel::Multiplicity,
//     CM: channel::Multiplicity,
//     C: Capacity,
//     const UNBOUNDED_BACKOFF: bool,
//     SP: SyncPrimitives,
// > Array<PM, CM, C, UNBOUNDED_BACKOFF, SP>
// {
//     pub fn new(capacity: C) -> Self {
//         Self {
//             capacity,
//             _kind: PhantomData,
//             _sync: PhantomData,
//         }
//     }
// }
//
// impl<
//     PM: channel::Multiplicity,
//     CM: channel::Multiplicity,
//     C: Capacity,
//     const UNBOUNDED_BACKOFF: bool,
//     SP: SyncPrimitives,
// > Channel for Array<PM, CM, C, UNBOUNDED_BACKOFF, SP>
// {
//     type TxHalf<T> = PM::ArrayTxHalf<T, Self>;
//     type RxHalf<T> = PM::ArrayTxHalf<T, Self>;
// }
//
// pub trait Kind {
//     const IS_SPSC: bool;
// }
//
// impl<PM: Multiplicity, CM: Multiplicity> Kind for (PM, CM) {
//     const IS_SPSC: bool = !PM::CONCURRENT & !CM::CONCURRENT;
// }
//
// pub struct Slot<T, const UNBOUNDED_BACKOFF: bool, K: Kind> {
//     msg: UnsafeCell<MaybeUninit<T>>,
//     stamp: AtomicUsize,
//     _kind: PhantomData<K>,
// }
//
// impl<T, const UNBOUNDED_BACKOFF: bool, K: Kind> Slot<T, UNBOUNDED_BACKOFF, K> {
//     fn alloc_buffer(capacity: usize) -> NonNull<Self> {
//         assert!(capacity > 0);
//         let layout = Layout::array::<Self>(capacity).expect("capacity overflow");
//         let ptr = if K::IS_SPSC {
//             unsafe { alloc_zeroed(layout) }.cast()
//         } else {
//             unsafe { alloc(layout) }.cast()
//         };
//         let buffer = NonNull::new(ptr).unwrap_or_else(|| handle_alloc_error(layout));
//         if cfg!(loom) || !K::IS_SPSC {
//             for i in 0..capacity {
//                 let stamp = if K::IS_SPSC { 0 } else { i };
//                 let slot = Self {
//                     msg: UnsafeCell::new(MaybeUninit::uninit()),
//                     stamp: AtomicUsize::new(stamp),
//                     _kind: PhantomData,
//                 };
//                 unsafe { buffer.add(i).write(slot) };
//             }
//         }
//         buffer
//     }
//
//     unsafe fn dealloc_buffer(buffer: NonNull<Self>, capacity: usize, range: Range<usize>) {
//         debug_assert!(capacity > 0);
//         let layout = unsafe { Layout::array::<Self>(capacity).unwrap_unchecked() };
//         for i in range {
//             let slot = unsafe { buffer.add(i).as_mut() };
//             unsafe { slot.msg.with_ref_mut(|msg| msg.assume_init_drop()) };
//         }
//         #[cfg(loom)]
//         for i in 0..capacity {
//             unsafe { buffer.add(i).drop_in_place() };
//         }
//         unsafe { dealloc(buffer.as_ptr().cast(), layout) };
//     }
//
//     fn can_write(&self, tail: usize) -> bool {
//         if K::IS_SPSC {
//             self.stamp.load(SeqCst) == 1
//         } else {
//             self.stamp.load(SeqCst) == tail
//         }
//     }
//
//     #[cfg_attr(loom, allow(unused_variables))]
//     unsafe fn write(&self, msg: T, tail: usize) {
//         unsafe { self.msg.with_ref_mut(|cell| cell.write(msg)) };
//         let order = if UNBOUNDED_BACKOFF { Release } else { SeqCst };
//         if K::IS_SPSC {
//             self.stamp.store(1, order);
//         } else {
//             self.stamp.store(tail + 1, order);
//         }
//     }
//
//     fn can_read(&self, head: usize) -> bool {
//         let expected = if K::IS_SPSC { 1 } else { head };
//         self.stamp.load(Acquire) == expected
//     }
//
//     unsafe fn read(&self, head: usize, mask: usize) -> T {
//         let msg = unsafe { self.msg.with_ref(|cell| cell.assume_init_read()) };
//         if K::IS_SPSC {
//             self.stamp.store(1, SeqCst);
//         } else {
//             self.stamp.store(head + mask, SeqCst);
//         }
//         msg
//     }
//
//     unsafe fn wait_write(&self, head: usize, spsc: bool) {
//         assert!(UNBOUNDED_BACKOFF);
//         let expected = if spsc { 1 } else { head };
//         #[cfg(not(loom))]
//         let backoff = crossbeam_utils::Backoff::new();
//         while self.stamp.load(Acquire) != expected {
//             #[cfg(not(loom))]
//             backoff.snooze();
//             #[cfg(loom)]
//             crate::loom::hint::spin_loop();
//         }
//     }
// }
//
// pub struct ArrayState<
//     T,
//     M: Multiplicity,
//     C: private::Capacity,
//     const UNBOUNDED_BACKOFF: bool,
//     K: Kind,
//     const TX: bool,
// > {
//     index: M::ArrayIndex,
//     capacity: C,
//     mask: C::Mask,
//     buffer: NonNull<Slot<T, UNBOUNDED_BACKOFF, K>>,
// }
//
// impl<
//     T,
//     M: Multiplicity,
//     C: private::Capacity,
//     const UNBOUNDED_BACKOFF: bool,
//     K: Kind,
//     const TX: bool,
// > ArrayState<T, M, C, UNBOUNDED_BACKOFF, K, TX>
// {
//     fn new(capacity: C, buffer: NonNull<Slot<T, UNBOUNDED_BACKOFF, K>>) -> Self {
//         Self {
//             index: Default::default(),
//             capacity,
//             mask: capacity.mask(),
//             buffer,
//         }
//     }
//
//     fn capacity(&self) -> usize {
//         self.capacity.get()
//     }
//
//     fn mask(&self) -> usize {
//         C::mask_get(self.mask)
//     }
//
//     fn closed_flag(&self) -> usize {
//         (self.mask() >> 1) + 1
//     }
//
//     fn slot(&self, index: usize) -> Option<&Slot<T, UNBOUNDED_BACKOFF, K>> {
//         let slot_idx = index & self.mask();
//         if M::CONCURRENT && slot_idx >= self.capacity() {
//             return None;
//         }
//         Some(unsafe { self.buffer.add(slot_idx).as_ref() })
//     }
//
//     fn next_index(&self, index: usize) -> usize {
//         let next_index = index + 1;
//         if next_index & self.mask() == self.capacity() {
//             (index & !self.mask()).wrapping_add(self.mask().wrapping_add(1))
//         } else {
//             next_index
//         }
//     }
// }
//
// trait ArrayIndex: Default {
//     fn load(&self) -> usize;
//     fn load_mut(&mut self) -> usize;
//     fn cas(&self, current: usize, new: usize) -> Result<usize, usize>;
//     fn is_closed(&self, flag: usize) -> bool;
//     fn close(&self, flag: usize);
// }
// #[derive(Default)]
// pub struct CellIndex {
//     index: Cell<usize>,
//     closed: AtomicBool,
// }
//
// impl Multiplicity for Single {
//     const CONCURRENT: bool = false;
//     type ArrayTxHalf<T, Ch: Channel> = Tx<T, Ch>;
//     type ArrayRxHalf<T, Ch: Channel> = Rx<T, Ch>;
//     type ArrayIndex = CellIndex;
//     type Waiter = SpmcWaker<false>;
//     type StateValue = ();
//     fn state(_index: usize) -> Self::StateValue {}
//     fn index(_state: Self::StateValue) -> Option<usize> {
//         None
//     }
// }
//
// impl Multiplicity for Multiple {
//     const CONCURRENT: bool = true;
//     type ArrayTxHalf<T, Ch: Channel> = MTx<T, Ch>;
//     type ArrayRxHalf<T, Ch: Channel> = MRx<T, Ch>;
//     type ArrayIndex = AtomicUsize;
//     type Waiter = WaitQueue;
//     type StateValue = usize;
//     fn state(index: usize) -> Self::StateValue {
//         index
//     }
//     fn index(state: Self::StateValue) -> Option<usize> {
//         Some(state)
//     }
// }
//
// impl<
//     PM: channel::Multiplicity,
//     CM: channel::Multiplicity,
//     C: Capacity,
//     const UNBOUNDED_BACKOFF: bool,
//     SP: SyncPrimitives,
// > private::Channel for Array<PM, CM, C, UNBOUNDED_BACKOFF, SP>
// {
//     type TxState<T> = ArrayState<T, PM, C, UNBOUNDED_BACKOFF, (PM, CM), true>;
//     type RxState<T> = ArrayState<T, CM, C, UNBOUNDED_BACKOFF, (PM, CM), false>;
//
//     fn init_storage<T>(self) -> (Self::TxState<T>, Self::RxState<T>) {
//         let buffer = Slot::<T, UNBOUNDED_BACKOFF, (PM, CM)>::alloc_buffer(self.capacity.get());
//         let tx_state = ArrayState::new(self.capacity, buffer);
//         let rx_state = ArrayState::new(self.capacity, buffer);
//         (tx_state, rx_state)
//     }
//
//     fn drop_storage<T>(tx_state: &mut Self::TxState<T>, rx_state: &mut Self::RxState<T>) {
//         let index_mask = C::mask_get(tx_state.mask) >> 1;
//         let tail = tx_state.index.load_mut() & index_mask;
//         let head = rx_state.index.load_mut() & index_mask;
//         unsafe { Slot::dealloc_buffer(tx_state.buffer, tx_state.capacity.get(), head..tail) };
//     }
// }
//
// impl ArrayIndex for AtomicUsize {
//     fn load(&self) -> usize {
//         self.load(Relaxed)
//     }
//     fn load_mut(&mut self) -> usize {
//         AtomicUsizeExt::load_mut(self)
//     }
//     fn cas(&self, current: usize, new: usize) -> Result<usize, usize> {
//         self.compare_exchange_weak(current, new, SeqCst, Relaxed)
//     }
//     fn is_closed(&self, flag: usize) -> bool {
//         self.load(Relaxed) & flag != 0
//     }
//     fn close(&self, flag: usize) {
//         self.fetch_or(flag, SeqCst);
//     }
// }
// impl ArrayIndex for CellIndex {
//     fn load(&self) -> usize {
//         self.index.get()
//     }
//     fn load_mut(&mut self) -> usize {
//         self.index.get()
//     }
//     fn cas(&self, current: usize, new: usize) -> Result<usize, usize> {
//         self.index.set(new);
//         Ok(current)
//     }
//     fn is_closed(&self, _flag: usize) -> bool {
//         self.closed.load(Relaxed)
//     }
//     fn close(&self, _flag: usize) {
//         self.closed.store(true, SeqCst);
//     }
// }
//
// impl<T, M: Multiplicity, C: Capacity, const UNBOUNDED_BACKOFF: bool, K: Kind> HalfState<T>
//     for ArrayState<T, M, C, UNBOUNDED_BACKOFF, K, true>
// {
//     type Waiter = M::Waiter;
//     type Slot = (NonNull<Slot<T, UNBOUNDED_BACKOFF, K>>, usize);
//     type StateValue = M::StateValue;
//     const CONCURRENT: bool = M::CONCURRENT;
//
//     fn acquire_slot(&self) -> Result<Self::Slot, Self::StateValue> {
//         let mut tail = self.index.load();
//         if let Some(slot) = self.slot(tail)
//             && slot.can_write(tail)
//         {
//             let next_tail = self.next_index(tail);
//             match self.index.cas(tail, next_tail) {
//                 Ok(_) => return Ok((slot.into(), tail)),
//                 Err(t) => tail = t,
//             }
//         }
//         Err(M::state(tail))
//     }
//
//     fn acquire_slot_cold(&self, tail: Self::StateValue) -> Result<Self::Slot, TryAcquireError> {
//         let mut tail = M::index(tail).unwrap_or_else(|| self.index.load());
//         #[cfg(not(loom))]
//         let backoff = crossbeam_utils::Backoff::new();
//         let slot = loop {
//             let Some(slot) = self.slot(tail) else {
//                 return Err(TryAcquireError::Closed);
//             };
//             if !slot.can_write(tail) {
//                 if UNBOUNDED_BACKOFF {
//                     todo!()
//                 }
//                 return Err(TryAcquireError::Unavailable);
//             }
//             #[cfg(not(loom))]
//             backoff.spin();
//             let new_tail = self.next_index(tail);
//             match self.index.cas(tail, new_tail) {
//                 Ok(_) => break slot,
//                 Err(t) => tail = t,
//             }
//         };
//         Ok((slot.into(), tail))
//     }
//
//     fn capacity(&self) -> Option<usize> {
//         Some(self.capacity())
//     }
//
//     fn is_closed(&self) -> bool {
//         self.index.is_closed(self.closed_flag())
//     }
//
//     fn close(&self) {
//         self.index.close(self.closed_flag());
//     }
// }
//
// impl<T, M: Multiplicity, C: Capacity, const UNBOUNDED_BACKOFF: bool, K: Kind> TxState<T>
//     for ArrayState<T, M, C, UNBOUNDED_BACKOFF, K, true>
// {
//     fn write_slot(&self, (mut slot, tail): Self::Slot, msg: T) -> Result<(), SendError<T>> {
//         unsafe { slot.as_mut().write(msg, tail) };
//         if !M::CONCURRENT && self.is_closed() {}
//         Ok(())
//     }
// }
//
// impl<T, M: Multiplicity, C: Capacity, const UNBOUNDED_BACKOFF: bool, K: Kind> HalfState<T>
//     for ArrayState<T, M, C, UNBOUNDED_BACKOFF, K, false>
// {
//     type Waiter = M::Waiter;
//     type Slot = (NonNull<Slot<T, UNBOUNDED_BACKOFF, K>>, usize);
//     type StateValue = M::StateValue;
//     const CONCURRENT: bool = M::CONCURRENT;
//
//     fn acquire_slot(&self) -> Result<Self::Slot, Self::StateValue> {
//         let mut head = self.index.load();
//         if let Some(slot) = self.slot(head)
//             && slot.can_read(head)
//         {
//             let next_tail = self.next_index(head);
//             match self.index.cas(head, next_tail) {
//                 Ok(_) => return Ok((slot.into(), head)),
//                 Err(t) => head = t,
//             }
//         }
//         Err(M::state(head))
//     }
//
//     fn acquire_slot_cold(&self, tail: Self::StateValue) -> Result<Self::Slot, TryAcquireError> {
//         todo!()
//     }
//
//     fn capacity(&self) -> Option<usize> {
//         Some(self.capacity())
//     }
//
//     fn is_closed(&self) -> bool {
//         self.index.is_closed(self.closed_flag())
//     }
//
//     fn close(&self) {
//         self.index.close(self.closed_flag());
//     }
// }
//
// impl<T, M: Multiplicity, C: Capacity, const UNBOUNDED_BACKOFF: bool, K: Kind> RxState<T>
//     for ArrayState<T, M, C, UNBOUNDED_BACKOFF, K, false>
// {
//     fn read_slot(&self, slot: Self::Slot) -> T {
//         todo!()
//     }
// }
