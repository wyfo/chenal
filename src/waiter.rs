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

pub(crate) trait Waiter<SP: SyncPrimitives>: Default + 'static {
    type Wait<'a>: Default + Send
    where
        Self: 'a;
    unsafe fn register<'a>(&'a self, wait: Pin<&mut Self::Wait<'a>>, waker: &Waker) -> bool;
    unsafe fn unregister(&self);
    fn close(&self);
}

impl<SP: SyncPrimitives> Waiter<SP> for SpmcWaker {
    type Wait<'a> = ();
    #[inline(always)]
    unsafe fn register<'a>(&'a self, _wait: Pin<&mut Self::Wait<'a>>, waker: &Waker) -> bool {
        unsafe { self.register(waker) }
    }
    #[inline(always)]
    unsafe fn unregister(&self) {
        unsafe { self.unregister() };
    }
    fn close(&self) {
        self.wake();
    }
}

impl<SP: SyncPrimitives> Waiter<SP> for WaitQueue<SP> {
    type Wait<'a> = OptionCold<Wait<&'a WaitQueue<SP>, SP>>;
    #[inline(always)]
    unsafe fn register<'a>(&'a self, mut wait: Pin<&mut Self::Wait<'a>>, waker: &Waker) -> bool {
        if wait.is_none() {
            wait.set(Some(self.wait()).into());
        }
        let some_wait = wait.as_mut().as_pin_mut().unwrap();
        let cx = &mut Context::from_waker(waker);
        some_wait.poll_wait(cx, true).is_pending()
    }
    unsafe fn unregister(&self) {}
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
