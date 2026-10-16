use bevy::prelude::*;

pub struct DrawingsPlugin;

impl Plugin for DrawingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_add_drawing)
            .add_observer(on_select_drawing);
    }
}

#[derive(Event)]
pub struct AddDrawing;

#[derive(Event)]
pub struct SelectDrawing {
    pub index: usize,
}

#[derive(Component, Clone, Default)]
pub enum DrawingContainer {
    #[default]
    Source,
    Target,
}

#[derive(Component, Clone, Default)]
struct SelectedDrawing;

#[derive(Component, Clone, FromTemplate)]
pub struct Drawing {
    pub material: Handle<StandardMaterial>,
    //XXX scale
}

fn deselect_drawings(commands: &mut Commands, drawings: &Children) {
    drawings.iter().for_each(|drawing| {
        commands.entity(drawing).remove::<SelectedDrawing>();
    });
}

fn on_add_drawing(
    _add: On<AddDrawing>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    q_drawing_containers: Query<Entity, With<DrawingContainer>>,
    q_children: Query<&Children>,
) {
    q_drawing_containers.iter().for_each(|drawing_container| {
        if let Ok(drawings) = q_children.get(drawing_container) {
            deselect_drawings(&mut commands, drawings);
        }

        //XXX get color from color selector, and scale (stored in Resources)
        let material = materials.add(StandardMaterial {
            base_color: Srgba::new(1.0, 0.0, 0.0, 1.0).into(),
            ..default()
        });
        let drawing = commands
            .spawn_scene(bsn! {
                Drawing { material }
                SelectedDrawing
            })
            .id();
        commands.entity(drawing_container).add_child(drawing);
    });
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
            //XXX modify Drawing material to hilite selected (and modify when deselected) - do in separate On<Add>?
        }
    });
}
