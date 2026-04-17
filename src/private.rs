use alloc::sync::Arc;

use crate::{
    capacity::ConstCapacity,
    channel::{Chan, Half},
    rx::RxState,
    tx::TxState,
};

pub trait Capacity {
    type Mask;
    fn get(&self) -> usize;
    fn mask(&self) -> Self::Mask;
    fn mask_get(mask: &Self::Mask) -> usize;
}

impl Capacity for usize {
    type Mask = usize;
    fn get(&self) -> usize {
        *self
    }
    fn mask(&self) -> Self::Mask {
        ((*self).next_power_of_two() << 1).wrapping_sub(1)
    }
    fn mask_get(mask: &Self::Mask) -> usize {
        *mask
    }
}

impl<const N: usize> Capacity for ConstCapacity<N> {
    type Mask = ();
    fn get(&self) -> usize {
        N
    }
    fn mask(&self) -> Self::Mask {}
    fn mask_get(_mask: &Self::Mask) -> usize {
        (N.next_power_of_two() << 1).wrapping_sub(1)
    }
}

pub(crate) trait Channel: 'static {
    type TxState<T>: TxState<T>;
    type RxState<T>: RxState<T>;
    fn storage() -> &'static str;
    fn drop_storage<T>(tx_state: &mut Self::TxState<T>, rx_state: &mut Self::RxState<T>);
}

pub(crate) trait ChannelHalf {
    type Msg;
    type Channel: Channel;
    const HALF: Half;
    fn new(chan: Arc<Chan<Self::Msg, Self::Channel>>) -> Self;
    fn chan(&self) -> &Arc<Chan<Self::Msg, Self::Channel>>;
    fn raw_clone(&self) -> Self
    where
        Self: Clone,
    {
        Self::new(self.chan().clone())
    }
}
