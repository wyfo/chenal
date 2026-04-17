use core::{
    mem::ManuallyDrop,
    ops::Deref,
    sync::atomic::Ordering::{AcqRel, Acquire, Relaxed},
};

use crate::channel::ChannelHalf;

pub struct Weak<H: ChannelHalf + Clone>(pub(crate) ManuallyDrop<H>);

impl<H: ChannelHalf + Clone> Deref for Weak<H> {
    type Target = H;

    fn deref(&self) -> &H {
        &self.0
    }
}

impl<H: ChannelHalf + Clone> Weak<H> {
    pub(crate) fn new(end: &H) -> Self {
        let weak = end.chan().weak_ref_cnt(H::HALF).unwrap();
        weak.fetch_add(1, Relaxed);
        Weak(ManuallyDrop::new(end.raw_clone()))
    }

    pub fn upgrade(&self) -> Option<H> {
        let incr = |c| (c != 0).then(|| c + 1);
        let strong = self.0.chan().ref_cnt(H::HALF).unwrap();
        strong.try_update(Relaxed, Acquire, incr).ok()?;
        Some(self.0.raw_clone())
    }
}

impl<H: ChannelHalf + Clone> Clone for Weak<H> {
    fn clone(&self) -> Self {
        let weak = self.0.chan().weak_ref_cnt(H::HALF).unwrap();
        weak.fetch_add(1, Relaxed);
        Self(ManuallyDrop::new(self.0.raw_clone()))
    }
}

impl<H: ChannelHalf + Clone> Drop for Weak<H> {
    fn drop(&mut self) {
        let weak = self.0.chan().weak_ref_cnt(H::HALF).unwrap();
        weak.fetch_sub(1, AcqRel);
    }
}
