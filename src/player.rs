use bevy::{pbr::ExtendedMaterial, prelude::*};
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

#[derive(Component, Clone, FromTemplate)]
struct PlayerDrawing {
    material: Handle<ExtendedMaterial<StandardMaterial, RimEffect>>,
}

fn play(
    mut commands: Commands,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, RimEffect>>>,
    q_drawings: Query<(&Drawing, &Interpolation, &Children)>,
    q_transforms: Query<&Transform>,
    drawing_mesh: Res<DrawingMesh>,
) {
    //XXX spawn a single parent Player, for each pair of Drawing, zip_pad the children Transforms
    // then spawn children of Player with Mesh3d, material etc. and the initial Transform and an animation of the Transform
    // XXX also need PlayerDrawing parents to hold animated material shared by Mesh3d children

    let player_drawings: Vec<Box<dyn SceneList>> = iter::zip(
        iter_drawings(Interpolation::Source, &q_drawings),
        iter_drawings(Interpolation::Target, &q_drawings),
    )
    .map(
        |((source_drawing, source_children), (target_drawing, target_children))| {
            let target_material = materials
                .get(target_drawing.material.id())
                .expect("material");
            let source_material = materials
                .get(source_drawing.material.id())
                .expect("material");
            let source_material = source_material.clone();
            let source_color = source_material.base.base_color;
            let target_color = target_material.base.base_color;
            let material = materials.add(source_material);

            let source_translations: Vec<_> = child_translations(source_children, &q_transforms);
            let target_translations: Vec<_> = child_translations(target_children, &q_transforms);
            let meshes = drawing_meshes(
                zip_pad(&source_translations, &target_translations),
                &drawing_mesh.0,
                &material,
            );

            bsn! {
                    PlayerDrawing { material }
                    Children [

                    ]

            }
            .into()
        },
    )
    .collect();

    commands.spawn_scene(bsn! {
        Player
        Children [
            {player_drawings}
        ]
    });
}

fn drawing_meshes(
    translations: impl Iterator<Item = (Vec3, Vec3)>,
    mesh: &Handle<Mesh>,
    material: &Handle<ExtendedMaterial<StandardMaterial, RimEffect>>,
) -> impl SceneList {
    //XXX need animations, and to reference the player entity
    translations
        .map(|(source, _)| {
            let mesh = mesh.clone();
            let material = material.clone();
            bsn! {
                Mesh3d(mesh)
                MeshMaterial3d::<ExtendedMaterial<StandardMaterial, RimEffect>>(material)
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
