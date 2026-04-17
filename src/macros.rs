macro_rules! channel_end {
    ($ty:ident) => {
        unsafe impl<T, Ch: crate::channel::Channel> Send for $ty<T, Ch> {}
        unsafe impl<T, Ch: crate::channel::Channel> Sync for $ty<T, Ch> {}
        impl<T, Ch: crate::channel::Channel> crate::channel::ChannelHalf for $ty<T, Ch> {}
        impl<T, Ch: crate::channel::Channel> crate::private::ChannelHalf for $ty<T, Ch> {
            type Msg = T;
            type Channel = Ch;
            const HALF: crate::channel::Half = crate::channel::Half::$ty;
            fn new(chan: crate::channel::ArcChan<Self::Msg, Self::Channel>) -> Self {
                Self { chan }
            }
            fn chan(&self) -> &crate::channel::ArcChan<Self::Msg, Self::Channel> {
                &self.chan
            }
        }
        impl<T, Ch: crate::channel::Channel> $ty<T, Ch> {
            pub fn is_closed(&self) -> bool {
                use crate::private::ChannelHalf;
                self.chan().is_closed(Self::HALF)
            }

            pub fn close(&self) {
                use crate::private::ChannelHalf;
                self.chan().close()
            }

            pub fn close_guard<'a>(&self) -> crate::channel::CloseGuard<'a> where T: 'a {
                use crate::private::ChannelHalf;
                crate::channel::CloseGuard::new(self.chan().clone())
            }

            pub fn channel_id(&self) -> crate::channel::ChannelId {
                use crate::private::ChannelHalf;
                self.chan().id()
            }
        }
        impl<T, Ch: crate::channel::Channel> Drop for $ty<T, Ch> {
            fn drop(&mut self) {
                use crate::private::ChannelHalf;
                self.chan().drop_half(Self::HALF);
            }
        }
        impl<T, Ch: crate::channel::Channel> core::fmt::Debug for $ty<T, Ch> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                use crate::private::ChannelHalf;
                f.debug_tuple(stringify!($ty)).field(self.chan()).finish()
            }
        }
        crate::macros::channel_end!(@ $ty clone($ty));
    };
    (@ $ty:ident clone(Tx)) => {};
    (@ $ty:ident clone(UTx)) => {};
    (@ $ty:ident clone(Rx)) => {};
    (@ $ty:ident clone(MTx)) => { crate::macros::channel_end!(@ $ty clone); };
    (@ $ty:ident clone(UMTx)) => { crate::macros::channel_end!(@ $ty clone); };
    (@ $ty:ident clone(MRx)) => { crate::macros::channel_end!(@ $ty clone); };
    (@ $ty:ident clone) => {
        impl<T, Ch: crate::channel::Channel> Clone for $ty<T, Ch> {
            fn clone(&self) -> Self {
                use crate::private::ChannelHalf;
                self.chan().clone_half(Self::HALF);
                self.raw_clone()
            }
        }

        impl<T, Ch: crate::channel::Channel> $ty<T, Ch> {
            #[cfg(feature = "weak")]
            pub fn downgrade(&self) -> crate::weak::Weak<Self> {
                crate::weak::Weak::new(self)
            }
        }
    };
}
pub(crate) use channel_end;
