use bevy::prelude::*;

use crate::plugins::world::material::VoxelAtlasMaterial;

#[derive(Resource)]
pub struct GameAssets {
    pub block_atlas: Handle<Image>,
}

#[derive(Resource)]
pub struct VoxelAtlasHandles {
    pub material: Handle<VoxelAtlasMaterial>,
}
