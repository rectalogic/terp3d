use bevy::{
    feathers::{
        containers::{pane, pane_body, pane_header},
        controls::{
            ButtonVariant, ColorInputValue, ColorSwatchValue, FeathersButton, FeathersColorInput,
            FeathersColorSwatch, FeathersDialog, FeathersDialogBody, FeathersDialogClose,
            FeathersDialogFooter, FeathersDialogHeader, FeathersListRow, FeathersListView,
            FeathersNumberInput, NumberInputPrecision, NumberInputStep, NumberInputValue,
            OptionIndex, SoftLimit, color_input_self_update,
        },
        display::{caption, label},
        rounded_corners::RoundedCorners,
    },
    platform::collections::HashMap,
    prelude::*,
    ui::Selected,
    ui_widgets::{Activate, RequestClose, ValueChange, listbox_update_selection},
};

use crate::{
    app::Interpolation,
    drawings::{AddDrawing, DrawingConfig, SelectDrawing},
};

#[derive(SceneComponent, Clone, Default, Debug)]
pub struct DrawingSelector;

#[derive(Component, Clone, FromTemplate)]
struct TargetListView(Entity);

#[derive(Component, Clone, Default)]
pub struct DrawingColor(pub Color);

#[derive(Component, Clone, Default)]
pub struct DrawingScale(pub f32);

impl DrawingSelector {
    fn scene() -> impl Scene {
        bsn! {
            @pane() Children [
                @pane_header() Children [
                    @caption("Drawings")
                ]
                --
                @pane_body() Children [
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
                    --
                    Interpolation::Source
                    DrawingColor
                    DrawingScale(3.0)
                    --
                    Interpolation::Target
                    DrawingColor
                    DrawingScale(3.0)
                    --
                    @FeathersButton {
                        @caption: bsn! { @caption("New") }
                    }
                    on(spawn_new_drawing_dialog)
                ]
            ]
        }
    }
}

fn spawn_new_drawing_dialog(
    activate: On<Activate>,
    mut commands: Commands,
    q_drawing_colors: Query<(&Interpolation, &DrawingColor)>,
    q_drawing_scales: Query<(&Interpolation, &DrawingScale)>,
) {
    let drawing_configs: HashMap<Interpolation, DrawingConfig> = q_drawing_colors
        .iter()
        .map(|(interpolation, drawing_color)| {
            let scale = q_drawing_scales
                .iter()
                .find_map(|(interp, scale)| (interp == interpolation).then_some(scale.0))
                .unwrap();
            (
                *interpolation,
                DrawingConfig {
                    color: drawing_color.0,
                    scale,
                },
            )
        })
        .collect();

    commands
        .entity(activate.event_target())
        .queue_spawn_related_scenes::<Children>(bsn_list! {
            @FeathersDialog {
                @width: px(320),
                @contents: bsn_list! {
                    @FeathersDialogHeader Children [
                        @caption("New Drawing")
                        --
                        @FeathersDialogClose
                    ]
                    --
                    @FeathersDialogBody Children [
                        @drawing_configurator(Interpolation::Source, &drawing_configs)
                        --
                        @drawing_configurator(Interpolation::Target, &drawing_configs)
                    ]
                    --
                    @FeathersDialogFooter Children [
                        @FeathersButton {
                            @caption: bsn! { @caption("Cancel") },
                        }
                        AccessibleLabel("Cancel")
                        on(|activate: On<Activate>, mut commands: Commands| {
                            commands.trigger(RequestClose { source: activate.event_target() });
                        })
                        --
                        @FeathersButton {
                            @caption: bsn! { @caption("Create") },
                            @variant: ButtonVariant::Primary,
                        }
                        AccessibleLabel("Create")
                        TargetListView(#ListView)
                        on(on_add_drawing)
                    ]
                }
            }
            on(|close: On<RequestClose>, mut commands: Commands| {
                commands.entity(close.event_target()).despawn();
            })
        });
}

fn drawing_configurator(
    interpolation: Interpolation,
    drawing_configs: &HashMap<Interpolation, DrawingConfig>,
) -> impl Scene {
    let (title, color, scale) = match interpolation {
        Interpolation::Source => (
            "Source",
            drawing_configs[&Interpolation::Source].color,
            drawing_configs[&Interpolation::Source].scale,
        ),
        Interpolation::Target => (
            "Target",
            drawing_configs[&Interpolation::Target].color,
            drawing_configs[&Interpolation::Target].scale,
        ),
    };
    bsn! {
        @pane() Children [
            @pane_header() Children [
                @caption(title)
            ]
            --
            @pane_body() Children [
                Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::Baseline,
                }
                Children [
                    @label("Color")
                    --
                    @FeathersColorInput
                    ColorInputValue(color)
                    interpolation
                    on(color_input_self_update)
                    --
                    @label("Size")
                    --
                    @FeathersNumberInput
                    Node {
                        width: px(100),
                        max_width: px(100),
                    }
                    NumberInputValue::F32(scale)
                    interpolation
                    NumberInputPrecision(0)
                    NumberInputStep(1.0)
                    SoftLimit::f32(1.0..=20.0)
                    on(|value_change: On<ValueChange<f32>>, mut commands: Commands| {
                        commands.entity(value_change.event_target())
                            .insert(NumberInputValue::F32(value_change.value));
                    })
                ]
            ]
        ]
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
    q_number_inputs: Query<(&Interpolation, &NumberInputValue)>,
    mut q_drawing_colors: Query<(&Interpolation, &mut DrawingColor)>,
    mut q_drawing_scales: Query<(&Interpolation, &mut DrawingScale)>,
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
                    NumberInputValue::F32(scale) => *scale,
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

        q_drawing_colors
            .iter_mut()
            .for_each(|(interpolation, mut drawing_color)| {
                drawing_color.0 = drawing_configs[interpolation].color;
            });
        q_drawing_scales
            .iter_mut()
            .for_each(|(interpolation, mut drawing_scale)| {
                drawing_scale.0 = drawing_configs[interpolation].scale;
            });

        commands.trigger(AddDrawing { drawing_configs });

        commands.trigger(RequestClose {
            source: activate.event_target(),
        });
    }
}
