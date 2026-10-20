use bevy::{
    camera::visibility::RenderLayers, material::OpaqueRendererMethod, pbr::ExtendedMaterial,
    prelude::*,
};

use crate::rim_effect::{RimEffect, RimEffectPlugin};

pub struct DrawingsPlugin;

impl Plugin for DrawingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RimEffectPlugin)
            .init_resource::<DrawingMesh>()
            .add_systems(Startup, setup)
            .add_observer(on_add_drawing)
            .add_observer(on_select_drawing)
            .add_observer(on_drawing_selected)
            .add_observer(on_drawing_deselected)
            .add_observer(on_draw);
    }
}

#[derive(Resource, Default)]
struct DrawingMesh(Handle<Mesh>);

#[derive(Event)]
pub struct AddDrawing;

#[derive(Event)]
pub struct SelectDrawing {
    pub index: usize,
}

#[derive(Component, Clone, Default)]
#[require(Transform)]
pub enum DrawingContainer {
    #[default]
    Source,
    Target,
}

#[derive(Component, Clone, Default)]
struct SelectedDrawing;

#[derive(Component, Clone, FromTemplate)]
#[require(Transform)]
pub struct Drawing {
    pub material: Handle<ExtendedMaterial<StandardMaterial, RimEffect>>,
    //XXX scale
}

fn setup(mut meshes: ResMut<Assets<Mesh>>, mut drawing_mesh: ResMut<DrawingMesh>) {
    drawing_mesh.0 = meshes.add(Sphere::new(0.5).mesh());
}

fn deselect_drawings(commands: &mut Commands, drawings: &Children) {
    drawings.iter().for_each(|drawing| {
        commands.entity(drawing).remove::<SelectedDrawing>();
    });
}

fn on_add_drawing(
    _add: On<AddDrawing>,
    mut commands: Commands,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, RimEffect>>>,
    q_drawing_containers: Query<(Entity, &RenderLayers), With<DrawingContainer>>,
    q_children: Query<&Children>,
) {
    q_drawing_containers
        .iter()
        .for_each(|(drawing_container, render_layers)| {
            if let Ok(drawings) = q_children.get(drawing_container) {
                deselect_drawings(&mut commands, drawings);
            }

            //XXX get color from color selector, and scale (stored in Resources)
            let material = materials.add(ExtendedMaterial {
                base: StandardMaterial {
                    base_color: Srgba::new(1.0, 0.0, 0.0, 1.0).into(),
                    opaque_render_method: OpaqueRendererMethod::Forward,
                    ..Default::default()
                },
                extension: RimEffect::default(),
            });
            let render_layers = render_layers.clone();
            let drawing = commands
                .spawn_scene(bsn! {
                    Drawing { material }
                    render_layers
                    SelectedDrawing
                })
                .id();
            commands.entity(drawing_container).add_child(drawing);
        });
}

//XXX placeholder, should respond to mouse events and add meshes
fn on_draw(
    add: On<Add<Drawing>>,
    mut commands: Commands,
    drawing_mesh: Res<DrawingMesh>,
    q_drawings: Query<(&Drawing, &RenderLayers)>,
) {
    if let Ok((drawing, render_layers)) = q_drawings.get(add.entity) {
        let mesh = drawing_mesh.0.clone();
        let material = drawing.material.clone();
        let transform = Transform::from_xyz(
            rand::random_range(-3.0..3.0),
            rand::random_range(-3.0..3.0),
            rand::random_range(-3.0..3.0),
        );
        let render_layers = render_layers.clone();
        let entity = commands
            .spawn_scene(bsn! {
                Mesh3d(mesh)
                MeshMaterial3d::<ExtendedMaterial<StandardMaterial, RimEffect>>(material)
                transform
                render_layers
            })
            .id();
        commands.entity(add.entity).add_child(entity);
    }
}

fn on_select_drawing(
    select: On<SelectDrawing>,
    mut commands: Commands,
    q_drawing_containers: Query<Entity, With<DrawingContainer>>,
    q_children: Query<&Children>,
) {
    q_drawing_containers.iter().for_each(|drawing_container| {
        if let Ok(drawings) = q_children.get(drawing_container) {
            deselect_drawings(&mut commands, drawings);
            commands
                .entity(drawings[select.index])
                .insert(SelectedDrawing);
        }
    });
}

fn on_drawing_selected(
    add: On<Add<SelectedDrawing>>,
    q_drawings: Query<&Drawing>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, RimEffect>>>,
) {
    if let Ok(drawing) = q_drawings.get(add.entity)
        && let Some(mut material) = materials.get_mut(drawing.material.id())
    {
        material.extension.disable(false);
    }
}

fn on_drawing_deselected(
    remove: On<Remove<SelectedDrawing>>,
    q_drawings: Query<&Drawing>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, RimEffect>>>,
) {
    if let Ok(drawing) = q_drawings.get(remove.entity)
        && let Some(mut material) = materials.get_mut(drawing.material.id())
    {
        material.extension.disable(true);
    }
}
