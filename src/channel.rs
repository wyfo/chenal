use core::{
    fmt, mem,
    ptr::NonNull,
    sync::atomic::{
        AtomicUsize,
        Ordering::{AcqRel, Relaxed},
    },
};

use crossbeam_utils::CachePadded;

use crate::{loom::sync::Arc, private, rx::RxState, tx::TxState};

#[expect(private_bounds)]
pub trait Channel: private::Channel {}
#[expect(private_bounds)]
pub trait ChannelHalf: private::ChannelHalf {}

#[derive(Copy, Clone)]
pub(crate) enum Half {
    Tx,
    UTx,
    MTx,
    UMTx,
    Rx,
    MRx,
}

impl Half {
    pub(crate) const fn cloneable(self) -> bool {
        matches!(self, Self::MTx | Self::UMTx | Self::MRx)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ChannelId(NonNull<()>);

pub(crate) type ArcChan<T, Ch> = Arc<Chan<T, Ch>>;

pub(crate) struct Chan<T, Ch: private::Channel> {
    pub(crate) tx_state: CachePadded<Ch::TxState<T>>,
    pub(crate) rx_state: CachePadded<Ch::RxState<T>>,
    pub(crate) tx_cnt: AtomicUsize,
    pub(crate) rx_cnt: AtomicUsize,
    pub(crate) weak_tx_cnt: AtomicUsize,
    pub(crate) weak_rx_cnt: AtomicUsize,
}

impl<T, Ch: private::Channel> Chan<T, Ch> {
    pub(crate) fn id(&self) -> ChannelId {
        ChannelId(NonNull::from(self).cast())
    }

    pub(crate) fn is_closed(&self, side: Half) -> bool {
        match side {
            Half::Tx | Half::UTx | Half::MTx | Half::UMTx => self.tx_state.is_closed(),
            Half::Rx | Half::MRx => self.rx_state.is_closed(),
        }
    }

    pub(crate) fn close(&self) {
        self.tx_state.close();
        self.rx_state.close();
    }

    pub(crate) fn ref_cnt(&self, half: Half) -> &AtomicUsize {
        match half {
            Half::Tx | Half::UTx | Half::MTx | Half::UMTx => &self.tx_cnt,
            Half::Rx | Half::MRx => &self.rx_cnt,
        }
    }

    pub(crate) fn weak_ref_cnt(&self, half: Half) -> &AtomicUsize {
        match half {
            Half::Tx | Half::UTx | Half::MTx | Half::UMTx => &self.weak_tx_cnt,
            Half::Rx | Half::MRx => &self.weak_rx_cnt,
        }
    }

    pub(crate) fn drop_half(&self, half: Half) {
        if !half.cloneable() || self.ref_cnt(half).fetch_sub(1, AcqRel) == 1 {
            self.close();
        }
    }

    pub(crate) fn clone_half(&self, half: Half) {
        assert!(half.cloneable());
        self.ref_cnt(half).fetch_add(1, Relaxed);
    }
}

impl<T, Ch: private::Channel> Drop for Chan<T, Ch> {
    fn drop(&mut self) {
        Ch::drop_storage(&mut self.tx_state, &mut self.rx_state);
    }
}

impl<T, Ch: private::Channel> fmt::Debug for Chan<T, Ch> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut builder = f.debug_struct("Chan");
        builder
            .field("id", &self.id())
            .field("storage", &Ch::storage());
        if let Some(capacity) = self.tx_state.capacity() {
            builder.field("capacity", &capacity);
        }
        builder.finish_non_exhaustive()
    }
}

trait Close: fmt::Debug {
    fn id(&self) -> ChannelId;
    fn close(&self);
}

impl<T, Ch: private::Channel> Close for Chan<T, Ch> {
    fn id(&self) -> ChannelId {
        self.id()
    }
    fn close(&self) {
        self.close();
    }
}

#[derive(Clone)]
pub struct CloseGuard<'a>(Arc<dyn Close + 'a>);

impl<'a> CloseGuard<'a> {
    pub(crate) fn new<T: 'a, Ch: Channel>(chan: ArcChan<T, Ch>) -> Self {
        Self(chan as _)
    }

    pub fn forget(self) {
        mem::forget(self);
    }
}

impl Drop for CloseGuard<'_> {
    fn drop(&mut self) {
        self.0.close();
    }
}
