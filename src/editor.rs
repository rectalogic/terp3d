use bevy::{
    asset::asset_value,
    camera::{ImageRenderTargetTemplate, RenderTarget, visibility::RenderLayers},
    ecs::template::OptionTemplate,
    feathers::{
        controls::{
            ColorInputValue, FeathersColorInput, FeathersNumberInput, HardLimit,
            NumberInputPrecision, NumberInputStep, SoftLimit, color_input_self_update,
        },
        display::label,
    },
    prelude::*,
    render::render_resource::TextureFormat,
    ui::widget::ViewportNode,
    ui_widgets::{NumericValue, ValueChange},
};

use crate::{
    app::Interpolation,
    camera_controller::{CameraController, CameraControllerPlugin, ZoomFinished},
};

const EDIT_DISTANCE: f32 = 5.0;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CameraControllerPlugin);
    }

    fn finish(&self, app: &mut App) {
        if let Some(mut gizmo_assets) = app.world_mut().get_resource_mut::<Assets<GizmoAsset>>() {
            // Drawing plane
            let mut edit_grid_gizmo_asset = GizmoAsset::default();
            edit_grid_gizmo_asset
                .grid(
                    Isometry3d::IDENTITY,
                    UVec2::splat(7),
                    Vec2::splat(0.5),
                    Srgba::new(1.0, 1.0, 0.0, 0.3),
                )
                .outer_edges();
            let gizmo_handle = gizmo_assets.add(edit_grid_gizmo_asset);
            app.insert_resource(EditGridAsset(gizmo_handle));
        }
    }
}

#[derive(SceneComponent, Default, Clone)]
#[scene(DrawingEditorProps)]
#[require(RenderLayers)]
pub struct DrawingEditor;

#[derive(Default)]
pub struct DrawingEditorProps {
    pub interpolation: Interpolation,
    pub render_layers: RenderLayers,
}

#[derive(Component)]
#[relationship(relationship_target = LinkedCamera)]
struct CameraLink(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = CameraLink, linked_spawn)]
struct LinkedCamera(Entity);

#[derive(Resource, Clone, Default)]
struct EditGridAsset(Handle<GizmoAsset>);

#[derive(Component, Clone, Default)]
struct EditGrid;

#[derive(Component, Clone, FromTemplate)]
struct LinkedEditGrid(Entity);

