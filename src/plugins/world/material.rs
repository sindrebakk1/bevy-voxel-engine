use bevy::{
    mesh::MeshVertexBufferLayoutRef,
    pbr::{
        ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline,
        MaterialPlugin,
    },
    prelude::*,
    render::render_resource::{
        AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError,
    },
    shader::ShaderRef,
};

use crate::plugins::world::meshers::naive_mesher::ATTRIBUTE_TILE_ID;

const SHADER_ASSET_PATH: &str = "shaders/voxel_atlas.wgsl";

pub type VoxelAtlasMaterial = ExtendedMaterial<StandardMaterial, VoxelAtlasMaterialExtension>;

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct VoxelAtlasMaterialExtension {
    #[texture(100)]
    #[sampler(101)]
    pub atlas: Handle<Image>,

    #[uniform(102)]
    pub grid: UVec2,
}

impl MaterialExtension for VoxelAtlasMaterialExtension {
    fn vertex_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn specialize(
        _pipeline: &MaterialExtensionPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialExtensionKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
            ATTRIBUTE_TILE_ID.at_shader_location(3),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}

pub struct VoxelAtlasMaterialPlugin;

impl Plugin for VoxelAtlasMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, VoxelAtlasMaterialExtension>,
        >::default());
    }
}
