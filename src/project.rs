use std::iter;

use bevy::{
    animation::{AnimatedBy, AnimationTargetId, animated_field},
    asset::uuid::Uuid,
    ecs::template::OptionTemplate,
    pbr::ExtendedMaterial,
    prelude::*,
};

use crate::{
    app::{AppState, Interpolation},
    camera_controller::CameraController,
    drawings::Drawing,
    player::{Player, PlayerDrawing},
    rim_effect::RimEffect,
};

pub struct TerpProject {
    camera: Transform,
    drawings: Vec<TerpDrawing>,
}

struct TerpDrawing {
    colors: (Oklaba, Oklaba),
    scales: (f32, f32),
    translations: Vec<(Vec3, Vec3)>,
}

impl TerpProject {
    pub fn build(
        camera_entity: Entity,
        q_drawings: Query<(&Drawing, &Interpolation, &Children)>,
        q_transforms: Query<&Transform>,
        extended_materials: Res<Assets<ExtendedMaterial<StandardMaterial, RimEffect>>>,
    ) -> Self {
        let drawings = iter::zip(
            iter_drawings(Interpolation::Source, &q_drawings),
            iter_drawings(Interpolation::Target, &q_drawings),
        )
        .map(
            |((source_drawing, source_children), (target_drawing, target_children))| {
                let scales = (source_drawing.scale, target_drawing.scale);

                let target_material = extended_materials
                    .get(target_drawing.material.id())
                    .expect("material");
                let source_material = extended_materials
                    .get(source_drawing.material.id())
                    .expect("material");
                let colors: (Oklaba, Oklaba) = (
                    source_material.base.base_color.into(),
                    target_material.base.base_color.into(),
                );

                let source_translations: Vec<_> =
                    child_translations(source_children, &q_transforms);
                let target_translations: Vec<_> =
                    child_translations(target_children, &q_transforms);
                let translations: Vec<_> =
                    zip_pad(&source_translations, &target_translations).collect();

                TerpDrawing {
                    colors,
                    scales,
                    translations,
                }
            },
        )
        .collect();

        let transform = q_transforms.get(camera_entity).unwrap();
        TerpProject {
            camera: *transform,
            drawings,
        }
    }

    #[expect(clippy::too_many_arguments)]
    pub fn play(
        &self,
        mut commands: Commands,
        camera_entity: Entity,
        mut q_transforms: Query<&mut Transform>,
        mut animations: ResMut<Assets<AnimationClip>>,
        mut graphs: ResMut<Assets<AnimationGraph>>,
        materials: ResMut<Assets<StandardMaterial>>,
        drawing_mesh: &Handle<Mesh>,
    ) {
        if let Ok(mut transform) = q_transforms.get_mut(camera_entity) {
            *transform = self.camera;
        }

        let mut animation = AnimationClip::default();
        let mut scene_commands = commands.spawn_scene(bsn! {
            #Player
            Player
            CameraController { camera: OptionTemplate::Some(camera_entity) }
            DespawnOnExit::<AppState>(AppState::Play)
        });
        scene_commands.queue_spawn_related_scenes::<Children>(self.player_drawings(
            scene_commands.id(),
            &mut animation,
            materials,
            drawing_mesh,
        ));
        let (animation_graph, animation_index) =
            AnimationGraph::from_clip(animations.add(animation));
        let mut animation_player = AnimationPlayer::default();
        animation_player.play(animation_index).repeat();
        scene_commands.insert((
            animation_player,
            AnimationGraphHandle(graphs.add(animation_graph)),
        ));
    }

    fn player_drawings(
        &self,
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

        self.drawings
            .iter()
            .map(|drawing| {
                let (source_color, target_color) = drawing.colors;
                let material = materials.add(StandardMaterial {
                    base_color: source_color.into(),
                    ..default()
                });

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

fn child_translations(children: &Children, q_transforms: &Query<&Transform>) -> Vec<Vec3> {
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
