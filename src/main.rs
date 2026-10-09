use bevy::prelude::*;
use terp3d::EditorPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Terp3D".into(),
                    resolution: (1200, 600).into(),
                    ..default()
                }),
                ..default()
            }),
            EditorPlugin,
        ))
        // .add_systems(Startup, setup)
        .run();
}
