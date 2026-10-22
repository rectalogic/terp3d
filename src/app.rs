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
    drawing_selector::{DrawingSelector, DrawingSelectorPlugin},
    drawings::{DrawingContainer, DrawingsPlugin},
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
            DrawingsPlugin,
            DrawingSelectorPlugin,
        ))
        .insert_resource(UiTheme(create_dark_theme()))
        .add_systems(Startup, setup);
    }
}

const SOURCE_LAYER: RenderLayers = RenderLayers::layer(1);
const TARGET_LAYER: RenderLayers = RenderLayers::layer(2);

#[derive(Component, Clone, Copy, Default, PartialEq, Eq)]
pub enum Interpolation {
    #[default]
    Source,
    Target,
}

fn setup(mut commands: Commands) {
    let all_layers = SOURCE_LAYER.union(&TARGET_LAYER);
    commands.spawn_scene_list(bsn_list! {
        Camera3d
        IsDefaultUiCamera
        --
        DrawingContainer
        Interpolation::Source
        SOURCE_LAYER
        Children []
        --
        DrawingContainer
        Interpolation::Target
        TARGET_LAYER
        Children []
        --
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
        }
        ThemeBackgroundColor(tokens::WINDOW_BG)
        Children [
            @DrawingEditor { @interpolation: Interpolation::Source, @render_layers: SOURCE_LAYER }
            BorderColor::all(css::YELLOW)
            Node {
                flex_grow: 1.0,
                height: percent(100),
                border: UiRect::all(px(8)),
            }
            --
            Node {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                height: percent(100),
            }
            Children [
                @DrawingSelector
            ]
            --
            @DrawingEditor { @interpolation: Interpolation::Target, @render_layers: TARGET_LAYER }
            BorderColor::all(css::BLUE)
            Node {
                flex_grow: 1.0,
                height: percent(100),
                border: UiRect::all(px(8)),
            }
        ]
        --
        all_layers
        DirectionalLight {
            illuminance: light_consts::lux::RAW_SUNLIGHT,
        }
        Transform::from_xyz(1.0, 1.0, -1.0).looking_at(Vec3::ZERO, Vec3::Y)
    });
}
