use bevy::{
    camera::visibility::RenderLayers, feathers::controls::ColorInputValue,
    material::OpaqueRendererMethod, pbr::ExtendedMaterial, prelude::*, ui_widgets::ValueChange,
};

use crate::{
    app::Interpolation,
    rim_effect::{RimEffect, RimEffectPlugin},
};

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
            .add_observer(on_draw)
            .add_observer(on_drawing_color_change);
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
pub struct DrawingContainer;

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
    q_drawing_containers: Query<(Entity, &RenderLayers, &Interpolation), With<DrawingContainer>>,
    q_color_inputs: Query<(&ColorInputValue, &Interpolation)>,
    q_children: Query<&Children>,
) {
    q_drawing_containers
        .iter()
        .for_each(|(drawing_container, render_layers, interpolation)| {
            if let Ok(drawings) = q_children.get(drawing_container) {
                deselect_drawings(&mut commands, drawings);
            }

            if let Some(current_color) =
                q_color_inputs
                    .iter()
                    .find_map(|(color_input, color_interpolation)| {
                        (color_interpolation == interpolation).then_some(color_input.0)
                    })
            {
                let material = materials.add(ExtendedMaterial {
                    base: StandardMaterial {
                        base_color: current_color,
                        opaque_render_method: OpaqueRendererMethod::Forward,
                        ..Default::default()
                    },
                    extension: RimEffect::default(),
                });
                let render_layers = render_layers.clone();
                let interpolation = *interpolation;
                let drawing = commands
                    .spawn_scene(bsn! {
                        Drawing { material }
                        render_layers
                        interpolation
                        SelectedDrawing
                    })
                    .id();
                commands.entity(drawing_container).add_child(drawing);
            }
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

fn on_drawing_color_change(
    value_change: On<ValueChange<Color>>,
    q_interpolations: Query<&Interpolation>,
    q_selected_drawings: Query<(&Drawing, &Interpolation), With<SelectedDrawing>>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, RimEffect>>>,
) {
    if let Ok(color_interpolation) = q_interpolations.get(value_change.source) {
        q_selected_drawings
            .iter()
            .filter_map(|(selected_drawing, interpolation)| {
                (interpolation == color_interpolation).then_some(selected_drawing)
            })
            .for_each(|selected_drawing| {
                if let Some(mut material) = materials.get_mut(selected_drawing.material.id()) {
                    material.base.base_color = value_change.value;
                }
            });
    }
}
