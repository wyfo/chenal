//! Bounded channel capacity abstraction.
use crate::internal;

/// The capacity of a bounded channel.
///
/// Either a runtime `usize` or a compile-time [`ConstCapacity<N>`].
#[expect(private_bounds)]
pub trait Capacity: internal::Capacity {}

impl Capacity for usize {}

/// A compile-time capacity, enabling more aggressive optimizations.
#[derive(Clone, Copy)]
pub struct ConstCapacity<const N: usize>;

impl<const N: usize> Capacity for ConstCapacity<N> {}
