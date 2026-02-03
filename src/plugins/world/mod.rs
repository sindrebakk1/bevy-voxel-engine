pub mod chunk;
pub mod events;
pub mod meshers;
pub mod voxel_picking;

use bevy::ecs::{entity::MapEntities, lifecycle::HookContext, world::DeferredWorld};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use chunk::{Chunk, CHUNK_SIZE};
use events::on_voxel_clicked;
use meshers::{ChunkMesher, NaiveMesher};
use voxel_picking::VoxelPickingPlugin;

#[derive(Resource)]
pub struct MesherResource(pub Box<dyn ChunkMesher>);

#[derive(Component, Default)]
#[require(ChunkCoord, Transform)]
pub struct ChunkComponent {
    pub chunk: Chunk,
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

    assert!(
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
        .resource_mut::<ChunkMap>()
        .insert(chunk_coord, context.entity);
}

#[derive(Resource, MapEntities, Debug, Default)]
pub struct ChunkMap {
    #[entities]
    chunks: HashMap<ChunkCoord, Entity>,
}

impl ChunkMap {
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
        app.insert_resource(MesherResource(Box::new(NaiveMesher)))
            .insert_resource(ChunkMap {
                chunks: HashMap::with_capacity(128),
            })
            .add_plugins(VoxelPickingPlugin)
            .add_systems(Update, rebuild_dirty_chunks)
            .add_observer(on_voxel_clicked);
    }
}

fn rebuild_dirty_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mesher: Res<MesherResource>,
    mut query: Query<(Entity, &mut ChunkComponent, Option<&mut Mesh3d>)>,
) {
    for (entity, mut chunk_comp, mesh3d_opt) in query.iter_mut() {
        if !chunk_comp.chunk.is_dirty() {
            continue;
        }

        let mesh = mesher.0.build_mesh(&chunk_comp.chunk);

        let handle = meshes.add(mesh);

        match mesh3d_opt {
            Some(mut mesh3d) => mesh3d.0 = handle,
            None => {
                commands.entity(entity).insert((
                    Mesh3d(handle),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: LinearRgba::GREEN.into(),
                        ..default()
                    })),
                ));
            }
        };

        chunk_comp.chunk.clear_dirty();
    }
}
