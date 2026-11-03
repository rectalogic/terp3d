use bevy::{
    animation::{AnimatedBy, AnimationTargetId, animated_field},
    asset::uuid::Uuid,
    ecs::template::{EntityTemplate, OptionTemplate},
    pbr::ExtendedMaterial,
    prelude::*,
};
use std::iter;

use crate::{
    app::{AppState, Interpolation},
    camera_controller::CameraController,
    drawings::{Drawing, DrawingMesh},
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
    mut commands: Commands,
    mut animations: ResMut<Assets<AnimationClip>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    extended_materials: Res<Assets<ExtendedMaterial<StandardMaterial, RimEffect>>>,
    standard_materials: ResMut<Assets<StandardMaterial>>,
    q_drawings: Query<(&Drawing, &Interpolation, &Children)>,
    q_transforms: Query<&Transform, Without<IsDefaultUiCamera>>,
    q_main_camera_transform: Single<(Entity, &mut Transform), With<IsDefaultUiCamera>>,
    q_editor_camera_transforms: Query<
        (&Transform, &Interpolation),
        (With<Camera>, Without<IsDefaultUiCamera>),
    >,
    drawing_mesh: Res<DrawingMesh>,
) {
    let (camera_entity, mut camera_transform) = q_main_camera_transform.into_inner();
    if let Some(transform) =
        q_editor_camera_transforms
            .iter()
            .find_map(|(transform, interpolation)| {
                (*interpolation == Interpolation::Source).then_some(transform)
            })
    {
        *camera_transform = *transform;
    };

    let mut animation = AnimationClip::default();
    let mut scene_commands = commands.spawn_scene(bsn! {
        #Player
        Node {
            width: percent(100),
            height: percent(100),
        }
        Player
        CameraController { camera: OptionTemplate::Some(camera_entity) }
        DespawnOnExit::<AppState>(AppState::Play)
        @player_drawings(#Player, &mut animation, q_drawings, extended_materials, standard_materials, q_transforms, &drawing_mesh.0)
    });
    let (animation_graph, animation_index) = AnimationGraph::from_clip(animations.add(animation));
    let mut player = AnimationPlayer::default();
    player.play(animation_index).repeat();
    scene_commands.insert((player, AnimationGraphHandle(graphs.add(animation_graph))));
}

fn player_drawings(
    player: EntityTemplate,
    animation: &mut AnimationClip,
    q_drawings: Query<(&Drawing, &Interpolation, &Children)>,
    extended_materials: Res<Assets<ExtendedMaterial<StandardMaterial, RimEffect>>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    q_transforms: Query<&Transform, Without<IsDefaultUiCamera>>,
    drawing_mesh: &Handle<Mesh>,
) -> impl Scene {
    let curve = EasingCurve::new(0.0, 1.0, EaseFunction::CubicInOut)
        .reparametrize_linear(interval(0.0, 2.5).unwrap())
        .expect("good curve")
        .ping_pong()
        .expect("good curve");

    let drawings: Vec<Box<dyn SceneList>> = iter::zip(
        iter_drawings(Interpolation::Source, &q_drawings),
        iter_drawings(Interpolation::Target, &q_drawings),
    )
    .map(
        |((source_drawing, source_children), (target_drawing, target_children))| {
            let target_material = extended_materials
                .get(target_drawing.material.id())
                .expect("material");
            let source_material = extended_materials
                .get(source_drawing.material.id())
                .expect("material");
            let source_color = source_material.base.base_color;
            let target_color = target_material.base.base_color;
            let source_material = source_material.base.clone();
            let material = standard_materials.add(source_material);

            let drawing_animation_target_id = AnimationTargetId(Uuid::new_v4());
            let color_curve = curve
                .clone()
                .map(move |t| Oklaba::from(source_color.mix(&target_color, t)));
            animation.add_curve_to_target(
                drawing_animation_target_id,
                AnimatableCurve::new(animated_field!(PlayerDrawing::color), color_curve),
            );

            let source_translations: Vec<_> = child_translations(source_children, &q_transforms);
            let target_translations: Vec<_> = child_translations(target_children, &q_transforms);
            let scales = (source_drawing.scale, target_drawing.scale);
            let meshes = drawing_meshes(
                player,
                zip_pad(&source_translations, &target_translations),
                scales,
                drawing_mesh,
                animation,
                curve.clone(),
                &material,
            );

            bsn! {
                PlayerDrawing { material, color: source_color }
                drawing_animation_target_id
                AnimatedBy(player)
                Children [
                    {meshes}
                ]
            }
            .into()
        },
    )
    .collect();

    bsn! {
        Children [
            {drawings}
        ]
    }
}

fn drawing_meshes(
    player: EntityTemplate,
    translations: impl Iterator<Item = (Vec3, Vec3)>,
    scales: (f32, f32),
    mesh: &Handle<Mesh>,
    animation: &mut AnimationClip,
    curve: impl AnimationCompatibleCurve<f32>,
    material: &Handle<StandardMaterial>,
) -> impl SceneList {
    let source_scale = Vec3::splat(scales.0);
    let target_scale = Vec3::splat(scales.1);
    let scale_curve = curve
        .clone()
        .map(move |t| source_scale.lerp(target_scale, t));
    translations
        .map(|(source_translation, target_translation)| {
            let animation_target_id = AnimationTargetId(Uuid::new_v4());
            let translation_curve = curve
                .clone()
                .map(move |t| source_translation.lerp(target_translation, t));
            animation.add_curve_to_target(
                animation_target_id,
                AnimatableCurve::new(animated_field!(Transform::translation), translation_curve),
            );
            animation.add_curve_to_target(
                animation_target_id,
                AnimatableCurve::new(animated_field!(Transform::scale), scale_curve.clone()),
            );
            let mesh = mesh.clone();
            let material = material.clone();
            bsn! {
                AnimatedBy(player)
                animation_target_id
                Mesh3d(mesh)
                MeshMaterial3d::<_>(material)
                Transform::from_translation(source_translation).with_scale(source_scale)
            }
        })
        .collect::<Vec<_>>()
}

fn iter_drawings<'q>(
    interpolation: Interpolation,
    q_drawings: &'q Query<(&Drawing, &Interpolation, &Children)>,
) -> impl Iterator<Item = (&'q Drawing, &'q Children)> + 'q {
    q_drawings
        .iter()
        .filter_map(move |(drawing, interp, children)| {
            (*interp == interpolation).then_some((drawing, children))
        })
}

fn child_translations(
    children: &Children,
    q_transforms: &Query<&Transform, Without<IsDefaultUiCamera>>,
) -> Vec<Vec3> {
    children
        .iter()
        .filter_map(|entity| q_transforms.get(entity).map(|t| t.translation).ok())
        .collect()
}

fn zip_pad(a: &[Vec3], b: &[Vec3]) -> impl Iterator<Item = (Vec3, Vec3)> {
    let (m, n) = (a.len(), b.len());
    (0..m.max(n)).map(move |i| {
        if m == 0 {
            (b[i], b[i])
        } else if n == 0 {
            (a[i], a[i])
        } else if m <= n {
            let j = (2 * i + 1) * m / (2 * n);
            (a[j], b[i])
        } else {
            let j = (2 * i + 1) * n / (2 * m);
            (a[i], b[j])
        }
    })
}