impl DrawingEditor {
    fn scene(props: DrawingEditorProps) -> impl Scene {
        let image = Image::new_target_texture(0, 0, TextureFormat::Bgra8UnormSrgb, None);
        let render_layers = props.render_layers.clone();

        bsn! {
            Node {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
            }
            Children [
                Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::Baseline,
                }
                Children [
                    @label("Color")
                    --
                    @FeathersColorInput
                    ColorInputValue
                    props.interpolation
                    on(color_input_self_update)
                    --
                    @label("Size")
                    --
                    @FeathersNumberInput
                    Node {
                        max_width: px(100),
                    }
                    NumericValue::F32(1.0)
                    props.interpolation
                    NumberInputPrecision(1)
                    NumberInputStep(0.1)
                    SoftLimit::f32(0.1..=3.0)
                    HardLimit::f32(0.1..=3.0)
                    on(|value_change: On<ValueChange<f32>>, mut commands: Commands| {
                        commands.entity(value_change.event_target())
                            .insert(NumericValue::F32((value_change.value * 10.0).round() / 10.0));
                    })
                ]
                --
                Node {
                    flex_grow: 1.0,
                }
                ViewportNode { camera: OptionTemplate::Some(#Camera) }
                CameraController { camera: OptionTemplate::Some(#Camera) }
                LinkedEditGrid(#EditGrid)
                on(on_editor_pointer_drag_start)
                on(on_editor_pointer_drag)
                on(on_editor_pointer_drag_end)
                on(on_editor_pointer_scroll)
                on(on_editor_zoom_finished)
                LinkedCamera [
                    #Camera
                    Camera3d
                    Camera { order: -1 }
                    Transform::from_xyz(0., 0., 10.0).looking_at(Vec3::ZERO, Vec3::Y)
                    props.render_layers
                    RenderTarget::Image(ImageRenderTargetTemplate {
                        handle: asset_value(image),
                        scale_factor: 1.0, //XXX default is 0, panics
                    })
                    Children [
                        #EditGrid
                        EditGrid
                        template(|ctx| {
                            Ok(Gizmo {
                                handle: ctx.resource::<EditGridAsset>().0.clone(),
                                line_config: GizmoLineConfig {
                                    width: 3.0,
                                    ..default()
                                },
                                ..default()
                            })
                        })
                        render_layers
                        Visibility::Hidden //XXX doesn't work
                        Transform::from_xyz(0., 0., -EDIT_DISTANCE)
                    ]
                ]
            ]
        }
    }
}

fn on_editor_pointer_drag_start(
    dragstart: On<PointerDragStart>,
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    q_linked_edit_grids: Query<&LinkedEditGrid>,
) {
    if input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
        && let Ok(linked_edit_grid) = q_linked_edit_grids.get(dragstart.entity)
    {
        commands
            .entity(linked_edit_grid.0)
            .insert(Visibility::Inherited);
    }
}

fn on_editor_pointer_drag(
    drag: On<PointerDrag>,
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    q_viewport_nodes: Query<&ViewportNode>,
    q_cameras: Query<(&Camera, &GlobalTransform)>,
    q_linked_edit_grids: Query<&LinkedEditGrid>,
) {
    if input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
        && let Ok(linked_edit_grid) = q_linked_edit_grids.get(drag.entity)
        && let Ok(viewport_node) = q_viewport_nodes.get(drag.entity)
        && let Some(camera_entity) = viewport_node.camera
        && let Ok((camera, camera_transform)) = q_cameras.get(camera_entity)
        && let Ok(ray) = camera.viewport_to_world(camera_transform, drag.pointer.position)
    {
        commands
            .entity(linked_edit_grid.0)
            .insert(Visibility::Inherited);

        let forward = camera_transform.forward();
        let plane_origin = camera_transform.translation() + forward * EDIT_DISTANCE;
        let plane = InfinitePlane3d::new(forward);
        if let Some(ray_distance) = ray.intersect_plane(plane_origin, plane) {
            let point = ray.get_point(ray_distance);
            info!("drag {point:?}"); //XXX
            //XXX trigger event with world coordinates
        };
    }
}

fn on_editor_pointer_drag_end(
    dragend: On<PointerDragEnd>,
    mut commands: Commands,
    q_linked_edit_grids: Query<&LinkedEditGrid>,
) {
    if let Ok(linked_edit_grid) = q_linked_edit_grids.get(dragend.entity) {
        commands
            .entity(linked_edit_grid.0)
            .insert(Visibility::Hidden);
    }
}

fn on_editor_pointer_scroll(
    scroll: On<PointerScroll>,
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    q_viewport_nodes: Query<&ViewportNode>,
    q_cameras: Query<(&Camera, &GlobalTransform)>,
    q_linked_edit_grids: Query<&LinkedEditGrid>,
) {
    if input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
        && let Ok(linked_edit_grid) = q_linked_edit_grids.get(scroll.entity)
        && let Ok(viewport_node) = q_viewport_nodes.get(scroll.entity)
        && let Some(camera_entity) = viewport_node.camera
        && let Ok((camera, camera_transform)) = q_cameras.get(camera_entity)
        && let Ok(ray) = camera.viewport_to_world(camera_transform, scroll.pointer.position)
    {
        commands
            .entity(linked_edit_grid.0)
            .insert(Visibility::Inherited);

        let forward = camera_transform.forward();
        let plane_origin = camera_transform.translation() + forward * EDIT_DISTANCE;
        let plane = InfinitePlane3d::new(forward);
        if let Some(ray_distance) = ray.intersect_plane(plane_origin, plane) {
            let point = ray.get_point(ray_distance);
            info!("scroll {point:?}"); //XXX
            //XXX trigger event with world coordinates
        };
    }
}

fn on_editor_zoom_finished(
    zoom: On<ZoomFinished>,
    mut commands: Commands,
    q_linked_edit_grids: Query<&LinkedEditGrid>,
) {
    if let Ok(linked_edit_grid) = q_linked_edit_grids.get(zoom.entity) {
        commands
            .entity(linked_edit_grid.0)
            .insert(Visibility::Hidden);
    }
}
