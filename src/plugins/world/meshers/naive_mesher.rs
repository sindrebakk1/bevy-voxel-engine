use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use crate::plugins::world::{
    chunk::{Chunk, Voxel, CHUNK_SIZE},
    meshers::ChunkMesher,
};

struct VoxelMeshBuilder {
    positions: Vec<Vec3>,
    normals: Vec<Vec3>,
    indices: Vec<u32>,
}

impl VoxelMeshBuilder {
    fn new() -> Self {
        Self {
            positions: Vec::new(),
            normals: Vec::new(),
            indices: Vec::new(),
        }
    }

    #[inline]
    fn add_quad(&mut self, verts: [Vec3; 4], normal: Vec3) {
        let base = self.positions.len() as u32;

        self.positions.extend_from_slice(&verts);
        self.normals.extend_from_slice(&[normal; 4]);

        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    fn build(self) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, Default::default());

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_indices(Indices::U32(self.indices));

        mesh
    }
}

#[derive(Copy, Clone)]
struct Face {
    pub normal: Vec3,
    pub neighbor_offset: IVec3,
    pub vertices: [Vec3; 4],
}

const FACES: [Face; 6] = [
    // +X
    Face {
        normal: Vec3::X,
        neighbor_offset: IVec3::X,
        vertices: [
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(1.0, 0.0, 1.0),
        ],
    },
    // -X
    Face {
        normal: Vec3::NEG_X,
        neighbor_offset: IVec3::NEG_X,
        vertices: [
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 1.0, 1.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
        ],
    },
    // +Y
    Face {
        normal: Vec3::Y,
        neighbor_offset: IVec3::Y,
        vertices: [
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 1.0, 1.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(1.0, 1.0, 0.0),
        ],
    },
    // -Y
    Face {
        normal: Vec3::NEG_Y,
        neighbor_offset: IVec3::NEG_Y,
        vertices: [
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 1.0),
        ],
    },
    // +Z
    Face {
        normal: Vec3::Z,
        neighbor_offset: IVec3::Z,
        vertices: [
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 1.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(0.0, 1.0, 1.0),
        ],
    },
    // -Z
    Face {
        normal: Vec3::NEG_Z,
        neighbor_offset: IVec3::NEG_Z,
        vertices: [
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
        ],
    },
];

pub struct NaiveMesher;

impl ChunkMesher for NaiveMesher {
    fn build_mesh(&self, chunk: &Chunk) -> Mesh {
        let mut builder = VoxelMeshBuilder::new();

        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    if chunk.get(x, y, z) == Voxel::Air {
                        continue;
                    }

                    let base = Vec3::new(x as f32, y as f32, z as f32);

                    for face in &FACES {
                        let n = face.neighbor_offset;
                        let nx = x as i32 + n.x;
                        let ny = y as i32 + n.y;
                        let nz = z as i32 + n.z;

                        let visible = nx < 0
                            || ny < 0
                            || nz < 0
                            || nx >= CHUNK_SIZE as i32
                            || ny >= CHUNK_SIZE as i32
                            || nz >= CHUNK_SIZE as i32
                            || chunk.get(nx as usize, ny as usize, nz as usize) == Voxel::Air;

                        if !visible {
                            continue;
                        }

                        let verts = face.vertices.map(|v| base + v);
                        builder.add_quad(verts, face.normal);
                    }
                }
            }
        }

        builder.build()
    }
}
