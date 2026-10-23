use bevy::{
    feathers::{
        containers::{subpane, subpane_body, subpane_header},
        controls::{
            ColorInputValue, FeathersButton, FeathersListRow, FeathersListView, OptionIndex,
        },
        display::caption,
    },
    platform::collections::HashMap,
    prelude::*,
    ui::Selected,
    ui_widgets::{Activate, ValueChange, listbox_update_selection},
};

use crate::{
    app::Interpolation,
    drawings::{AddDrawing, DrawingConfig, SelectDrawing},
};

pub struct DrawingSelectorPlugin;

impl Plugin for DrawingSelectorPlugin {
    fn build(&self, _app: &mut App) {
        //XXX remove if unused
    }
}

#[derive(SceneComponent, Clone, Default, Debug)]
pub struct DrawingSelector;

#[derive(Component, Clone, FromTemplate)]
struct TargetListView(Entity);

impl DrawingSelector {
    fn scene() -> impl Scene {
        bsn! {
            @subpane() Children [
                @subpane_header() Children [
                    @caption("Drawings")
                ]
                --
                @subpane_header() Children [
                    @FeathersButton {
                        @caption: bsn! { @caption("New") }
                    }
                    TargetListView(#ListView)
                    on(on_add_drawing)
                ]
                --
                @subpane_body() Children [
                    #ListView
                    Node {
                        height: em(6),
                        max_height: em(6),
                    }
                    @FeathersListView {
                        @rows: { bsn! {
                            @FeathersListRow
                            Node {
                                display: Display::None
                            }
                        } }
                    }
                    on(listbox_update_selection)
                    on(listbox_select)
                ]
            ]
        }
    }
}

fn listbox_select(
    value_change: On<ValueChange<Entity>>,
    mut commands: Commands,
    q_option_indexes: Query<&OptionIndex>,
) {
    let change = value_change.event();
    let row = change.value;
    if let Ok(index) = q_option_indexes.get(row) {
        commands.trigger(SelectDrawing { index: index.0 });
    }
}

fn listrow(index: usize, selected: bool) -> Box<dyn Scene> {
    let label = index.to_string();
    if selected {
        bsn! { @FeathersListRow Selected OptionIndex(index) Children[@caption(label)] }.into()
    } else {
        bsn! { @FeathersListRow OptionIndex(index) Children[@caption(label)] }.into()
    }
}

// fn on_add_drawing(
//     _add: On<AddDrawing>,
//     mut commands: Commands,
//     mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, RimEffect>>>,
//     q_drawing_containers: Query<(Entity, &RenderLayers, &Interpolation), With<DrawingContainer>>,
//     q_color_inputs: Query<(&ColorInputValue, &Interpolation)>,
//     q_children: Query<&Children>,
// ) {
//     q_drawing_containers
//         .iter()
//         .for_each(|(drawing_container, render_layers, interpolation)| {
//             if let Ok(drawings) = q_children.get(drawing_container) {
//                 deselect_drawings(&mut commands, drawings);
//             }
//
//             if let Some(current_color) =
//                 q_color_inputs
//                     .iter()
//                     .find_map(|(color_input, color_interpolation)| {
//                         (color_interpolation == interpolation).then_some(color_input.0)
//                     })
//             {}
//         });
// }
fn on_add_drawing(
    activate: On<Activate>,
    mut commands: Commands,
    q_listviews: Query<&TargetListView>,
    q_listrows: Query<(), With<FeathersListRow>>,
    q_children: Query<&Children>,
    q_childof: Query<&ChildOf>,
    q_color_inputs: Query<(&Interpolation, &ColorInputValue)>,
) {
    let Ok(target_listview) = q_listviews.get(activate.entity) else {
        return;
    };
    if let Some((Ok(listrow_parent), row_index)) = q_children
        .iter_descendants(target_listview.0)
        .filter(|descendent| q_listrows.contains(*descendent))
        .enumerate()
        .map(|(index, listrow)| {
            commands.entity(listrow).remove::<Selected>();
            (q_childof.get(listrow), index)
        })
        .last()
    {
        let drawing_configs = q_color_inputs
            .iter()
            .map(|(interpolation, color_input)| {
                (
                    *interpolation,
                    DrawingConfig {
                        color: color_input.0,
                        scale: 1.0,
                    },
                )
            })
            .collect::<HashMap<_, _>>();

        //XXX show colors in listrow
        let new_row = commands.spawn_scene(listrow(row_index, true)).id();
        commands
            .entity(listrow_parent.0)
            .insert_child(row_index, new_row);
        commands.trigger(AddDrawing { drawing_configs });
    }
}
