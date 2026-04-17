use core::marker::PhantomData;

use crate::{
    DEFAULT_UNBOUNDED_BACKOFF,
    capacity::Capacity,
    sync::{DefaultSyncPrimitives, SyncPrimitives},
};

#[allow(dead_code)]
pub struct BlockArray<
    const BLOCK_SIZE: usize,
    C: Capacity = usize,
    const UNBOUNDED_BACKOFF: bool = DEFAULT_UNBOUNDED_BACKOFF,
    SP: SyncPrimitives = DefaultSyncPrimitives,
> {
    capacity: C,
    sync: PhantomData<SP>,
}

impl<const BLOCK_SIZE: usize, C: Capacity, const UNBOUNDED_BACKOFF: bool, SP: SyncPrimitives>
    BlockArray<BLOCK_SIZE, C, UNBOUNDED_BACKOFF, SP>
{
    pub fn new(capacity: C) -> Self {
        Self {
            capacity,
            sync: PhantomData,
        }
    }
}
