use crate::plugins::world::blocks::BlockId;

#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Default)]
pub struct Voxel(BlockId);

impl core::fmt::Debug for Voxel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Voxel")
            .field("solid", &self.is_solid())
            .field("block_id", &self.block_id())
            .finish()
    }
}

impl Voxel {
    pub const AIR: Self = Self::new(0);

    #[inline]
    pub const fn new(block_id: BlockId) -> Self {
        Self(block_id)
    }

    #[inline]
    pub const fn is_air(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub const fn is_solid(self) -> bool {
        self.0 != 0
    }

    #[inline]
    pub const fn block_id(self) -> BlockId {
        self.0
    }
}
