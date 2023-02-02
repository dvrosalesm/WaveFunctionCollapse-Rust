use bevy::prelude::*;
use wfc::core::WFCPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                width: 1000.0,
                height: 1000.0,
                title: "RUSTY WFC".to_string(),
                ..default()
            },
            ..default()
        }))
        .add_plugin(WFCPlugin)
        .run();
}
