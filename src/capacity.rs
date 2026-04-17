use crate::private;

pub trait Capacity: private::Capacity {}

impl Capacity for usize {}

pub struct ConstCapacity<const N: usize>;

impl<const N: usize> Capacity for ConstCapacity<N> {}
