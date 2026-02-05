pub mod assets;

use bevy::{
    asset::LoadState,
    image::{ImageSampler, ImageSamplerDescriptor},
    prelude::*,
};

use crate::{plugins::world::material::VoxelAtlasMaterial, state::loading_state::LoadingState};

use assets::*;

pub struct AssetLoaderPlugin;

impl Plugin for AssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(LoadingState::Loading), load_assets)
            .add_systems(
                Update,
                finalize_assets.run_if(in_state(LoadingState::Loading)),
            );
    }
}

fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(GameAssets {
        block_atlas: asset_server.load("textures/blocks.png"),
    });
}

fn finalize_assets(
    mut commands: Commands,
    assets: Res<GameAssets>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<VoxelAtlasMaterial>>,
    mut next_state: ResMut<NextState<LoadingState>>,
) {
    if !matches!(
        asset_server.get_load_state(&assets.block_atlas),
        Some(LoadState::Loaded)
    ) {
        return;
    }

    let img = images
        .get_mut(&assets.block_atlas)
        .expect("Loaded but image missing from Assets<Image>");

    img.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor::nearest());

    let w = img.size().x;
    let h = img.size().y;

    assert!(
        w.is_multiple_of(32) && h == 32 * 3,
        "textures/blocks.png is {}x{} but must be divisible by 32 (tile size 32x32, no padding).",
        w,
        h
    );

    let grid = UVec2::new(w / 32, h / 32);

    let material = materials.add(VoxelAtlasMaterial {
        atlas: assets.block_atlas.clone(),
        grid,
    });

    commands.insert_resource(VoxelAtlasHandles { material });

    next_state.set(LoadingState::Initialized);
}
