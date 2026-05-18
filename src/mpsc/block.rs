use core::marker::PhantomData;

use crate::{
    DEFAULT_UNBOUNDED_BACKOFF,
    capacity::Capacity,
    sync::{DefaultSyncPrimitives, SyncPrimitives},
};

/// Bounded channel implementation fragmented in blocks.
///
/// Blocks are freed at once by receiver, so the exact capacity of the channel at one instant has a
/// lower bound to `capacity - BLOCK_SIZE`.
///
/// Contrary to regular channels, `send` operation use fetch-and-add instead of compare-and-swap
/// for every block slots except the last one.
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
    /// Constructs a new `BlockArray` with the specified capacity.
    pub fn new(capacity: C) -> Self {
        Self {
            capacity,
            sync: PhantomData,
        }
    }
}
