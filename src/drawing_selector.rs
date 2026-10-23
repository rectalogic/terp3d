use bevy::{
    feathers::{
        containers::{subpane, subpane_body, subpane_header},
        controls::{
            ColorInputValue, ColorSwatchValue, FeathersButton, FeathersColorSwatch,
            FeathersListRow, FeathersListView, OptionIndex,
        },
        display::caption,
        rounded_corners::RoundedCorners,
    },
    platform::collections::HashMap,
    prelude::*,
    ui::Selected,
    ui_widgets::{Activate, NumericValue, ValueChange, listbox_update_selection},
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

fn new_row(index: usize, drawing_configs: &HashMap<Interpolation, DrawingConfig>) -> impl Scene {
    let source_color = drawing_configs[&Interpolation::Source].color;
    let target_color = drawing_configs[&Interpolation::Target].color;

    bsn! {
        @FeathersListRow
        Node {
            justify_content: JustifyContent::SpaceEvenly,
        }
        Selected
        OptionIndex(index)
        Children[
            @FeathersColorSwatch {
                @corners: RoundedCorners::All,
            }
            ColorSwatchValue(source_color)
            --
            @FeathersColorSwatch {
                @corners: RoundedCorners::All,
            }
            ColorSwatchValue(target_color)
        ]
    }
}

#[expect(clippy::too_many_arguments)]
fn on_add_drawing(
    activate: On<Activate>,
    mut commands: Commands,
    q_listviews: Query<&TargetListView>,
    q_listrows: Query<(), With<FeathersListRow>>,
    q_children: Query<&Children>,
    q_childof: Query<&ChildOf>,
    q_color_inputs: Query<(&Interpolation, &ColorInputValue)>,
    q_number_inputs: Query<(&Interpolation, &NumericValue)>,
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
        let scales = q_number_inputs
            .iter()
            .map(|(interpolation, number_input)| {
                let scale = match number_input {
                    NumericValue::F32(scale) => *scale,
                    _ => 1.0,
                };
                (*interpolation, scale)
            })
            .collect::<HashMap<_, _>>();
        let drawing_configs = q_color_inputs
            .iter()
            .map(|(interpolation, color_input)| {
                (
                    *interpolation,
                    DrawingConfig {
                        color: color_input.0,
                        scale: scales[interpolation],
                    },
                )
            })
            .collect::<HashMap<_, _>>();

        let new_row = commands
            .spawn_scene(new_row(row_index, &drawing_configs))
            .id();
        commands
            .entity(listrow_parent.0)
            .insert_child(row_index, new_row);
        commands.trigger(AddDrawing { drawing_configs });
    }
}
