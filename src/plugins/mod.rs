pub mod asset_loader;
pub mod character;
pub mod debug;
pub mod player;
pub mod world;

pub use {asset_loader::AssetLoaderPlugin, debug::mesh_debug::MeshDebugPlugin, world::WorldPlugin};
