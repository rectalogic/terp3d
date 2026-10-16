use bevy::{
    feathers::{
        containers::{subpane, subpane_body, subpane_header},
        controls::{FeathersButton, FeathersListRow, FeathersListView, OptionIndex},
        display::caption,
    },
    prelude::*,
    ui::Selected,
    ui_widgets::{Activate, ValueChange, listbox_update_selection},
};

use crate::drawings::{AddDrawing, SelectDrawing};

pub struct DrawingSelectorPlugin;

impl Plugin for DrawingSelectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
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
                        @rows: { listview_rows(1) }
                    }
                    on(listbox_update_selection)
                    on(listbox_select)
                ]
            ]
        }
    }
}

fn setup(mut commands: Commands) {
    // We initialize the list with one entry
    commands.trigger(AddDrawing);
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

fn listview_rows(count: usize) -> Box<dyn SceneList> {
    Box::new(
        (0..count)
            .map(|i| -> Box<dyn Scene> { listrow(i, i == count - 1) })
            .collect::<Vec<_>>(),
    ) as Box<dyn SceneList>
}

fn on_add_drawing(
    activate: On<Activate>,
    mut commands: Commands,
    q_listviews: Query<&TargetListView>,
    q_listrows: Query<(), With<FeathersListRow>>,
    q_children: Query<&Children>,
    q_childof: Query<&ChildOf>,
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
        let new_row = commands.spawn_scene(listrow(row_index + 1, true)).id();
        commands.entity(listrow_parent.0).add_child(new_row);
    }

    commands.trigger(AddDrawing);
}
