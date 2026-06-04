use crossfire::flavor::{Flavor, FlavorMC, FlavorMP};

use crate::{AsyncReceiver, AsyncSender, BlockingReceiver, BlockingSender, FutureExt as _, Receiver, Sender};

pub mod mpmc {
    pub use crossfire::mpmc::{
        bounded_async as async_channel, bounded_blocking as blocking_channel,
    };
}

pub mod mpsc {
    pub use crossfire::mpsc::{
        bounded_async as async_channel, bounded_blocking as blocking_channel,
    };
}

pub mod spsc {
    pub use crossfire::spsc::{
        bounded_async as async_channel, bounded_blocking as blocking_channel,
    };
}

impl<T: Send + 'static, F: Flavor<Item = T>> Sender<T> for crossfire::Tx<F> {
    const CLONEABLE: bool = false;
    fn try_send(&mut self, msg: T) {
        crossfire::BlockingTxTrait::try_send(self, msg).unwrap();
    }
    fn clone(&self) -> Self {
        unimplemented!()
    }
}

impl<T: Send + 'static, F: Flavor<Item = T>> BlockingSender<T> for crossfire::Tx<F> {
    fn send(&mut self, msg: T) {
        crossfire::BlockingTxTrait::send(self, msg).unwrap();
    }
}

impl<T: Send + Unpin + 'static, F: Flavor<Item = T>> Sender<T> for crossfire::AsyncTx<F> {
    const CLONEABLE: bool = false;
    fn try_send(&mut self, msg: T) {
        crossfire::AsyncTxTrait::try_send(self, msg).unwrap();
    }
    fn clone(&self) -> Self {
        unimplemented!()
    }
}

impl<T: Send + Unpin + 'static, F: Flavor<Item = T>> AsyncSender<T> for crossfire::AsyncTx<F> {
    fn send(&mut self, msg: T) -> impl Future<Output = ()> + Send + '_ {
        crossfire::AsyncTxTrait::send(self, msg).unwrap()
    }
}

impl<T: Send + 'static, F: Flavor<Item = T> + FlavorMP> Sender<T> for crossfire::MTx<F> {
    const CLONEABLE: bool = true;
    fn try_send(&mut self, msg: T) {
        crossfire::BlockingTxTrait::try_send(self, msg).unwrap();
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}

impl<T: Send + 'static, F: Flavor<Item = T> + FlavorMP> BlockingSender<T> for crossfire::MTx<F> {
    fn send(&mut self, msg: T) {
        crossfire::BlockingTxTrait::send(self, msg).unwrap();
    }
}

impl<T: Send + Unpin + 'static, F: Flavor<Item = T> + FlavorMP> Sender<T>
    for crossfire::MAsyncTx<F>
{
    const CLONEABLE: bool = true;
    fn try_send(&mut self, msg: T) {
        crossfire::AsyncTxTrait::try_send(self, msg).unwrap();
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}

impl<T: Send + Unpin + 'static, F: Flavor<Item = T> + FlavorMP> AsyncSender<T>
    for crossfire::MAsyncTx<F>
{
    fn send(&mut self, msg: T) -> impl Future<Output = ()> + Send + '_ {
        crossfire::AsyncTxTrait::send(self, msg).unwrap()
    }
}

impl<T: Send + 'static, F: Flavor<Item = T>> Receiver<T> for crossfire::Rx<F> {
    const CLONEABLE: bool = false;
    fn try_recv(&mut self) -> T {
        crossfire::BlockingRxTrait::try_recv(self).unwrap()
    }
    fn clone(&self) -> Self {
        unimplemented!()
    }
}

impl<T: Send + 'static, F: Flavor<Item = T>> BlockingReceiver<T> for crossfire::Rx<F> {
    fn recv(&mut self) -> T {
        crossfire::BlockingRxTrait::recv(self).unwrap()
    }
}

impl<T: Send + Unpin + 'static, F: Flavor<Item = T>> Receiver<T> for crossfire::AsyncRx<F> {
    const CLONEABLE: bool = false;
    fn try_recv(&mut self) -> T {
        crossfire::AsyncRxTrait::try_recv(self).unwrap()
    }
    fn clone(&self) -> Self {
        unimplemented!()
    }
}

impl<T: Send + Unpin + 'static, F: Flavor<Item = T>> AsyncReceiver<T> for crossfire::AsyncRx<F> {
    fn recv(&mut self) -> impl Future<Output = T> + Send + '_ {
        crossfire::AsyncRxTrait::recv(self).unwrap()
    }
}

impl<T: Send + 'static, F: Flavor<Item = T> + FlavorMC> Receiver<T> for crossfire::MRx<F> {
    const CLONEABLE: bool = true;
    fn try_recv(&mut self) -> T {
        crossfire::BlockingRxTrait::try_recv(self).unwrap()
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}

impl<T: Send + 'static, F: Flavor<Item = T> + FlavorMC> BlockingReceiver<T> for crossfire::MRx<F> {
    fn recv(&mut self) -> T {
        crossfire::BlockingRxTrait::recv(self).unwrap()
    }
}

impl<T: Send + Unpin + 'static, F: Flavor<Item = T> + FlavorMC> Receiver<T>
    for crossfire::MAsyncRx<F>
{
    const CLONEABLE: bool = true;
    fn try_recv(&mut self) -> T {
        crossfire::AsyncRxTrait::try_recv(self).unwrap()
    }
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}

impl<T: Send + Unpin + 'static, F: Flavor<Item = T> + FlavorMC> AsyncReceiver<T>
    for crossfire::MAsyncRx<F>
{
    fn recv(&mut self) -> impl Future<Output = T> + Send + '_ {
        crossfire::AsyncRxTrait::recv(self).unwrap()
    }
}
