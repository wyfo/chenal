#[cfg(feature = "std")]
extern crate std;

use core::mem::MaybeUninit;
#[cfg(not(loom))]
pub(crate) use core::*;
#[cfg(not(loom))]
#[cfg(feature = "blocking")]
pub(crate) use std::thread;

#[cfg(loom)]
pub(crate) use loom::*;

pub(crate) mod sync {
    #[cfg(not(loom))]
    pub(crate) use alloc::sync::*;

    #[cfg(loom)]
    pub(crate) use loom::sync::*;
    pub(crate) mod atomic {
        #[cfg(not(loom))]
        pub(crate) use core::sync::atomic::*;

        #[cfg(loom)]
        pub(crate) use loom::sync::atomic::*;

        #[cfg(loom)]
        fn seqcst_fence(order: Ordering) {
            if order == loom::sync::atomic::Ordering::SeqCst {
                loom::sync::atomic::fence(order);
            }
        }

        #[cfg(loom)]
        #[derive(Debug, Default)]
        pub struct AtomicUsize(loom::sync::atomic::AtomicUsize);

        #[cfg(loom)]
        impl AtomicUsize {
            pub(crate) fn new(x: usize) -> Self {
                Self(loom::sync::atomic::AtomicUsize::new(x))
            }

            pub(crate) fn with_mut<R>(&mut self, f: impl FnOnce(&mut usize) -> R) -> R {
                self.0.with_mut(|x| f(x))
            }

            pub(crate) fn load(&self, order: Ordering) -> usize {
                seqcst_fence(order);
                self.0.load(order)
            }

            pub(crate) fn store(&self, x: usize, order: Ordering) {
                self.0.store(x, order);
                seqcst_fence(order);
            }

            pub(crate) fn swap(&self, x: usize, order: Ordering) -> usize {
                seqcst_fence(order);
                let res = self.0.swap(x, order);
                seqcst_fence(order);
                res
            }

            pub(crate) fn compare_exchange_weak(
                &self,
                current: usize,
                new: usize,
                success: Ordering,
                failure: Ordering,
            ) -> Result<usize, usize> {
                seqcst_fence(success);
                let res = self.0.compare_exchange_weak(current, new, success, failure);
                if res.is_ok() {
                    seqcst_fence(success);
                }
                res
            }

            pub(crate) fn fetch_add(&self, val: usize, order: Ordering) -> usize {
                seqcst_fence(order);
                let res = self.0.fetch_add(val, order);
                seqcst_fence(order);
                res
            }

            pub(crate) fn fetch_sub(&self, val: usize, order: Ordering) -> usize {
                seqcst_fence(order);
                let res = self.0.fetch_sub(val, order);
                seqcst_fence(order);
                res
            }

            pub(crate) fn fetch_or(&self, val: usize, order: Ordering) -> usize {
                seqcst_fence(order);
                let res = self.0.fetch_or(val, order);
                seqcst_fence(order);
                res
            }

            pub fn fetch_update<F>(
                &self,
                set_order: Ordering,
                fetch_order: Ordering,
                f: F,
            ) -> Result<usize, usize>
            where
                F: FnMut(usize) -> Option<usize>,
            {
                seqcst_fence(fetch_order);
                let res = self.0.fetch_update(set_order, fetch_order, f);
                if res.is_ok() {
                    seqcst_fence(set_order);
                };
                res
            }
        }

        #[cfg(loom)]
        #[derive(Debug, Default)]
        pub struct AtomicBool(loom::sync::atomic::AtomicBool);

        #[cfg(loom)]
        impl AtomicBool {
            pub(crate) fn new(x: bool) -> Self {
                Self(loom::sync::atomic::AtomicBool::new(x))
            }

            pub(crate) fn load(&self, order: Ordering) -> bool {
                seqcst_fence(order);
                self.0.load(order)
            }

            pub(crate) fn store(&self, x: bool, order: Ordering) {
                self.0.store(x, order);
                seqcst_fence(order);
            }
        }
    }
}

