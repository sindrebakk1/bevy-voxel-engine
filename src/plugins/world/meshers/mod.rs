use bevy::prelude::*;

use crate::plugins::world::{blocks::BlockRegistry, chunk::Chunk};

pub mod naive_mesher;

#[derive(Copy, Clone, Default)]
pub struct Neighbors<'a>([Option<&'a Chunk>; 6]);

#[repr(u8)]
pub enum Neighbour {
    X = 0,
    NegX = 1,
    Y = 2,
    NegY = 3,
    Z = 4,
    NegZ = 5,
}

impl Neighbour {
    pub const ALL: [Self; 6] = [
        Self::X,
        Self::NegX,
        Self::Y,
        Self::NegY,
        Self::Z,
        Self::NegZ,
    ];
    pub fn normal(&self) -> IVec3 {
        match *self {
            Self::X => IVec3::X,
            Self::NegX => IVec3::NEG_X,
            Self::Y => IVec3::Y,
            Self::NegY => IVec3::NEG_Y,
            Self::Z => IVec3::Z,
            Self::NegZ => IVec3::NEG_Z,
        }
    }
    pub fn from_normal(normal: IVec3) -> Self {
        match normal {
            IVec3::X => Self::X,
            IVec3::NEG_X => Self::NegX,
            IVec3::Y => Self::Y,
            IVec3::NEG_Y => Self::NegY,
            IVec3::Z => Self::Z,
            IVec3::NEG_Z => Self::NegZ,
            _ => panic!("invalid normal {}", normal),
        }
    }
}

impl<'a> Neighbors<'a> {
    pub fn from_array(array: [Option<&'a Chunk>; 6]) -> Self {
        Self(array)
    }

    pub fn get(&self, neighbour: Neighbour) -> Option<&'a Chunk> {
        self.0[neighbour as usize]
    }

    pub fn get_from_normal(&self, normal: IVec3) -> Option<&'a Chunk> {
        self.get(Neighbour::from_normal(normal))
    }
}

pub trait ChunkMesher: Send + Sync + 'static {
    fn build_mesh(&self, chunk: &Chunk, neighbours: Neighbors, registry: &BlockRegistry) -> Mesh;
}

pub use naive_mesher::NaiveMesher;
