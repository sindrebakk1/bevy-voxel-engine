use bevy::prelude::Mesh;

use crate::plugins::world::{blocks::BlockRegistry, chunk::Chunk};

pub mod naive_mesher;

pub trait ChunkMesher: Send + Sync + 'static {
    fn build_mesh(&self, chunk: &Chunk, registry: &BlockRegistry) -> Mesh;
}

pub use naive_mesher::NaiveMesher;
