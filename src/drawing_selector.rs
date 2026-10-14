use bevy::{
    feathers::{
        controls::{FeathersListRow, FeathersSelect, OptionIndex},
        display::{caption, label},
    },
    prelude::*,
    ui::Selected,
};

#[derive(SceneComponent, Clone, Default, Debug)]
#[scene(DrawingSelectorProps)]
pub struct DrawingSelector;

#[derive(Default)]
pub struct DrawingSelectorProps {
    pub count: usize,
    pub selected: Option<usize>,
}

impl DrawingSelector {
    fn scene(props: DrawingSelectorProps) -> impl Scene {
        bsn! {
            Node {
                align_self: AlignSelf::FlexStart,
                padding: UiRect::axes(px(5), px(1)),
            }
            Children [
                @label("Drawing")
                --
                @FeathersSelect {
                    @options: {
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
                    },
                    @max_visible: 6,
                }
            ]
        }
    }
}
