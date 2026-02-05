use bevy::platform::collections::HashMap;
use bevy::prelude::Resource;

pub type BlockId = u16;
pub type TileId = u16;

pub const BLOCK_GRASS: BlockId = 1;
pub const BLOCK_DIRT: BlockId = 2;
pub const BLOCK_STONE: BlockId = 3;

#[derive(Copy, Clone)]
pub struct BlockTiles {
    pub top: TileId,
    pub side: TileId,
    pub bottom: TileId,
}

impl BlockTiles {
    pub fn new(block_id: BlockId) -> Self {
        let idx_start = (block_id - 1) * 3;
        Self {
            top: idx_start,
            side: idx_start + 1,
            bottom: idx_start + 2,
        }
    }
}

pub struct BlockRegistry {
    blocks: HashMap<BlockId, BlockTiles>,
}

impl BlockRegistry {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            blocks: HashMap::with_capacity(capacity),
        }
    }

    #[inline]
    pub fn tiles(&self, id: BlockId) -> BlockTiles {
        self.blocks[&id]
    }

    #[inline]
    pub fn insert(&mut self, block_id: BlockId) -> Option<BlockTiles> {
        self.blocks.insert(block_id, BlockTiles::new(block_id))
    }
}

#[derive(Resource)]
pub struct BlockRegistryRes(pub BlockRegistry);

impl Default for BlockRegistryRes {
    fn default() -> Self {
        let mut registry = BlockRegistry::with_capacity(32);

        registry.insert(BLOCK_GRASS);
        registry.insert(BLOCK_DIRT);
        registry.insert(BLOCK_STONE);

        BlockRegistryRes(registry)
    }
}
