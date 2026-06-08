#[cfg(feature = "std")]
extern crate std;

use core::mem::MaybeUninit;
#[cfg(not(loom))]
pub(crate) use core::*;
#[cfg(not(loom))]
#[cfg(feature = "std")]
pub(crate) use std::thread;
#[cfg(not(loom))]
#[cfg(feature = "blocking")]
pub(crate) use std::thread_local;

#[cfg(loom)]
pub(crate) use loom::*;

pub(crate) mod sync {
    #[cfg(loom)]
    pub(crate) use loom::sync::*;

    #[cfg(all(miri, feature = "std"))]
    pub(crate) use super::std::sync::*;
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

            pub(crate) fn compare_exchange(
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

            pub(crate) fn compare_exchange_weak(
                &self,
                current: usize,
                new: usize,
                success: Ordering,
                failure: Ordering,
            ) -> Result<usize, usize> {
                self.compare_exchange(current, new, success, failure)
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

#[cfg(not(racy_test))]
pub(crate) struct RacyCell<T>(core::cell::UnsafeCell<MaybeUninit<T>>);
#[cfg(racy_test)]
pub(crate) struct RacyCell<T> {
    atomic: sync::atomic::AtomicPtr<()>,
    _phantom: core::marker::PhantomData<T>,
}

impl<T> RacyCell<T> {
    #[cfg(not(racy_test))]
    pub(crate) fn new() -> Self {
        Self(core::cell::UnsafeCell::new(MaybeUninit::uninit()))
    }

    #[cfg(racy_test)]
    pub(crate) fn new() -> Self {
        const { assert!(size_of::<T>() == size_of::<*mut ()>()) };
        Self {
            atomic: Default::default(),
            _phantom: Default::default(),
        }
    }

    #[cfg(not(racy_test))]
    #[inline(always)]
    pub(crate) unsafe fn write_racy(&self, t: T) {
        unsafe { (*self.0.get()).write(t) };
    }

    #[cfg(racy_test)]
    pub(crate) unsafe fn write_racy(&self, t: T) {
        let t = core::mem::ManuallyDrop::new(t);
        let ptr = core::ptr::from_ref(&t).cast::<*mut ()>();
        (self.atomic).store(unsafe { ptr.read() }, sync::atomic::Ordering::Relaxed);
    }

    #[cfg(not(racy_test))]
    #[inline(always)]
    pub(crate) unsafe fn read_racy(&self) -> MaybeUninit<T> {
        let msg = unsafe { self.0.get().cast::<MaybeUninit<T>>().read_volatile() };
        sync::atomic::fence(sync::atomic::Ordering::Acquire);
        msg
    }

    #[cfg(racy_test)]
    pub(crate) unsafe fn read_racy(&self) -> MaybeUninit<T> {
        let mut res = MaybeUninit::uninit();
        let msg = self.atomic.load(sync::atomic::Ordering::Relaxed);
        unsafe { core::ptr::from_mut(&mut res).cast::<*mut ()>().write(msg) };
        res
    }
}

pub(crate) trait AtomicUsizeExt {
    fn load_mut(&mut self) -> usize;
    fn store_seq_cst(&self, x: usize);
}

impl AtomicUsizeExt for sync::atomic::AtomicUsize {
    fn load_mut(&mut self) -> usize {
        #[cfg(not(loom))]
        return *self.get_mut();
        #[cfg(loom)]
        return self.with_mut(|ptr| *ptr);
    }
    #[inline(always)]
    fn store_seq_cst(&self, x: usize) {
        use sync::atomic::Ordering::*;
        // Splitting the store into a store + fence gives better result on
        // aarch64 benchmarks when it concerns channel states (not slots).
        // This split doesn't impact correctness of the algorithm as it
        // relies on `store X; load Y || store Y; load X` pattern, which
        // is still correct when atomic operations are relaxed with a
        // SeqCst fence in between.
        if cfg!(target_arch = "aarch64") {
            self.store(x, Relaxed);
            sync::atomic::fence(SeqCst);
        } else {
            self.store(x, SeqCst);
        }
    }
}
