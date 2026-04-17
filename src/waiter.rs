use core::{
    hint::assert_unchecked,
    mem,
    mem::ManuallyDrop,
    ops::Deref,
    pin::Pin,
    task::{Context, Waker},
};

use aiq::{WaitQueue, sync::SyncPrimitives, wait_queue::Wait};
use spmc_waker::SpmcWaker;

pub(crate) trait Waiter: Default + 'static {
    type Wait<'a>: Default
    where
        Self: 'a;
    unsafe fn register<'a>(&'a self, wait: Pin<&mut Self::Wait<'a>>, waker: &Waker) -> bool;
    unsafe fn unregister(&self);
    fn wake(&self);
    fn close(&self);
}

impl Waiter for SpmcWaker<false> {
    type Wait<'a> = ();
    unsafe fn register<'a>(&'a self, _wait: Pin<&mut Self::Wait<'a>>, waker: &Waker) -> bool {
        unsafe { self.register(waker) }
    }
    unsafe fn unregister(&self) {
        unsafe { self.unregister() };
    }
    fn wake(&self) {
        self.wake_cold();
    }
    fn close(&self) {
        self.wake();
    }
}

impl<SP: SyncPrimitives> Waiter for WaitQueue<SP> {
    type Wait<'a> = OptionCold<Wait<&'a WaitQueue<SP>, SP>>;
    unsafe fn register<'a>(&'a self, mut wait: Pin<&mut Self::Wait<'a>>, waker: &Waker) -> bool {
        if wait.is_none() {
            wait.set(Some(self.wait()).into());
        }
        let some_wait = wait.as_mut().as_pin_mut().unwrap();
        if some_wait.poll(&mut Context::from_waker(waker)).is_ready() {
            wait.set(None.into());
            return false;
        }
        true
    }
    unsafe fn unregister(&self) {}
    fn wake(&self) {
        self.notify_one();
    }
    fn close(&self) {
        self.close();
    }
}

pub(crate) struct OptionCold<T>(ManuallyDrop<Option<T>>);
impl<T> OptionCold<T> {
    fn as_pin_mut(self: Pin<&mut Self>) -> Option<Pin<&mut T>> {
        unsafe { self.map_unchecked_mut(|this| &mut *this.0) }.as_pin_mut()
    }
}
impl<T> Deref for OptionCold<T> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> From<Option<T>> for OptionCold<T> {
    fn from(value: Option<T>) -> Self {
        Self(ManuallyDrop::new(value))
    }
}
impl<T> Default for OptionCold<T> {
    fn default() -> Self {
        None.into()
    }
}
impl<T> Drop for OptionCold<T> {
    #[inline(always)]
    fn drop(&mut self) {
        if mem::needs_drop::<T>() && self.0.is_some() {
            #[cold]
            fn drop_cold<T>(this: &mut OptionCold<T>) {
                unsafe { assert_unchecked(this.0.is_some()) };
                unsafe { ManuallyDrop::drop(&mut this.0) };
            }
            drop_cold(self);
        }
    }
}
