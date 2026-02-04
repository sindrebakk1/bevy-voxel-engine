use bevy::color::palettes::basic::YELLOW;
use bevy::math::Ray3d;
use bevy::prelude::*;

use crate::plugins::world::{ChunkComponent, ChunkCoord, chunk::CHUNK_SIZE};

pub struct VoxelPickingPlugin;

impl Plugin for VoxelPickingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HoveredVoxel>()
            .init_resource::<VoxelPickingDebugGizmosEnabled>()
            .add_systems(
                Update,
                (
                    update_voxel_hover,
                    toggle_voxel_picking_gizmos,
                    draw_voxel_hover_gizmos,
                ),
            );
    }
}

/// Toggleable debug draw for the hover highlight.
#[derive(Resource, Default)]
pub struct VoxelPickingDebugGizmosEnabled(pub bool);

/// Which face of the voxel the ray entered through (i.e. the face you are hovering).
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum VoxelFace {
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ,
}

impl VoxelFace {
    pub fn normal_i(self) -> IVec3 {
        match self {
            VoxelFace::PosX => IVec3::X,
            VoxelFace::NegX => IVec3::NEG_X,
            VoxelFace::PosY => IVec3::Y,
            VoxelFace::NegY => IVec3::NEG_Y,
            VoxelFace::PosZ => IVec3::Z,
            VoxelFace::NegZ => IVec3::NEG_Z,
        }
    }

    pub fn normal_f(self) -> Vec3 {
        self.normal_i().as_vec3()
    }
}

/// Public resource you can read anywhere (UI, placing blocks, etc.).
#[derive(Resource, Default, Debug, Clone)]
pub struct HoveredVoxel {
    pub hit: Option<VoxelHit>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct VoxelHit {
    /// Chunk coordinate (in chunk units).
    pub chunk: IVec3,
    /// Local voxel coord inside chunk (0..CHUNK_SIZE-1).
    pub local: IVec3,
    /// World voxel coord (voxel grid).
    pub world: IVec3,
    /// Face that was hit / hovered.
    pub face: VoxelFace,
}

fn toggle_voxel_picking_gizmos(
    keys: Res<ButtonInput<KeyCode>>,
    mut enabled: ResMut<VoxelPickingDebugGizmosEnabled>,
) {
    if keys.just_pressed(KeyCode::F4) {
        enabled.0 = !enabled.0;
        info!("Voxel picking gizmos: {}", enabled.0);
    }
}

/// Update HoveredVoxel by raycasting into voxel grid.
fn update_voxel_hover(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    chunks: Query<(&ChunkComponent, &ChunkCoord)>,
    mut hovered: ResMut<HoveredVoxel>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        hovered.hit = None;
        return;
    };

    let Ok((camera, cam_gt)) = camera_q.single() else {
        hovered.hit = None;
        return;
    };

    let Ok(ray) = camera.viewport_to_world(cam_gt, cursor) else {
        hovered.hit = None;
        return;
    };

    // Tune this if you want longer/shorter reach.
    let max_distance = 128.0;

    hovered.hit = pick_voxel_dda(ray, max_distance, &chunks);
}

