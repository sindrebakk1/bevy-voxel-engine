mod plugins;
mod state;

use bevy::camera_controller::free_camera::{FreeCamera, FreeCameraPlugin};
use bevy::prelude::*;

use plugins::{
    AssetLoaderPlugin, MeshDebugPlugin, WorldPlugin,
    world::{
        ChunkComponent, ChunkCoord,
        blocks::{BLOCK_DIRT, BLOCK_GRASS},
        chunk::{CHUNK_SIZE, Chunk},
        events::VoxelClicked,
        voxel::Voxel,
        voxel_picking::HoveredVoxel,
    },
};
use state::loading_state::LoadingState;

const MAP_HALF_SIZE: i32 = 2;
const MAP_GROUND_HEIGHT: usize = 16;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<LoadingState>()
            .add_plugins((
                AssetLoaderPlugin,
                WorldPlugin,
                FreeCameraPlugin,
                MeshDebugPlugin,
            ))
            .add_systems(
                OnEnter(LoadingState::Initialized),
                (spawn_test_chunks, spawn_camera),
            )
            .add_systems(
                PreUpdate,
                emit_voxel_click_event.run_if(in_state(LoadingState::Initialized)),
            );
    }
}

fn spawn_test_chunks(mut commands: Commands) {
    for chunk_z in 0..3 {
        for chunk_x in -MAP_HALF_SIZE..MAP_HALF_SIZE {
            for chunk_y in -MAP_HALF_SIZE..MAP_HALF_SIZE {
                let mut chunk = Chunk::new();
                if chunk_z == 0 {
                    for x in 0..CHUNK_SIZE {
                        for z in 0..CHUNK_SIZE {
                            for y in 0..MAP_GROUND_HEIGHT - 1 {
                                chunk.set(x, y, z, Voxel::new(BLOCK_DIRT));
                            }
                            chunk.set(x, MAP_GROUND_HEIGHT - 1, z, Voxel::new(BLOCK_GRASS));
                        }
                    }
                }

                commands.spawn((
                    ChunkComponent { chunk },
                    ChunkCoord(IVec3::new(chunk_x, chunk_z, chunk_y)),
                ));
            }
        }
    }
}

fn spawn_camera(mut commands: Commands) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(20.0, 20.0, 20.0).looking_at(
            Vec3::new(CHUNK_SIZE as f32 / 2.0, 4.0, CHUNK_SIZE as f32 / 2.0),
            Vec3::Y,
        ),
        FreeCamera {
            key_up: KeyCode::Space,
            key_down: KeyCode::ControlLeft,
            walk_speed: 10.0,
            run_speed: 20.0,
            mouse_key_cursor_grab: MouseButton::Middle,
            ..default()
        },
    ));

    // Sun
    commands.spawn((
        DirectionalLight {
            illuminance: 7_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.0, -0.8, 0.0)),
    ));
}

fn emit_voxel_click_event(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    hovered: Res<HoveredVoxel>,
) {
    let Some(hit) = hovered.hit else {
        return;
    };

    if mouse.just_pressed(MouseButton::Left) {
        commands.trigger(VoxelClicked {
            hit,
            button: MouseButton::Left,
        });
    }
    if mouse.just_pressed(MouseButton::Right) {
        commands.trigger(VoxelClicked {
            hit,
            button: MouseButton::Right,
        });
    }
}
