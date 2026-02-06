use bevy::mesh::{Indices, MeshVertexAttribute, PrimitiveTopology, VertexFormat};
use bevy::prelude::*;

use crate::plugins::world::{
    blocks::{BlockId, BlockRegistry, TileId},
    chunk::{CHUNK_SIZE, Chunk},
    meshers::{ChunkMesher, Neighbors},
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
    fn add_quad(&mut self, verts: [Vec3; 4], uvs: [[f32; 2]; 4], tile_id: TileId, normal: Vec3) {
        let base = self.positions.len() as u32;

        self.positions.extend_from_slice(&verts);

        self.normals.extend_from_slice(&[normal; 4]);

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

#[derive(Copy, Clone)]
struct Face {
    pub normal: Vec3,
    pub neighbor_offset: IVec3,
    pub vertices: [Vec3; 4],
    pub uvs: [[f32; 2]; 4],
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
        uvs: [[0., 1.], [0., 0.], [1., 0.], [1., 1.]],
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
        uvs: [[1., 1.], [1., 0.], [0., 0.], [0., 1.]],
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
        uvs: [[0., 1.], [1., 1.], [1., 0.], [0., 0.]],
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
        uvs: [[0., 1.], [1., 1.], [1., 0.], [0., 0.]],
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
        uvs: [[0., 1.], [1., 1.], [1., 0.], [0., 0.]],
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
        uvs: [[1., 1.], [0., 1.], [0., 0.], [1., 0.]],
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
    pub fn resolve(&self, block_id: BlockId, face: FaceKind) -> TileId {
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
    fn build_mesh(&self, chunk: &Chunk, neighbors: Neighbors, registry: &BlockRegistry) -> Mesh {
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
                        if !neighbor_is_air(chunk, &neighbors, x, y, z, face.neighbor_offset) {
                            continue;
                        }

                        let face_kind = face_kind_from_normal(face.normal);
                        let tile_id = resolver.resolve(voxel.block_id(), face_kind);

                        let verts = face.vertices.map(|v| base + v);
                        builder.add_quad(verts, face.uvs, tile_id, face.normal);
                    }
                }
            }
        }

        builder.build()
    }
}

fn neighbor_is_air(
    chunk: &Chunk,
    neighbours: &Neighbors,
    x: usize,
    y: usize,
    z: usize,
    offset: IVec3,
) -> bool {
    let nx = x as i32 + offset.x;
    let ny = y as i32 + offset.y;
    let nz = z as i32 + offset.z;

    if (0..CHUNK_SIZE as i32).contains(&nx)
        && (0..CHUNK_SIZE as i32).contains(&ny)
        && (0..CHUNK_SIZE as i32).contains(&nz)
    {
        return chunk.get(nx as usize, ny as usize, nz as usize).is_air();
    }

    let Some(nchunk) = neighbours.get_from_normal(offset) else {
        return true;
    };

    let lx = ((nx % CHUNK_SIZE as i32) + CHUNK_SIZE as i32) % CHUNK_SIZE as i32;
    let ly = ((ny % CHUNK_SIZE as i32) + CHUNK_SIZE as i32) % CHUNK_SIZE as i32;
    let lz = ((nz % CHUNK_SIZE as i32) + CHUNK_SIZE as i32) % CHUNK_SIZE as i32;

    nchunk.get(lx as usize, ly as usize, lz as usize).is_air()
}
