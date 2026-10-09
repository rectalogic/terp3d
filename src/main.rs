use bevy::{camera::visibility::RenderLayers, color::palettes::css, prelude::*};
use terp3d::{DrawingEditor, EditorPlugin};

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
        .add_systems(PostStartup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_scene_list(bsn_list! {
        Camera2d
        --
        Node {
            width: percent(100),
            height: percent(100),
        }
        Children [
            @DrawingEditor { @render_layers: RenderLayers::layer(1) }
            // BackgroundColor(css::YELLOW)
            FixedNode
            Node {
                left: percent(0),
                width: percent(50),
                height: percent(100),
            }
            --
            @DrawingEditor { @render_layers: RenderLayers::layer(2) }
            // BackgroundColor(css::BLUE)
            FixedNode
            Node {
                left: percent(50),
                width: percent(50),
                height: percent(100),
            }
        ]
        --
        @cube(Color::srgb_u8(124, 144, 255))
        RenderLayers::layer(1)
        --
        @cube(Color::srgb_u8(124, 0, 0))
        RenderLayers::layer(1)
    });
}

fn cube(color: Color) -> impl Scene {
    bsn! {
        #Cube
        Mesh3d(asset_value(Cuboid::new(1.0, 1.0, 1.0)))
        MeshMaterial3d::<StandardMaterial>(asset_value(color))
    }
}
