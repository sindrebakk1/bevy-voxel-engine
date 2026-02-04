#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}

struct MaterialUniform {
    grid: vec2<u32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var atlas_tex : texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var atlas_smp : sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var<uniform> material : MaterialUniform;

struct VertexIn {
    @builtin(instance_index) instance_index : u32,

    // These MUST match the shader locations you used in specialize()
    @location(0) position : vec3<f32>,
    @location(1) normal   : vec3<f32>,
    @location(2) uv       : vec2<f32>,
    @location(3) tile_id : u32,
};

struct VertexOut {
    @builtin(position) clip_position : vec4<f32>,
    @location(0) uv       : vec2<f32>,
    @location(1) tile_id : u32,
};

fn atlas_uv(base_uv: vec2<f32>, id: u32) -> vec2<f32> {
    let gx = material.grid.x;
    let gy = material.grid.y;

    // Avoid divide-by-zero if someone misconfigures the material
    if (gx == 0u || gy == 0u) {
        return base_uv;
    }

    let max_id = gx * gy;
    if (id >= max_id) {
        return base_uv; // or clamp, or return debug color elsewhere
    }

    let x = id / gy;
    let y = id % gy;

    let tile_size = vec2<f32>(1.0 / f32(gx), 1.0 / f32(gy));
    let origin = vec2<f32>(f32(x), f32(y)) * tile_size;

    // Slightly clamp into the tile to reduce bleeding
    let eps = 0.001;
    let uv_clamped = clamp(base_uv, vec2<f32>(eps), vec2<f32>(1.0 - eps));

    return origin + uv_clamped * tile_size;
}

@vertex
fn vertex(v: VertexIn) -> VertexOut {
    var out: VertexOut;

    out.clip_position = mesh_position_local_to_clip(
        get_world_from_local(v.instance_index),
        vec4<f32>(v.position, 1.0),
    );

    out.uv = v.uv;
    out.tile_id = v.tile_id;
    return out;
}

@fragment
fn fragment(in: VertexOut) -> @location(0) vec4<f32> {
    let uv = atlas_uv(in.uv, in.tile_id);
    let c = textureSample(atlas_tex, atlas_smp, uv);

    return c;
}

