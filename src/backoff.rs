//! Atomic compare-and-swap backoff abstraction.
use core::hint::spin_loop;

/// Backoff strategy in case of atomic compare-and-swap (CAS) failure.
pub trait BackoffStrategy: Send + Sync + 'static {
    /// Backoff state.
    type State: Default;
    /// Performs backoff and returns how the CAS should be retried.
    fn backoff(backoff: &mut Self::State) -> RetryStrategy;
}

/// Retry strategy of a failed atomic compare-and-swap (CAS).
#[derive(PartialEq)]
pub enum RetryStrategy {
    /// Retries without reloading the atomic value.
    ///
    /// Uses the value returned by the failed CAS.
    Retry,
    /// Reloads the atomic and retries the CAS with the updated value.
    ReloadAndRetry,
    /// Reloads the atomic and keep backing off while the value changes
    /// between reloads, then retries with the up-to-date value.
    ReloadAndRetryIfUnchanged,
}

/// No backoff, retry CAS straight away.
pub struct NoBackoff;

impl BackoffStrategy for NoBackoff {
    type State = ();
    fn backoff(_backoff: &mut Self::State) -> RetryStrategy {
        RetryStrategy::Retry
    }
}

/// Calls [`spin_loop`] and reload the atomic.
pub struct SpinReload;

impl BackoffStrategy for SpinReload {
    type State = ();
    fn backoff(_backoff: &mut Self::State) -> RetryStrategy {
        spin_loop();
        RetryStrategy::ReloadAndRetry
    }
}

/// Performs exponential backoff, as done by [`crossbeam_utils::Backoff`].
///
/// Each backoff iteration calls [`spin_loop`] `1 << iter` times, and increment iter until
/// it reaches `SPIN_LIMIT`. If CAS keeps failing and `YIELD` is true, then [`yield_now`]
/// is called instead of spinning.
///
/// If `LOOP` is true, then backoff continues until the atomic has not its value
/// updated during backoff.
///
/// [`yield_now`]: crate::blocking::std::thread::yield_now
pub struct ExponentialBackoff<const SPIN_LIMIT: usize, const YIELD: bool, const LOOP: bool>;

impl<const SPIN_LIMIT: usize, const YIELD: bool, const LOOP: bool> BackoffStrategy
    for ExponentialBackoff<SPIN_LIMIT, YIELD, LOOP>
{
    type State = usize;
    #[inline(always)]
    fn backoff(backoff: &mut Self::State) -> RetryStrategy {
        if *backoff >= SPIN_LIMIT && YIELD && cfg!(feature = "std") {
            #[cfg(feature = "std")]
            extern crate std;
            #[cfg(feature = "std")]
            std::thread::yield_now();
        } else {
            for _ in 0..1 << *backoff {
                spin_loop();
            }
            if *backoff < SPIN_LIMIT {
                *backoff += 1;
            }
        }
        if LOOP {
            RetryStrategy::ReloadAndRetryIfUnchanged
        } else {
            RetryStrategy::ReloadAndRetry
        }
    }
}

pub(crate) struct Backoff<B: BackoffStrategy> {
    state: B::State,
    backoff: bool,
}

impl<B: BackoffStrategy> Backoff<B> {
    pub(crate) fn new(backoff: bool) -> Self {
        Self {
            state: Default::default(),
            backoff,
        }
    }

    #[inline(always)]
    pub(crate) fn backoff<T: PartialEq>(
        &mut self,
        atomic: &mut T,
        reload: impl FnOnce() -> T,
    ) -> bool {
        if !self.backoff {
            self.backoff = true;
            return false;
        }
        let retry = B::backoff(&mut self.state);
        if retry == RetryStrategy::Retry {
            return false;
        }
        let reloaded = reload();
        if reloaded == *atomic {
            return false;
        }
        *atomic = reloaded;
        self.backoff = retry == RetryStrategy::ReloadAndRetryIfUnchanged;
        true
    }
}
