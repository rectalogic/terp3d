use bevy::{
    feathers::{
        containers::{subpane, subpane_body, subpane_header},
        controls::{FeathersListRow, FeathersListView, OptionIndex},
        display::caption,
    },
    prelude::*,
    ui::Selected,
    ui_widgets::listbox_update_selection,
};

#[derive(SceneComponent, Clone, Default, Debug)]
#[scene(DrawingSelectorProps)]
pub struct DrawingSelector;

#[derive(Default)]
pub struct DrawingSelectorProps {
    pub count: usize,
    pub selected: Option<usize>,
}

// XXX add +- buttons and rebuild when adding/removing - and select new item
impl DrawingSelector {
    fn scene(props: DrawingSelectorProps) -> impl Scene {
        bsn! {
            @subpane() Children [
                @subpane_header() Children [
                    @caption("Drawings")
                ]
                --
                @subpane_body() Children [
                    Node {
                        max_height: px(130)
                    }
                    @FeathersListView {
                        @rows: {
                            Box::new(
                                (0..=props.count)
                                    .map(|i| -> Box<dyn SceneList> {
                                        if i == 0 {
                                            if props.selected.is_none() {
                                                bsn! { @FeathersListRow Selected Children[@caption("None")] }.into()
                                            } else {
                                                bsn! { @FeathersListRow Children[@caption("None")] }.into()
                                            }
                                        } else {
                                            let index = i - 1;
                                            let label = i.to_string();
                                            if Some(index) == props.selected {
                                                bsn! { @FeathersListRow Selected OptionIndex(index) Children[@caption(label)] }.into()
                                            } else {
                                                bsn! { @FeathersListRow OptionIndex(index) Children[@caption(label)] }.into()
                                            }
                                        }
                                    })
                                    .collect::<Vec<_>>(),
                            ) as Box<dyn SceneList>
                        }
                    }
                    on(listbox_update_selection)
                ]
            ]
        }
    }
}
