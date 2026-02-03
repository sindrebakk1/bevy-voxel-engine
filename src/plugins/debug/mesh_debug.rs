use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::prelude::*;

pub struct MeshDebugPlugin;

impl Plugin for MeshDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WireframePlugin::default())
            .insert_resource(WireframeConfig {
                global: false,
                ..default()
            })
            .add_systems(Update, toggle_wireframe);
    }
}

fn toggle_wireframe(keys: Res<ButtonInput<KeyCode>>, mut cfg: ResMut<WireframeConfig>) {
    if keys.just_pressed(KeyCode::F3) {
        cfg.global = !cfg.global;
        info!("Wireframe: {}", cfg.global);
    }
}
