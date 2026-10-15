use bevy::{
    feathers::{
        containers::{flex_spacer, subpane, subpane_body, subpane_header},
        controls::{FeathersButton, FeathersListRow, FeathersListView, OptionIndex},
        display::caption,
    },
    prelude::*,
    ui::Selected,
    ui_widgets::{Activate, listbox_update_selection},
};

#[derive(SceneComponent, Clone, Default, Debug)]
#[scene(DrawingSelectorProps)]
pub struct DrawingSelector;

#[derive(Default)]
pub struct DrawingSelectorProps {
    pub count: usize,
    pub selected: Option<usize>,
}

#[derive(Component, Clone, FromTemplate)]
struct TargetListView(Entity);

impl DrawingSelector {
    fn scene(props: DrawingSelectorProps) -> impl Scene {
        bsn! {
            @subpane() Children [
                @subpane_header() Children [
                    @caption("Drawings")
                ]
                --
                @subpane_header() Children [
                    @FeathersButton {
                        @caption: bsn! { @caption("-") }
                    }
                    TargetListView(#ListView)
                    on(on_remove_drawing)
                    --
                    @flex_spacer()
                    --
                    @FeathersButton {
                        @caption: bsn! { @caption("+") }
                    }
                    TargetListView(#ListView)
                    on(on_add_drawing)
                ]
                --
                @subpane_body() Children [
                    #ListView
                    Node {
                        max_height: px(130)
                    }
                    @FeathersListView {
                        @rows: { listview_rows(props) }
                    }
                    on(listbox_update_selection)
                ]
            ]
        }
    }
}

fn listrow(index: usize, label: String, selected: bool) -> Box<dyn Scene> {
    if selected {
        bsn! { @FeathersListRow Selected OptionIndex(index) Children[@caption(label)] }.into()
    } else {
        bsn! { @FeathersListRow OptionIndex(index) Children[@caption(label)] }.into()
    }
}

fn listview_rows(props: DrawingSelectorProps) -> Box<dyn SceneList> {
    Box::new(
        (0..=props.count)
            .map(|i| -> Box<dyn Scene> {
                if i == 0 {
                    if props.selected.is_none() {
                        bsn! { @FeathersListRow Selected Children[@caption("None")] }.into()
                    } else {
                        bsn! { @FeathersListRow Children[@caption("None")] }.into()
                    }
                } else {
                    let index = i - 1;
                    listrow(index, i.to_string(), Some(index) == props.selected)
                }
            })
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
        let new_row = commands
            .spawn_scene(listrow(row_index, (row_index + 1).to_string(), true))
            .id();
        commands.entity(listrow_parent.0).add_child(new_row);
    }

    //XXX fire selection event for row_index-1
}

fn on_remove_drawing(_activate: On<Activate>) {}
