use bevy::prelude::*;

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub enum Voxel {
    #[default]
    Air = 0,
    Solid = 1,
}

#[derive(Clone, Debug)]
pub struct Chunk {
    voxels: Box<[Voxel; CHUNK_VOLUME]>,
    dirty: bool,
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            voxels: Box::new([Voxel::default(); CHUNK_VOLUME]),
            dirty: false,
        }
    }
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            voxels: Box::new([Voxel::Air; CHUNK_VOLUME]),
            dirty: true,
        }
    }

    #[inline]
    pub fn index(x: usize, y: usize, z: usize) -> usize {
        debug_assert!(x < CHUNK_SIZE);
        debug_assert!(y < CHUNK_SIZE);
        debug_assert!(z < CHUNK_SIZE);
        x + CHUNK_SIZE * (y + CHUNK_SIZE * z)
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize, z: usize) -> Voxel {
        self.voxels[Self::index(x, y, z)]
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, z: usize, voxel: Voxel) {
        let idx = Self::index(x, y, z);
        self.voxels[idx] = voxel;
        self.dirty = true;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }
}
