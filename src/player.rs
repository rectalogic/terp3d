use bevy::{
    animation::{AnimatedBy, AnimationTargetId, animated_field},
    asset::uuid::Uuid,
    feathers::{controls::FeathersButton, display::caption},
    prelude::*,
    ui_widgets::Activate,
};

use crate::{
    app::AppState,
    camera_controller::CameraController,
    drawings::{DrawingMesh, base_material},
    project::TerpProject,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_play_project).add_systems(
            PostUpdate,
            sync_material_color
                .after(bevy::animation::animate_targets)
                .run_if(in_state(AppState::Play)),
        );
    }
}

#[derive(Event)]
pub struct PlayProject {
    pub project: TerpProject,
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

#[expect(clippy::too_many_arguments)]
fn on_play_project(
    play: On<PlayProject>,
    mut commands: Commands,
    q_main_camera: Single<(Entity, &mut Transform), With<IsDefaultUiCamera>>,
    mut animations: ResMut<Assets<AnimationClip>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    materials: ResMut<Assets<StandardMaterial>>,
    drawing_mesh: Res<DrawingMesh>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let (camera_entity, mut camera_transform) = q_main_camera.into_inner();
    *camera_transform = play.project.camera;

    let mut animation = AnimationClip::default();
    let mut scene_commands = commands.spawn_scene(bsn! {
        #Player
        Player
        Node {
            width: percent(100),
            height: percent(100),
        }
        @CameraController { @camera: camera_entity }
        DespawnOnExit::<AppState>(AppState::Play)
        Children [
            @FeathersButton { @caption: bsn! { @caption("Edit") } }
            on(|_activate: On<Activate>, mut next_state: ResMut<NextState<AppState>>| {
                next_state.set(AppState::Edit);
            })
        ]
    });
    scene_commands.queue_spawn_related_scenes::<Children>(player_drawings(
        &play.project,
        scene_commands.id(),
        &mut animation,
        materials,
        &drawing_mesh.0,
    ));
    let (animation_graph, animation_index) = AnimationGraph::from_clip(animations.add(animation));
    let mut animation_player = AnimationPlayer::default();
    animation_player.play(animation_index).repeat();
    scene_commands.insert((
        animation_player,
        AnimationGraphHandle(graphs.add(animation_graph)),
    ));

    next_state.set(AppState::Play);
}

fn player_drawings(
    project: &TerpProject,
    player: Entity,
    animation: &mut AnimationClip,
    mut materials: ResMut<Assets<StandardMaterial>>,
    drawing_mesh: &Handle<Mesh>,
) -> impl SceneList {
    let curve = EasingCurve::new(0.0, 1.0, EaseFunction::CubicInOut)
        .reparametrize_linear(interval(0.0, 2.5).unwrap())
        .expect("good curve")
        .ping_pong()
        .expect("good curve");

    project
        .drawings
        .iter()
        .map(|drawing| {
            let (source_color, target_color) = drawing.colors;
            let material = materials.add(base_material(source_color.into()));

            let color_animation_target_id = AnimationTargetId(Uuid::new_v4());
            let color_curve = curve
                .clone()
                .map(move |t| source_color.mix(&target_color, t));
            animation.add_curve_to_target(
                color_animation_target_id,
                AnimatableCurve::new(animated_field!(PlayerDrawing::color), color_curve),
            );

            let source_scale = Vec3::splat(drawing.scales.0);
            let target_scale = Vec3::splat(drawing.scales.1);
            let scale_curve = curve
                .clone()
                .map(move |t| source_scale.lerp(target_scale, t));
            let meshes = drawing
                .translations
                .iter()
                .map(|&(source_translation, target_translation)| {
                    let transform_animation_target_id = AnimationTargetId(Uuid::new_v4());
                    let translation_curve = curve
                        .clone()
                        .map(move |t| source_translation.lerp(target_translation, t));
                    animation.add_curve_to_target(
                        transform_animation_target_id,
                        AnimatableCurve::new(
                            animated_field!(Transform::translation),
                            translation_curve,
                        ),
                    );
                    animation.add_curve_to_target(
                        transform_animation_target_id,
                        AnimatableCurve::new(
                            animated_field!(Transform::scale),
                            scale_curve.clone(),
                        ),
                    );
                    let mesh = drawing_mesh.clone();
                    let material = material.clone();
                    bsn! {
                        AnimatedBy(player)
                        transform_animation_target_id
                        Mesh3d(mesh)
                        MeshMaterial3d::<_>(material)
                        Transform::from_translation(source_translation).with_scale(source_scale)
                    }
                })
                .collect::<Vec<_>>();

            bsn! {
                PlayerDrawing { material, color: source_color }
                color_animation_target_id
                AnimatedBy(player)
                Children [
                    {meshes}
                ]
            }
        })
        .collect::<Vec<_>>()
}
