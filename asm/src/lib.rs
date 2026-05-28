#![allow(unexpected_cfgs)]

#[allow(unused_imports)]
use std::{
    pin::pin,
    task::{Context, Poll},
};

#[allow(dead_code)]
type Msg = usize;

#[cfg(mpmc_recv)]
#[unsafe(no_mangle)]
pub fn mpmc_recv(
    rx: &chenal::mpmc::MRx<Msg>,
    cx: &mut Context,
) -> Poll<Result<Msg, chenal::errors::RecvError>> {
    pin!(rx.recv()).poll(cx)
}

#[cfg(mpmc_recv_blocking)]
#[unsafe(no_mangle)]
pub fn mpmc_recv_blocking(rx: &chenal::mpmc::MRx<Msg>) -> Result<Msg, chenal::errors::RecvError> {
    rx.recv_blocking()
}

#[cfg(mpmc_send)]
#[unsafe(no_mangle)]
pub fn mpmc_send(
    tx: &chenal::mpmc::MTx<Msg>,
    msg: Msg,
    cx: &mut Context,
) -> Poll<Result<(), chenal::errors::SendError<Msg>>> {
    pin!(tx.send(msg)).poll(cx)
}

#[cfg(mpmc_send_blocking)]
#[unsafe(no_mangle)]
pub fn mpmc_send_blocking(
    tx: &chenal::mpmc::MTx<Msg>,
    msg: Msg,
) -> Result<(), chenal::errors::SendError<Msg>> {
    tx.send_blocking(msg)
}

#[cfg(mpsc_recv)]
#[unsafe(no_mangle)]
pub fn mpsc_recv(
    rx: &mut chenal::mpsc::Rx<Msg>,
    cx: &mut Context,
) -> Poll<Result<Msg, chenal::errors::RecvError>> {
    pin!(rx.recv()).poll(cx)
}

#[cfg(mpsc_recv_blocking)]
#[unsafe(no_mangle)]
pub fn mpsc_recv_blocking(
    rx: &mut chenal::mpsc::Rx<Msg>,
) -> Result<Msg, chenal::errors::RecvError> {
    rx.recv_blocking()
}

#[cfg(mpsc_send)]
#[unsafe(no_mangle)]
pub fn mpsc_send(
    tx: &chenal::mpsc::MTx<Msg>,
    msg: Msg,
    cx: &mut Context,
) -> Poll<Result<(), chenal::errors::SendError<Msg>>> {
    pin!(tx.send(msg)).poll(cx)
}

#[cfg(mpsc_send_blocking)]
#[unsafe(no_mangle)]
pub fn mpsc_send_blocking(
    tx: &chenal::mpsc::MTx<Msg>,
    msg: Msg,
) -> Result<(), chenal::errors::SendError<Msg>> {
    tx.send_blocking(msg)
}

#[cfg(spmc_recv)]
#[unsafe(no_mangle)]
pub fn spmc_recv(
    rx: &chenal::spmc::MRx<Msg>,
    cx: &mut Context,
) -> Poll<Result<Msg, chenal::errors::RecvError>> {
    pin!(rx.recv()).poll(cx)
}

#[cfg(spmc_recv_blocking)]
#[unsafe(no_mangle)]
pub fn spmc_recv_blocking(
    rx: &chenal::spmc::MRx<Msg>,
) -> Result<Msg, chenal::errors::RecvError> {
    rx.recv_blocking()
}

#[cfg(spmc_send)]
#[unsafe(no_mangle)]
pub fn spmc_send(
    tx: &mut chenal::spmc::Tx<Msg>,
    msg: Msg,
    cx: &mut Context,
) -> Poll<Result<(), chenal::errors::SendError<Msg>>> {
    pin!(tx.send(msg)).poll(cx)
}

#[cfg(spmc_send_blocking)]
#[unsafe(no_mangle)]
pub fn spmc_send_blocking(
    tx: &mut chenal::spmc::Tx<Msg>,
    msg: Msg,
) -> Result<(), chenal::errors::SendError<Msg>> {
    tx.send_blocking(msg)
}

#[cfg(spsc_recv)]
#[unsafe(no_mangle)]
pub fn spsc_recv(
    rx: &mut chenal::spsc::Rx<Msg>,
    cx: &mut Context,
) -> Poll<Result<Msg, chenal::errors::RecvError>> {
    pin!(rx.recv()).poll(cx)
}

#[cfg(spsc_recv_blocking)]
#[unsafe(no_mangle)]
pub fn spsc_recv_blocking(
    rx: &mut chenal::spsc::Rx<Msg>,
) -> Result<Msg, chenal::errors::RecvError> {
    rx.recv_blocking()
}

#[cfg(spsc_send)]
#[unsafe(no_mangle)]
pub fn spsc_send(
    tx: &mut chenal::spsc::Tx<Msg>,
    msg: Msg,
    cx: &mut Context,
) -> Poll<Result<(), chenal::errors::SendError<Msg>>> {
    pin!(tx.send(msg)).poll(cx)
}

#[cfg(spsc_send_blocking)]
#[unsafe(no_mangle)]
pub fn spsc_send_blocking(
    tx: &mut chenal::spsc::Tx<Msg>,
    msg: Msg,
) -> Result<(), chenal::errors::SendError<Msg>> {
    tx.send_blocking(msg)
}
