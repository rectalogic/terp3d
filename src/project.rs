use bevy::{
    animation::{AnimatedBy, AnimationTargetId, animated_field},
    asset::uuid::Uuid,
    ecs::template::EntityTemplate,
    prelude::*,
};

use crate::player::PlayerDrawing;

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
    pub fn play(
        &self,
        mut commands: Commands,
        camera_entity: Entity,
        q_transforms: Query<&mut Transform>,
        mut animations: ResMut<Assets<AnimationClip>>,
        mut graphs: ResMut<Assets<AnimationGraph>>,
        drawing_mesh: &Handle<Mesh>,
    ) {
        if let Ok(transform) = q_transforms.get_mut(camera_entity) {
            *transform = self.camera;
        }

        let mut animation = AnimationClip::default();
        let mut scene_commands = commands.spawn_scene(bsn! {
            #Player
            Player
            CameraController { camera: OptionTemplate::Some(camera_entity) }
            DespawnOnExit::<AppState>(AppState::Play)
            Children [
                {self.player_drawings(#Player, &mut animation, standard_materials, drawing_mesh)}
            ]
        });
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
        player: EntityTemplate,
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
                    .map(move |t| Oklaba::from(source_color.mix(&target_color, t)));
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