/// 3D DDA through the integer voxel grid.
/// Returns first solid voxel hit + the face we entered through.
fn pick_voxel_dda(
    ray: Ray3d,
    max_distance: f32,
    chunks: &Query<(&ChunkComponent, &ChunkCoord)>,
) -> Option<VoxelHit> {
    let origin = ray.origin;
    let dir = ray.direction.normalize();

    // If direction is degenerate, bail.
    if !dir.is_finite() || dir.length_squared() < 1e-12 {
        return None;
    }

    // Current voxel cell in world grid coordinates.
    let mut cell = ivec3_floor(origin);

    // Step direction per axis.
    let step_x = if dir.x > 0.0 {
        1
    } else if dir.x < 0.0 {
        -1
    } else {
        0
    };
    let step_y = if dir.y > 0.0 {
        1
    } else if dir.y < 0.0 {
        -1
    } else {
        0
    };
    let step_z = if dir.z > 0.0 {
        1
    } else if dir.z < 0.0 {
        -1
    } else {
        0
    };

    // How far along the ray we must move for the ray to cross a voxel boundary on each axis.
    let t_delta_x = if step_x != 0 {
        1.0 / dir.x.abs()
    } else {
        f32::INFINITY
    };
    let t_delta_y = if step_y != 0 {
        1.0 / dir.y.abs()
    } else {
        f32::INFINITY
    };
    let t_delta_z = if step_z != 0 {
        1.0 / dir.z.abs()
    } else {
        f32::INFINITY
    };

    // Distance to the first voxel boundary on each axis.
    let next_boundary_x = if step_x > 0 {
        cell.x as f32 + 1.0
    } else {
        cell.x as f32
    };
    let next_boundary_y = if step_y > 0 {
        cell.y as f32 + 1.0
    } else {
        cell.y as f32
    };
    let next_boundary_z = if step_z > 0 {
        cell.z as f32 + 1.0
    } else {
        cell.z as f32
    };

    let mut t_max_x = if step_x != 0 {
        (next_boundary_x - origin.x) / dir.x
    } else {
        f32::INFINITY
    };
    let mut t_max_y = if step_y != 0 {
        (next_boundary_y - origin.y) / dir.y
    } else {
        f32::INFINITY
    };
    let mut t_max_z = if step_z != 0 {
        (next_boundary_z - origin.z) / dir.z
    } else {
        f32::INFINITY
    };

    // Track which face we crossed last to enter the current cell.
    // If we start inside a solid voxel, this will be None; we’ll pick a fallback.
    let mut entered_face: Option<VoxelFace> = None;

    // Check starting cell first.
    if is_solid(cell, chunks) {
        let face = entered_face.unwrap_or(VoxelFace::PosY);
        return Some(build_hit(cell, face));
    }

    // Walk up to max_distance.
    let mut t = 0.0_f32;

    while t <= max_distance {
        // Step to next cell: choose smallest next boundary.
        if t_max_x < t_max_y && t_max_x < t_max_z {
            cell.x += step_x;
            t = t_max_x;
            t_max_x += t_delta_x;
            entered_face = Some(if step_x > 0 {
                VoxelFace::NegX
            } else {
                VoxelFace::PosX
            });
        } else if t_max_y < t_max_z {
            cell.y += step_y;
            t = t_max_y;
            t_max_y += t_delta_y;
            entered_face = Some(if step_y > 0 {
                VoxelFace::NegY
            } else {
                VoxelFace::PosY
            });
        } else {
            cell.z += step_z;
            t = t_max_z;
            t_max_z += t_delta_z;
            entered_face = Some(if step_z > 0 {
                VoxelFace::NegZ
            } else {
                VoxelFace::PosZ
            });
        }

        if is_solid(cell, chunks) {
            let face = entered_face.unwrap_or(VoxelFace::PosY);
            return Some(build_hit(cell, face));
        }
    }

    None
}

fn build_hit(world_cell: IVec3, face: VoxelFace) -> VoxelHit {
    let (chunk, local) = world_to_chunk_local(world_cell);
    VoxelHit {
        chunk,
        local,
        world: world_cell,
        face,
    }
}

fn is_solid(world_cell: IVec3, chunks: &Query<(&ChunkComponent, &ChunkCoord)>) -> bool {
    let (chunk_coord, local) = world_to_chunk_local(world_cell);

    // Find the chunk. This is O(n) right now; if you have many chunks,
    // switch to an index (HashMap) resource.
    for (chunk_comp, cc) in chunks.iter() {
        if cc.0 == chunk_coord {
            let v = chunk_comp
                .chunk
                .get(local.x as usize, local.y as usize, local.z as usize);
            return !v.is_air();
        }
    }

    false
}

/// Convert world voxel coord -> (chunk coord, local voxel coord).
fn world_to_chunk_local(world: IVec3) -> (IVec3, IVec3) {
    let cs = CHUNK_SIZE as i32;

    // Euclidean division so negatives work the way you expect.
    let chunk = IVec3::new(
        div_euclid(world.x, cs),
        div_euclid(world.y, cs),
        div_euclid(world.z, cs),
    );

    let local = IVec3::new(
        rem_euclid(world.x, cs),
        rem_euclid(world.y, cs),
        rem_euclid(world.z, cs),
    );

    (chunk, local)
}

