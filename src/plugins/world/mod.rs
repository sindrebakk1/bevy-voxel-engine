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
use meshers::{ChunkMesher, NaiveMesher};
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

#[derive(Component, Default)]
#[require(ChunkCoord, Transform)]
pub struct ChunkComponent;

pub trait SpawnChunkCommandExt {
    fn spawn_chunk(&mut self, chunk: Chunk, coord: IVec3);
}

impl<'w, 's> SpawnChunkCommandExt for Commands<'w, 's> {
    fn spawn_chunk(&mut self, chunk: Chunk, coord: IVec3) {
        self.queue(move |world: &mut World| {
            let entity = world.spawn((ChunkComponent, ChunkCoord(coord))).id();
            world.resource_mut::<Chunks>().0.insert(entity, chunk);
        })
    }
}

#[derive(Component, Copy, Clone, Eq, PartialEq, Default, Hash, MapEntities, Debug)]
#[component(on_add = on_add_chunk_coord)]
pub struct ChunkCoord(pub IVec3);

impl ChunkCoord {
    pub fn translation(&self) -> Vec3 {
        Vec3::new(
            (self.0.x * CHUNK_SIZE as i32) as f32,
            (self.0.y * CHUNK_SIZE as i32) as f32,
            (self.0.z * CHUNK_SIZE as i32) as f32,
        )
    }
}

fn on_add_chunk_coord(mut world: DeferredWorld, context: HookContext) {
    let mut entity_mut = world.entity_mut(context.entity);

    debug_assert!(
        entity_mut.contains_id(context.component_id),
        "added component not present on entity"
    );

    let chunk_coord = *entity_mut.get::<ChunkCoord>().unwrap();

    let translation = chunk_coord.translation();

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
        .insert(chunk_coord, context.entity);
}

#[derive(Resource, MapEntities, Debug, Default)]
pub struct ChunkEntityMap {
    #[entities]
    chunks: HashMap<ChunkCoord, Entity>,
}

impl ChunkEntityMap {
    pub fn insert(&mut self, chunk_coord: ChunkCoord, entity: Entity) -> bool {
        self.chunks.insert(chunk_coord, entity).is_some()
    }

    pub fn get(&self, chunk_coord: &ChunkCoord) -> Option<Entity> {
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

fn rebuild_dirty_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    block_registry: Res<BlockRegistryRes>,
    mesher: Res<MesherResource>,
    handles: Res<VoxelAtlasHandles>,
    mut chunk_query: Query<(Entity, &ChunkCoord, Option<&mut Mesh3d>), With<ChunkComponent>>,
    mut chunks: ResMut<Chunks>,
) {
    for (entity, _chunk_coord, mesh3d_opt) in chunk_query.iter_mut() {
        chunks.0.entry(entity).and_modify(|chunk| {
            if !chunk.is_dirty() {
                return;
            }

            let mesh = mesher.0.build_mesh(chunk, &block_registry.0);

            let handle = meshes.add(mesh);

            match mesh3d_opt {
                Some(mut mesh3d) => mesh3d.0 = handle,
                None => {
                    commands
                        .entity(entity)
                        .insert((Mesh3d(handle), MeshMaterial3d(handles.material.clone())));
                }
            };

            chunk.clear_dirty();
        });
    }
}
