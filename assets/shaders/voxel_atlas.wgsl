#import bevy_pbr::{
    pbr_types,
    pbr_functions::alpha_discard,
    pbr_fragment::pbr_input_from_standard_material,
    decal::clustered::apply_decals,
};

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
};
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
    pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT,
};
#endif

#ifdef VISIBILITY_RANGE_DITHER
#import bevy_pbr::pbr_functions::visibility_range_dither;
#endif
#ifdef OIT_ENABLED
#import bevy_core_pipeline::oit::oit_draw;
#endif
#ifdef FORWARD_DECAL
#import bevy_pbr::decal::forward::get_forward_decal_info;
#endif

#import bevy_pbr::mesh_functions::{
    get_world_from_local,
    mesh_position_local_to_clip,
    mesh_position_local_to_world,
    mesh_normal_local_to_world,
};

struct MaterialUniform {
    grid: vec2<u32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(100) var atlas_tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(101) var atlas_smp: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(102) var<uniform> material: MaterialUniform;

struct VertexIn {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) tile_id: u32,
};

struct VertexOut {
    @builtin(position) position: vec4<f32>,

    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(6) @interpolate(flat) instance_index: u32,

    @location(20) @interpolate(flat) tile_id: u32,
};

fn atlas_uv(base_uv: vec2<f32>, id: u32) -> vec2<f32> {
    let gx = material.grid.x;
    let gy = material.grid.y;
    if (gx == 0u || gy == 0u) { return base_uv; }

    let max_id = gx * gy;
    if (id >= max_id) { return base_uv; }

    let x = id / gy;
    let y = id % gy;

    let tile_size = vec2<f32>(1.0 / f32(gx), 1.0 / f32(gy));
    let origin = vec2<f32>(f32(x), f32(y)) * tile_size;

    let eps = 0.001;
    let uv_clamped = clamp(base_uv, vec2<f32>(eps), vec2<f32>(1.0 - eps));
    return origin + uv_clamped * tile_size;
}

@vertex
fn vertex(v: VertexIn) -> VertexOut {
    let w = get_world_from_local(v.instance_index);

    var out: VertexOut;
    out.position = mesh_position_local_to_clip(w, vec4<f32>(v.position, 1.0));
    out.world_position = mesh_position_local_to_world(w, vec4<f32>(v.position, 1.0));
    out.world_normal = mesh_normal_local_to_world(v.normal, v.instance_index);
    out.uv = v.uv;
    out.instance_index = v.instance_index;
    out.tile_id = v.tile_id;
    return out;
}

@fragment
fn fragment(
    vin: VertexOut,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    // Rebuild Bevy’s VertexOutput from our extended output.
    var in: VertexOutput;
    in.position = vin.position;
    in.world_position = vin.world_position;
    in.world_normal = vin.world_normal;
    in.uv = vin.uv;
    in.instance_index = vin.instance_index;

#ifdef VISIBILITY_RANGE_DITHER
    visibility_range_dither(in.position, in.visibility_range_dither);
#endif

#ifdef FORWARD_DECAL
    let forward_decal_info = get_forward_decal_info(in);
    in.world_position = forward_decal_info.world_position;
    in.uv = forward_decal_info.uv;
#endif

    var pbr_input = pbr_input_from_standard_material(in, is_front);

    // >>> atlas albedo injection
    let uv = atlas_uv(in.uv, vin.tile_id);
    let tex = textureSample(atlas_tex, atlas_smp, uv);
    pbr_input.material.base_color *= tex;

    // keep StandardMaterial behavior after this point
    pbr_input.material.base_color =
        alpha_discard(pbr_input.material, pbr_input.material.base_color);

    apply_decals(&pbr_input);

#ifdef PREPASS_PIPELINE
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    if (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u {
        out.color = apply_pbr_lighting(pbr_input);
    } else {
        out.color = pbr_input.material.base_color;
    }
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif

#ifdef OIT_ENABLED
    let alpha_mode = pbr_input.material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_ALPHA_MODE_RESERVED_BITS;
    if alpha_mode != pbr_types::STANDARD_MATERIAL_FLAGS_ALPHA_MODE_OPAQUE {
        oit_draw(in.position, out.color);
        discard;
    }
#endif

#ifdef FORWARD_DECAL
    out.color.a = min(forward_decal_info.alpha, out.color.a);
#endif

    return out;
}

