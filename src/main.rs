use bevy::{
    DefaultPlugins,
    app::{App, AppExit, PluginGroup},
    window::{Window, WindowPlugin},
};

use aettesaga::GamePlugin;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins::set(
            DefaultPlugins,
            WindowPlugin {
                primary_window: Some(Window {
                    title: "Ættesaga".into(),
                    ..Default::default()
                }),
                ..Default::default()
            },
        ))
        .add_plugins(GamePlugin)
        .run()
}