pub(crate) trait UnsafeCellExt<T> {
    /// # Safety
    ///
    /// Cell content must be safe to deref immutably.
    unsafe fn with_ref<'a, R, F: FnOnce(&'a T) -> R>(&'a self, f: F) -> R
    where
        T: 'a;

    /// # Safety
    ///
    /// Cell content must be safe to deref mutably.
    unsafe fn with_ref_mut<'a, R, F: FnOnce(&'a mut T) -> R>(&'a self, f: F) -> R
    where
        T: 'a;
}

impl<T> UnsafeCellExt<T> for cell::UnsafeCell<T> {
    #[cfg_attr(loom, track_caller)]
    unsafe fn with_ref<'a, R, F: FnOnce(&'a T) -> R>(&'a self, f: F) -> R
    where
        T: 'a,
    {
        #[cfg(not(loom))]
        return f(unsafe { &*self.get() });
        #[cfg(loom)]
        return self.with(|ptr| f(unsafe { &*ptr }));
    }
    #[cfg_attr(loom, track_caller)]
    unsafe fn with_ref_mut<'a, R, F: FnOnce(&'a mut T) -> R>(&'a self, f: F) -> R
    where
        T: 'a,
    {
        #[cfg(not(loom))]
        return f(unsafe { &mut *self.get() });
        #[cfg(loom)]
        return self.with_mut(|ptr| f(unsafe { &mut *ptr }));
    }
}

pub(crate) trait RacyUnsafeCellExt<T> {
    fn new_racy() -> Self;
    unsafe fn write_racy(&self, t: T);
    unsafe fn read_racy(&self) -> MaybeUninit<T>;
}

impl<T> RacyUnsafeCellExt<T> for core::cell::UnsafeCell<MaybeUninit<T>> {
    #[cfg(not(miri_racy))]
    fn new_racy() -> Self {
        Self::new(MaybeUninit::uninit())
    }

    #[cfg(miri_racy)]
    fn new_racy() -> Self {
        const { assert!(size_of::<T>() == size_of::<*mut ()>()) };
        let mut t = MaybeUninit::uninit();
        let ptr = ptr::from_mut(&mut t).cast::<*mut ()>();
        unsafe { ptr.write(ptr::null_mut()) };
        Self::new(t)
    }

    #[cfg(not(miri_racy))]
    unsafe fn write_racy(&self, t: T) {
        use core::sync::atomic::{Ordering::Release, fence};
        fence(Release);
        unsafe { (*self.get()).write(t) };
    }

    #[cfg(miri_racy)]
    unsafe fn write_racy(&self, t: T) {
        use core::{
            mem::ManuallyDrop,
            sync::atomic::{AtomicPtr, Ordering::Relaxed},
        };
        let t = ManuallyDrop::new(t);
        let ptr = ptr::from_ref(&t).cast::<*mut ()>();
        unsafe { (*self.get().cast::<AtomicPtr<()>>()).store(ptr.read(), Relaxed) }
    }

    #[cfg(not(miri_racy))]
    unsafe fn read_racy(&self) -> MaybeUninit<T> {
        use core::sync::atomic::{Ordering::Acquire, fence};
        let msg = unsafe { self.get().cast::<MaybeUninit<T>>().read_volatile() };
        fence(Acquire);
        msg
    }

    #[cfg(miri_racy)]
    unsafe fn read_racy(&self) -> MaybeUninit<T> {
        use core::sync::atomic::{AtomicPtr, Ordering::Relaxed};
        let mut res = MaybeUninit::uninit();
        let msg = unsafe { (*self.get().cast::<AtomicPtr<()>>()).load(Relaxed) };
        unsafe { ptr::from_mut(&mut res).cast::<*mut ()>().write(msg) };
        res
    }
}

pub(crate) trait AtomicUsizeExt {
    fn load_mut(&mut self) -> usize;
}

impl AtomicUsizeExt for sync::atomic::AtomicUsize {
    fn load_mut(&mut self) -> usize {
        #[cfg(not(loom))]
        return *self.get_mut();
        #[cfg(loom)]
        return self.with_mut(|ptr| *ptr);
    }
}
