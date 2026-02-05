use bevy::{
    mesh::MeshVertexBufferLayoutRef,
    pbr::{Material, MaterialPipeline, MaterialPipelineKey, MaterialPlugin},
    prelude::*,
    render::render_resource::{
        AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError,
    },
    shader::ShaderRef,
};

use crate::plugins::world::meshers::naive_mesher::ATTRIBUTE_TILE_ID;

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct VoxelAtlasMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub atlas: Handle<Image>,

    #[uniform(2)]
    pub grid: UVec2,
}

impl Material for VoxelAtlasMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/voxel_atlas.wgsl".into()
    }
    fn fragment_shader() -> ShaderRef {
        "shaders/voxel_atlas.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
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
        app.add_plugins(MaterialPlugin::<VoxelAtlasMaterial>::default());
    }
}
