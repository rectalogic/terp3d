use bevy::{
    color::palettes,
    feathers::{
        FeathersPlugins,
        controls::FeathersButton,
        dark_theme::create_dark_theme,
        display::caption,
        theme::{SemanticToken, SurfaceLevel, ThemeBackgroundColor, ThemeProps, UiTheme},
        tokens,
    },
    pbr::ExtendedMaterial,
    prelude::*,
    ui_widgets::Activate,
};

use crate::{
    drawing_selector::DrawingSelector,
    drawings::{Drawing, DrawingContainer, DrawingsPlugin},
    editor::{DrawingEditor, EditorPlugin, EditorUI},
    interpolation::Interpolation,
    player::{PlayProject, PlayerPlugin},
    project::TerpProject,
    rim_effect::RimEffect,
    skybox::SkyboxCamera,
};

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Terp3D".into(),
                    resolution: (1200, 600).into(),
                    canvas: Some("#canvas".into()),
                    fit_canvas_to_parent: true,
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
        .insert_resource(GlobalAmbientLight::NONE)
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

fn show_editor(mut commands: Commands, q_editor: Single<Entity, With<EditorUI>>) {
    commands.entity(*q_editor).insert(Visibility::Inherited);
}

fn hide_editor(mut commands: Commands, q_editor: Single<Entity, With<EditorUI>>) {
    commands.entity(*q_editor).insert(Visibility::Hidden);
}

fn setup(mut commands: Commands) {
    commands.spawn_scene_list(bsn_list! {
        @SkyboxCamera
        IsDefaultUiCamera
        --
        DrawingContainer
        Interpolation::Source
        Interpolation::Source.render_layers().clone()
        Children []
        --
        DrawingContainer
        Interpolation::Target
        Interpolation::Target.render_layers().clone()
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
            @DrawingEditor { @interpolation: Interpolation::Source }
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
                on(on_play)
                --
                @FeathersButton {
                    @caption: bsn! { @caption("Copy URL") },
                }
                on(on_copy_url)
            ]
            --
            @DrawingEditor { @interpolation: Interpolation::Target }
            Node {
                flex_grow: 1.0,
                height: percent(100),
                border: UiRect::all(px(2)),
            }
        ]
    });
}

#[expect(clippy::type_complexity)]
fn on_play(
    _activate: On<Activate>,
    mut commands: Commands,
    extended_materials: Res<Assets<ExtendedMaterial<StandardMaterial, RimEffect>>>,
    q_drawings: Query<(&Drawing, &Interpolation, &Children)>,
    q_transforms: Query<&mut Transform>,
    q_editor_cameras: Query<(Entity, &Interpolation), (With<Camera>, Without<IsDefaultUiCamera>)>,
) {
    let source_camera_entity = q_editor_cameras
        .iter()
        .find_map(|(entity, &interpolation)| {
            (interpolation == Interpolation::Source).then_some(entity)
        })
        .unwrap();

    let project = TerpProject::build(
        source_camera_entity,
        q_drawings,
        q_transforms.as_readonly(),
        extended_materials,
    );
    commands.trigger(PlayProject { project });
}

#[expect(clippy::type_complexity)]
fn on_copy_url(
    _activate: On<Activate>,
    mut clipboard: ResMut<Clipboard>,
    extended_materials: Res<Assets<ExtendedMaterial<StandardMaterial, RimEffect>>>,
    q_drawings: Query<(&Drawing, &Interpolation, &Children)>,
    q_transforms: Query<&mut Transform>,
    q_editor_cameras: Query<(Entity, &Interpolation), (With<Camera>, Without<IsDefaultUiCamera>)>,
) {
    let source_camera_entity = q_editor_cameras
        .iter()
        .find_map(|(entity, &interpolation)| {
            (interpolation == Interpolation::Source).then_some(entity)
        })
        .unwrap();

    let project = TerpProject::build(
        source_camera_entity,
        q_drawings,
        q_transforms.as_readonly(),
        extended_materials,
    );

    if let Ok(project_data) = project.save() {
        let url = format!("https://rectalogic.com/terp3d?project={project_data}");
        let _ = clipboard.set_text(&url);
    }
}
