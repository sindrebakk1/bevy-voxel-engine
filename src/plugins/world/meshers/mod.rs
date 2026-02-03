use super::chunk::Chunk;
use bevy::prelude::Mesh;

pub mod naive_mesher;

pub trait ChunkMesher: Send + Sync + 'static {
    fn build_mesh(&self, chunk: &Chunk) -> Mesh;
}

pub use naive_mesher::NaiveMesher;
