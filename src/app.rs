use bevy::{
    camera::visibility::RenderLayers,
    color::palettes,
    feathers::{
        FeathersPlugins,
        controls::FeathersButton,
        dark_theme::create_dark_theme,
        display::caption,
        theme::{SemanticToken, SurfaceLevel, ThemeBackgroundColor, ThemeProps, UiTheme},
        tokens,
    },
    prelude::*,
    ui_widgets::Activate,
};

use crate::{
    drawing_selector::DrawingSelector,
    drawings::{DrawingContainer, DrawingsPlugin},
    editor::{DrawingEditor, EditorPlugin, EditorUI},
    player::PlayerPlugin,
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
            PlayerPlugin,
            DrawingsPlugin,
        ))
        .init_state::<AppState>()
        .insert_resource(UiTheme(create_theme()))
        .add_systems(Startup, setup)
        .add_systems(OnEnter(AppState::Edit), show_editor)
        .add_systems(OnExit(AppState::Edit), hide_editor);
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, States)]
pub enum AppState {
    #[default]
    Edit,
    Play,
}

fn create_theme() -> ThemeProps {
    let mut props = create_dark_theme();
    let listrow_selected = SemanticToken::new_static("feathers.listrow.bg.selected");
    props
        .token_assignments
        .insert(tokens::LISTROW_BG_SELECTED, listrow_selected.clone());

    let yellow: Color = palettes::basic::YELLOW.into();
    props.semantic_base.insert(listrow_selected.clone(), yellow);

    for level in [
        SurfaceLevel::Higher,
        SurfaceLevel::Highest,
        SurfaceLevel::Floating,
    ] {
        props
            .semantic_overrides
            .entry(level)
            .or_default()
            .insert(listrow_selected.clone(), yellow);
    }
    props
}

const SOURCE_LAYER: RenderLayers = RenderLayers::layer(1);
const TARGET_LAYER: RenderLayers = RenderLayers::layer(2);

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Interpolation {
    #[default]
    Source,
    Target,
}

fn show_editor(mut commands: Commands, q_editor: Single<Entity, With<EditorUI>>) {
    commands.entity(*q_editor).insert(Visibility::Inherited);
}

fn hide_editor(mut commands: Commands, q_editor: Single<Entity, With<EditorUI>>) {
    commands.entity(*q_editor).insert(Visibility::Hidden);
}

fn setup(mut commands: Commands) {
    let all_layers = RenderLayers::default() | SOURCE_LAYER | TARGET_LAYER;
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
        EditorUI
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
        }
        ThemeBackgroundColor(tokens::WINDOW_BG)
        Children [
            @DrawingEditor { @interpolation: Interpolation::Source, @render_layers: SOURCE_LAYER }
            Node {
                flex_grow: 1.0,
                height: percent(100),
                border: UiRect::all(px(2)),
            }
            --
            Node {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                height: percent(100),
            }
            Children [
                @DrawingSelector
                --
                @FeathersButton {
                    @caption: bsn! { @caption("Play") },
                }
                on(|_activate: On<Activate>, mut next_state: ResMut<NextState<AppState>>| {
                    next_state.set(AppState::Play);
                })
            ]
            --
            @DrawingEditor { @interpolation: Interpolation::Target, @render_layers: TARGET_LAYER }
            Node {
                flex_grow: 1.0,
                height: percent(100),
                border: UiRect::all(px(2)),
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