fn div_euclid(a: i32, b: i32) -> i32 {
    a.div_euclid(b)
}

fn rem_euclid(a: i32, b: i32) -> i32 {
    a.rem_euclid(b)
}

fn ivec3_floor(v: Vec3) -> IVec3 {
    IVec3::new(v.x.floor() as i32, v.y.floor() as i32, v.z.floor() as i32)
}

/// Draw a voxel highlight and the hovered face outline using Gizmos.
fn draw_voxel_hover_gizmos(
    mut gizmos: Gizmos,
    enabled: Res<VoxelPickingDebugGizmosEnabled>,
    hovered: Res<HoveredVoxel>,
) {
    if !enabled.0 {
        return;
    }
    let Some(hit) = hovered.hit else {
        return;
    };

    // Voxel center in world space (each voxel is 1 unit).
    let min = hit.world.as_vec3();
    let max = min + Vec3::ONE;
    let center = (min + max) * 0.5;

    // 1) Draw cube outline around voxel
    // (If this doesn't compile in your exact Bevy build, replace with 12 gizmos.line() calls.)
    gizmos.cube(Isometry3d::new(center, Quat::IDENTITY), Color::WHITE);

    // 2) Draw the hovered face as a rectangle outline
    draw_face_outline(&mut gizmos, hit.world, hit.face);
}

fn draw_face_outline(gizmos: &mut Gizmos, voxel_world: IVec3, face: VoxelFace) {
    let base = voxel_world.as_vec3();
    let (a, b, c, d) = match face {
        VoxelFace::PosX => {
            let x = base.x + 1.0;
            (
                Vec3::new(x, base.y, base.z),
                Vec3::new(x, base.y + 1.0, base.z),
                Vec3::new(x, base.y + 1.0, base.z + 1.0),
                Vec3::new(x, base.y, base.z + 1.0),
            )
        }
        VoxelFace::NegX => {
            let x = base.x;
            (
                Vec3::new(x, base.y, base.z + 1.0),
                Vec3::new(x, base.y + 1.0, base.z + 1.0),
                Vec3::new(x, base.y + 1.0, base.z),
                Vec3::new(x, base.y, base.z),
            )
        }
        VoxelFace::PosY => {
            let y = base.y + 1.0;
            (
                Vec3::new(base.x, y, base.z),
                Vec3::new(base.x, y, base.z + 1.0),
                Vec3::new(base.x + 1.0, y, base.z + 1.0),
                Vec3::new(base.x + 1.0, y, base.z),
            )
        }
        VoxelFace::NegY => {
            let y = base.y;
            (
                Vec3::new(base.x, y, base.z + 1.0),
                Vec3::new(base.x, y, base.z),
                Vec3::new(base.x + 1.0, y, base.z),
                Vec3::new(base.x + 1.0, y, base.z + 1.0),
            )
        }
        VoxelFace::PosZ => {
            let z = base.z + 1.0;
            (
                Vec3::new(base.x, base.y, z),
                Vec3::new(base.x + 1.0, base.y, z),
                Vec3::new(base.x + 1.0, base.y + 1.0, z),
                Vec3::new(base.x, base.y + 1.0, z),
            )
        }
        VoxelFace::NegZ => {
            let z = base.z;
            (
                Vec3::new(base.x + 1.0, base.y, z),
                Vec3::new(base.x, base.y, z),
                Vec3::new(base.x, base.y + 1.0, z),
                Vec3::new(base.x + 1.0, base.y + 1.0, z),
            )
        }
    };

    // Slight offset to avoid z-fighting with the voxel surface.
    let n = face.normal_f();
    let eps = 0.002;
    let a = a + n * eps;
    let b = b + n * eps;
    let c = c + n * eps;
    let d = d + n * eps;

    gizmos.line(a, b, YELLOW);
    gizmos.line(b, c, YELLOW);
    gizmos.line(c, d, YELLOW);
    gizmos.line(d, a, YELLOW);
}
