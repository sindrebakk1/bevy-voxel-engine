pub mod blocks;
pub mod chunk;
pub mod events;
pub mod material;
pub mod meshers;
pub mod voxel;
pub mod voxel_picking;

use bevy::ecs::{entity::MapEntities, lifecycle::HookContext, world::DeferredWorld};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use blocks::BlockRegistryRes;
use chunk::{CHUNK_SIZE, Chunk};
use events::on_voxel_clicked;
use material::VoxelAtlasMaterialPlugin;
use meshers::{ChunkMesher, NaiveMesher, Neighbors, Neighbour};
use voxel_picking::VoxelPickingPlugin;

use crate::{plugins::asset_loader::assets::VoxelAtlasHandles, state::LoadingState};

#[derive(Resource)]
pub struct MesherResource(pub Box<dyn ChunkMesher>);

#[derive(Resource)]
pub struct Chunks(HashMap<Entity, Chunk>);

impl Default for Chunks {
    fn default() -> Self {
        Self(HashMap::with_capacity(256))
    }
}

#[derive(Component, Copy, Clone, Eq, PartialEq, Default, Hash, MapEntities, Debug)]
#[require(Transform)]
#[component(on_add = on_add_chunk_component)]
pub struct ChunkComponent {
    pub coord: IVec3,
}

impl ChunkComponent {
    pub fn translation(&self) -> Vec3 {
        Vec3::new(
            (self.coord.x * CHUNK_SIZE as i32) as f32,
            (self.coord.y * CHUNK_SIZE as i32) as f32,
            (self.coord.z * CHUNK_SIZE as i32) as f32,
        )
    }
}

fn on_add_chunk_component(mut world: DeferredWorld, context: HookContext) {
    let mut entity_mut = world.entity_mut(context.entity);

    debug_assert!(
        entity_mut.contains_id(context.component_id),
        "added component not present on entity"
    );

    let chunk_cmp = *entity_mut.get::<ChunkComponent>().unwrap();

    let translation = chunk_cmp.translation();

    if let Some(mut transform) = entity_mut.get_mut::<Transform>() {
        transform.translation = translation;
    } else {
        world
            .commands()
            .entity(context.entity)
            .insert(Transform::from_translation(translation));
    }

    world
        .resource_mut::<ChunkEntityMap>()
        .insert(chunk_cmp.coord, context.entity);
}

pub trait SpawnChunkCommandExt {
    fn spawn_chunk(&mut self, chunk: Chunk, coord: IVec3);
}

impl<'w, 's> SpawnChunkCommandExt for Commands<'w, 's> {
    fn spawn_chunk(&mut self, chunk: Chunk, coord: IVec3) {
        self.queue(move |world: &mut World| {
            let entity = world.spawn(ChunkComponent { coord }).id();
            world.resource_mut::<Chunks>().0.insert(entity, chunk);
        })
    }
}

#[derive(Resource, Debug, Default)]
pub struct ChunkEntityMap {
    chunks: HashMap<IVec3, Entity>,
}

impl ChunkEntityMap {
    pub fn insert(&mut self, chunk_coord: IVec3, entity: Entity) -> bool {
        self.chunks.insert(chunk_coord, entity).is_some()
    }

    pub fn get(&self, chunk_coord: &IVec3) -> Option<Entity> {
        self.chunks.get(chunk_coord).copied()
    }
}

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BlockRegistryRes>()
            .init_resource::<Chunks>()
            .insert_resource(MesherResource(Box::new(NaiveMesher)))
            .insert_resource(ChunkEntityMap {
                chunks: HashMap::with_capacity(128),
            })
            .add_plugins((VoxelAtlasMaterialPlugin, VoxelPickingPlugin))
            .add_systems(
                Update,
                rebuild_dirty_chunks.run_if(in_state(LoadingState::Initialized)),
            )
            .add_observer(on_voxel_clicked);
    }
}

#[allow(clippy::too_many_arguments)]
fn rebuild_dirty_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    block_registry: Res<BlockRegistryRes>,
    mesher: Res<MesherResource>,
    handles: Res<VoxelAtlasHandles>,
    mut chunk_query: Query<(Entity, &ChunkComponent, Option<&mut Mesh3d>)>,
    mut chunks: ResMut<Chunks>,
    chunk_map: Res<ChunkEntityMap>,
) {
    for (entity, chunk_cmp, mesh3d_opt) in chunk_query.iter_mut() {
        // Fast check: if we don't have it, skip
        let Some(is_dirty) = chunks.0.get(&entity).map(|c| c.is_dirty()) else {
            continue;
        };
        if !is_dirty {
            continue;
        }

        let mut chunk = match chunks.0.remove(&entity) {
            Some(c) => c,
            None => continue,
        };

        let neighbours = get_neighbours(&chunk_cmp.coord, &chunk_map, &chunks);
        let mesh = mesher.0.build_mesh(&chunk, neighbours, &block_registry.0);
        let handle = meshes.add(mesh);

        match mesh3d_opt {
            Some(mut mesh3d) => mesh3d.0 = handle,
            None => {
                commands
                    .entity(entity)
                    .insert((Mesh3d(handle), MeshMaterial3d(handles.material.clone())));
            }
        }

        chunk.clear_dirty();

        chunks.0.insert(entity, chunk);
    }
}

fn get_neighbours<'a>(coord: &IVec3, map: &ChunkEntityMap, chunks: &'a Chunks) -> Neighbors<'a> {
    let neighbours = Neighbour::ALL.map(|n| {
        map.get(&(coord + n.normal()))
            .and_then(|entity| chunks.0.get(&entity))
    });
    Neighbors::from_array(neighbours)
}
