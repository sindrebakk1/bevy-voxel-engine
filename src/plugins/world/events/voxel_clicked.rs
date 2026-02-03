use bevy::prelude::*;

use crate::plugins::world::{
    chunk::{Voxel, CHUNK_SIZE},
    voxel_picking::{VoxelFace, VoxelHit},
    ChunkComponent, ChunkCoord, ChunkMap,
};

#[derive(Event, Debug, Clone, Copy)]
pub struct VoxelClicked {
    pub hit: VoxelHit,
    pub button: MouseButton,
}

pub fn on_voxel_clicked(event: On<VoxelClicked>, mut commands: Commands, chunk_map: Res<ChunkMap>) {
    let VoxelClicked { hit, button } = *event.event();

    let local = hit.local.as_uvec3();
    if local.x >= CHUNK_SIZE as u32 || local.y >= CHUNK_SIZE as u32 || local.z >= CHUNK_SIZE as u32
    {
        info!("out of bounds");
        return;
    }

    info!("clicked: ({:?}, {:?}, {:?})", local.x, local.y, local.z);

    match button {
        MouseButton::Left => {
            let Some((chunk, local)) = adjacent_voxel(hit) else {
                return;
            };

            info!("{chunk:?}, {local:?}");

            let Some(entity) = chunk_map.get(&ChunkCoord(chunk)) else {
                return;
            };

            commands
                .entity(entity)
                .entry::<ChunkComponent>()
                .and_modify(move |mut chunk_comp| {
                    chunk_comp.chunk.set(
                        local.x as usize,
                        local.y as usize,
                        local.z as usize,
                        Voxel::Solid,
                    );
                });
        }

        MouseButton::Right => {
            let Some(entity) = chunk_map.get(&ChunkCoord(hit.chunk)) else {
                return;
            };

            commands
                .entity(entity)
                .entry::<ChunkComponent>()
                .and_modify(move |mut chunk_comp| {
                    chunk_comp.chunk.set(
                        local.x as usize,
                        local.y as usize,
                        local.z as usize,
                        Voxel::Air,
                    );
                });
        }

        _ => {}
    }
}

fn adjacent_voxel(hit: VoxelHit) -> Option<(IVec3, IVec3)> {
    let max = CHUNK_SIZE as i32 - 1;

    let normal = match hit.face {
        VoxelFace::PosX => IVec3::X,
        VoxelFace::NegX => -IVec3::X,
        VoxelFace::PosY => IVec3::Y,
        VoxelFace::NegY => -IVec3::Y,
        VoxelFace::PosZ => IVec3::Z,
        VoxelFace::NegZ => -IVec3::Z,
    };

    let mut chunk = hit.chunk;
    let mut local = hit.local + normal;

    if local.x < 0 {
        local.x = max;
        chunk.x -= 1;
    } else if local.x > max {
        local.x = 0;
        chunk.x += 1;
    }

    if local.y < 0 {
        local.y = max;
        chunk.y -= 1;
    } else if local.y > max {
        local.y = 0;
        chunk.y += 1;
    }

    if local.z < 0 {
        local.z = max;
        chunk.z -= 1;
    } else if local.z > max {
        local.z = 0;
        chunk.z += 1;
    }

    Some((chunk, local))
}
