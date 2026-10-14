use bevy::{
    camera::visibility::RenderLayers,
    color::palettes::css,
    feathers::{
        FeathersPlugins,
        dark_theme::create_dark_theme,
        theme::{ThemeBackgroundColor, UiTheme},
        tokens,
    },
    prelude::*,
};

use crate::{
    drawing_selector::DrawingSelector,
    editor::{DrawingEditor, EditorPlugin},
};

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Terp3D".into(),
                    resolution: (1200, 600).into(),
                    ..default()
                }),
                ..default()
            }),
            FeathersPlugins,
            EditorPlugin,
        ))
        .insert_resource(UiTheme(create_dark_theme()))
        .add_systems(Startup, setup);
    }
}

const SOURCE_LAYER: RenderLayers = RenderLayers::layer(1);
const TARGET_LAYER: RenderLayers = RenderLayers::layer(2);

fn setup(mut commands: Commands) {
    let all_layers = SOURCE_LAYER.union(&TARGET_LAYER);
    commands.spawn_scene_list(bsn_list! {
        Camera3d
        IsDefaultUiCamera
        --
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
        }
        ThemeBackgroundColor(tokens::WINDOW_BG)
        Children [
            DrawingEditor { render_layers: SOURCE_LAYER }
            BorderColor::all(css::YELLOW)
            Node {
                flex_grow: 1.0,
                height: percent(100),
                border: UiRect::all(px(8)),
            }
            --
            @DrawingSelector {
                @count: 3,
                @selected: None,
            }
            --
            DrawingEditor { render_layers: TARGET_LAYER }
            BorderColor::all(css::BLUE)
            Node {
                flex_grow: 1.0,
                height: percent(100),
                border: UiRect::all(px(8)),
            }
        ]
        --
        @cube(Color::srgb_u8(124, 144, 255))
        SOURCE_LAYER
        Transform::from_xyz(1.0, 1.0, 1.0)
        --
        @cube(Color::srgb_u8(124, 0, 0))
        TARGET_LAYER
        Transform::from_xyz(-1.0, -1.0, -1.0)
        --
        all_layers
        DirectionalLight {
            illuminance: light_consts::lux::RAW_SUNLIGHT,
        }
        Transform::from_xyz(1.0, 0.15, 1.0).looking_at(Vec3::ZERO, Vec3::Y)
    });
}

fn cube(color: Color) -> impl Scene {
    bsn! {
        #Cube
        Mesh3d(asset_value(Cuboid::new(1.0, 1.0, 1.0)))
        MeshMaterial3d::<StandardMaterial>(asset_value(color))
    }
}
