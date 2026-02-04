use bevy::mesh::{Indices, MeshVertexAttribute, PrimitiveTopology, VertexFormat};
use bevy::prelude::*;

use crate::plugins::world::{
    blocks::{BlockId, BlockRegistry, TileId},
    chunk::{CHUNK_SIZE, Chunk},
    meshers::ChunkMesher,
};

pub const ATTRIBUTE_TILE_ID: MeshVertexAttribute =
    MeshVertexAttribute::new("TileId", 0xBADC0DE1, VertexFormat::Uint32);

struct VoxelMeshBuilder {
    positions: Vec<Vec3>,
    normals: Vec<Vec3>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
    tile_ids: Vec<u32>,
}

impl VoxelMeshBuilder {
    fn new() -> Self {
        Self {
            positions: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            indices: Vec::new(),
            tile_ids: Vec::new(),
        }
    }

    #[inline]
    fn add_quad(&mut self, verts: [Vec3; 4], tile_id: TileId, normal: Vec3) {
        let base = self.positions.len() as u32;

        self.positions.extend_from_slice(&verts);

        self.normals.extend_from_slice(&[normal; 4]);

        let uvs = face_uvs(normal);
        self.uvs.extend_from_slice(&uvs);

        self.tile_ids.extend_from_slice(&[tile_id as u32; 4]);

        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    fn build(self) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, Default::default());

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
        mesh.insert_attribute(ATTRIBUTE_TILE_ID, self.tile_ids);
        mesh.insert_indices(Indices::U32(self.indices));

        mesh
    }
}

fn face_uvs(normal: Vec3) -> [[f32; 2]; 4] {
    if normal == Vec3::X {
        [[0., 1.], [0., 0.], [1., 0.], [1., 1.]]
    } else if normal == Vec3::NEG_X {
        [[1., 1.], [1., 0.], [0., 0.], [0., 1.]]
    } else if normal == Vec3::Z {
        [[0., 1.], [1., 1.], [1., 0.], [0., 0.]]
    } else if normal == Vec3::NEG_Z {
        [[1., 1.], [0., 1.], [0., 0.], [1., 0.]]
    } else {
        [[0., 1.], [1., 1.], [1., 0.], [0., 0.]]
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

#[derive(Copy, Clone)]
pub enum FaceKind {
    Top,
    Bottom,
    Side,
}

#[inline]
pub fn face_kind_from_normal(n: Vec3) -> FaceKind {
    if n == Vec3::Y {
        FaceKind::Top
    } else if n == Vec3::NEG_Y {
        FaceKind::Bottom
    } else {
        FaceKind::Side
    }
}

pub struct TileResolver<'a> {
    pub registry: &'a BlockRegistry,
}

impl<'a> TileResolver<'a> {
    #[inline]
    pub fn resolve(
        &self,
        block_id: BlockId,
        face: FaceKind,
        // room for neighbor-based rules later:
        // world_pos: IVec3,
        // chunk: &Chunk,
    ) -> TileId {
        let t = self.registry.tiles(block_id);
        match face {
            FaceKind::Top => t.top,
            FaceKind::Bottom => t.bottom,
            FaceKind::Side => t.side,
        }
    }
}

pub struct NaiveMesher;

impl ChunkMesher for NaiveMesher {
    fn build_mesh(&self, chunk: &Chunk, registry: &BlockRegistry) -> Mesh {
        let resolver = TileResolver { registry };
        let mut builder = VoxelMeshBuilder::new();

        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let voxel = chunk.get(x, y, z);
                    if voxel.is_air() {
                        continue;
                    }

                    let base = Vec3::new(x as f32, y as f32, z as f32);

                    for face in &FACES {
                        let n = face.neighbor_offset;
                        let nx = x as i32 + n.x;
                        let ny = y as i32 + n.y;
                        let nz = z as i32 + n.z;

                        let outside = nx < 0
                            || ny < 0
                            || nz < 0
                            || nx >= CHUNK_SIZE as i32
                            || ny >= CHUNK_SIZE as i32
                            || nz >= CHUNK_SIZE as i32;

                        let neighbor_is_air = if outside {
                            true
                        } else {
                            chunk.get(nx as usize, ny as usize, nz as usize).is_air()
                        };

                        if !neighbor_is_air {
                            continue;
                        }

                        let face_kind = face_kind_from_normal(face.normal);
                        let tile_id = resolver.resolve(voxel.block_id(), face_kind);

                        let verts = face.vertices.map(|v| base + v);
                        builder.add_quad(verts, tile_id, face.normal);
                    }
                }
            }
        }

        builder.build()
    }
}
