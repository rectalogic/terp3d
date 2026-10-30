use bevy::{
    animation::{AnimatedBy, AnimationTargetId, animated_field},
    asset::uuid::Uuid,
    ecs::template::EntityTemplate,
    pbr::ExtendedMaterial,
    prelude::*,
};
use std::iter;

use crate::{
    app::Interpolation,
    drawings::{Drawing, DrawingMesh},
    rim_effect::RimEffect,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        //XXX observe state change event and play()
    }
}

#[derive(Component, Clone, Default)]
struct Player;

#[derive(Component, Reflect, Clone, FromTemplate)]
struct PlayerDrawing {
    material: Handle<StandardMaterial>,
    color: Oklaba,
}

fn play(
    mut commands: Commands,
    mut animations: ResMut<Assets<AnimationClip>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    extended_materials: Res<Assets<ExtendedMaterial<StandardMaterial, RimEffect>>>,
    standard_materials: ResMut<Assets<StandardMaterial>>,
    q_drawings: Query<(&Drawing, &Interpolation, &Children)>,
    q_transforms: Query<&Transform>,
    drawing_mesh: Res<DrawingMesh>,
) {
    //XXX spawn a single parent Player, for each pair of Drawing, zip_pad the children Transforms
    // then spawn children of Player with Mesh3d, material etc. and the initial Transform and an animation of the Transform
    // XXX also need PlayerDrawing parents to hold animated material shared by Mesh3d children

    let mut animation = AnimationClip::default();
    let mut scene_commands = commands.spawn_scene(bsn! {
        #Player
        Player
        @player_drawings(#Player, &mut animation, q_drawings, extended_materials, standard_materials,q_transforms, &drawing_mesh.0)
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
    q_transforms: Query<&Transform>,
    drawing_mesh: &Handle<Mesh>,
) -> impl Scene {
    let color_curve = EasingCurve::new(0.0, 1.0, EaseFunction::CubicInOut)
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

            let drawing_color_animation_target_id = AnimationTargetId(Uuid::new_v4());
            animation.add_curve_to_target(
                drawing_color_animation_target_id,
                AnimatableCurve::new(animated_field!(PlayerDrawing::color), color_curve.clone()),
            );

            let source_translations: Vec<_> = child_translations(source_children, &q_transforms);
            let target_translations: Vec<_> = child_translations(target_children, &q_transforms);
            let meshes = drawing_meshes(
                zip_pad(&source_translations, &target_translations),
                drawing_mesh,
                &material,
            );

            bsn! {
                PlayerDrawing { material, color: source_color }
                drawing_color_animation_target_id
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
    translations: impl Iterator<Item = (Vec3, Vec3)>,
    mesh: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
) -> impl SceneList {
    //XXX need animations, and to reference the player entity
    translations
        .map(|(source, _)| {
            let mesh = mesh.clone();
            let material = material.clone();
            bsn! {
                Mesh3d(mesh)
                MeshMaterial3d::<_>(material)
                Transform::from_translation(source)//XXX with_scale
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
