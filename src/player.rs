use bevy::{pbr::ExtendedMaterial, prelude::*};

use crate::{
    app::{AppState, Interpolation},
    drawings::{Drawing, DrawingMesh},
    project::TerpProject,
    rim_effect::RimEffect,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Play), setup_player)
            .add_systems(
                PostUpdate,
                sync_material_color
                    .after(bevy::animation::animate_targets)
                    .run_if(in_state(AppState::Play)),
            );
    }
}

#[derive(Component, Clone, Default)]
#[require(Transform, Visibility)]
pub struct Player;

#[derive(Component, Reflect, Clone, FromTemplate)]
#[require(Transform, Visibility)]
pub struct PlayerDrawing {
    pub material: Handle<StandardMaterial>,
    pub color: Oklaba,
}

fn sync_material_color(
    mut materials: ResMut<Assets<StandardMaterial>>,
    q_drawings: Query<&PlayerDrawing, Changed<PlayerDrawing>>,
) {
    for drawing in &q_drawings {
        if let Some(mut material) = materials.get_mut(&drawing.material) {
            material.base_color = Color::from(drawing.color);
        }
    }
}

#[expect(clippy::too_many_arguments, clippy::type_complexity)]
fn setup_player(
    commands: Commands,
    animations: ResMut<Assets<AnimationClip>>,
    graphs: ResMut<Assets<AnimationGraph>>,
    extended_materials: Res<Assets<ExtendedMaterial<StandardMaterial, RimEffect>>>,
    standard_materials: ResMut<Assets<StandardMaterial>>,
    q_drawings: Query<(&Drawing, &Interpolation, &Children)>,
    q_transforms: Query<&mut Transform>,
    q_editor_cameras: Query<(Entity, &Interpolation), (With<Camera>, Without<IsDefaultUiCamera>)>,
    q_main_camera: Single<Entity, (With<Camera>, With<IsDefaultUiCamera>)>,
    drawing_mesh: Res<DrawingMesh>,
) {
    let source_camera_entity = q_editor_cameras
        .iter()
        .find_map(|(entity, &interpolation)| {
            (interpolation == Interpolation::Source).then_some(entity)
        })
        .unwrap();

    TerpProject::build(
        source_camera_entity,
        q_drawings,
        q_transforms.as_readonly(),
        extended_materials,
    )
    .play(
        commands,
        *q_main_camera,
        q_transforms,
        animations,
        graphs,
        standard_materials,
        &drawing_mesh.0,
    );
}
