use bevy::{
    asset::asset_value,
    camera::{ImageRenderTargetTemplate, RenderTarget, visibility::RenderLayers},
    ecs::template::OptionTemplate,
    feathers::{
        containers::{pane, pane_header},
        display::label_dim,
    },
    picking::{hover::Hovered, pointer::PointerLocation},
    prelude::*,
    render::render_resource::TextureFormat,
    ui::widget::ViewportNode,
};

use crate::{
    app::AppState,
    camera_controller::{CameraController, CameraControllerPlugin},
    interpolation::Interpolation,
    skybox::SkyboxCamera,
};

const EDIT_DISTANCE: f32 = 8.0;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CameraControllerPlugin).add_systems(
            Update,
            handle_edit_grid_visibility.run_if(in_state(AppState::Edit)),
        );
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

#[derive(Component, Clone, Default)]
pub struct EditorUI;

#[derive(Event)]
pub struct EditDrawing {
    pub interpolation: Interpolation,
    pub origin: Vec3,
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
        let title = match props.interpolation {
            Interpolation::Source => "Source",
            Interpolation::Target => "Target",
        };

        bsn! {
            @pane() Children [
                @pane_header() Children [
                    @label_dim(title)
                ]
                --
                Node {
                    flex_grow: 1.0,
                }
                Hovered
                props.interpolation
                ViewportNode { camera: OptionTemplate::Some(#Camera) }
                @CameraController { @camera: #Camera }
                LinkedEditGrid(#EditGrid)
                on(on_editor_pointer_drag)
                on(on_editor_pointer_drag_end)
                on(on_editor_pointer_scroll)
                LinkedCamera [
                    #Camera
                    @SkyboxCamera
                    Camera { order: -1 }
                    props.interpolation
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
                        Visibility::Hidden
                        Transform::from_xyz(0., 0., -EDIT_DISTANCE)
                    ]
                ]
            ]
        }
    }
}

fn send_edit_draw_events<E: PointerEvent>(
    event: &E,
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    q_viewport_nodes: Query<(&ViewportNode, &Interpolation, &PointerLocation)>,
    q_cameras: Query<(&Camera, &GlobalTransform)>,
) {
    if input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
        && let Ok((viewport_node, interpolation, pointer_location)) =
            q_viewport_nodes.get(event.event_target())
        && let Some(camera_entity) = viewport_node.camera
        && let Some(location) = pointer_location.location()
        && let Ok((camera, camera_transform)) = q_cameras.get(camera_entity)
        && let Ok(ray) = camera.viewport_to_world(camera_transform, location.position)
    {
        let forward = camera_transform.forward();
        let plane_origin = camera_transform.translation() + forward * EDIT_DISTANCE;
        let plane = InfinitePlane3d::new(forward);
        if let Some(ray_distance) = ray.intersect_plane(plane_origin, plane) {
            let point = ray.get_point(ray_distance);
            commands.trigger(EditDrawing {
                interpolation: *interpolation,
                origin: point,
            });
        };
    }
}

fn on_editor_pointer_drag(
    drag: On<PointerDrag>,
    input: Res<ButtonInput<KeyCode>>,
    commands: Commands,
    q_viewport_nodes: Query<(&ViewportNode, &Interpolation, &PointerLocation)>,
    q_cameras: Query<(&Camera, &GlobalTransform)>,
) {
    send_edit_draw_events(drag.event(), input, commands, q_viewport_nodes, q_cameras);
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
    input: Res<ButtonInput<KeyCode>>,
    commands: Commands,
    q_viewport_nodes: Query<(&ViewportNode, &Interpolation, &PointerLocation)>,
    q_cameras: Query<(&Camera, &GlobalTransform)>,
) {
    send_edit_draw_events(scroll.event(), input, commands, q_viewport_nodes, q_cameras);
}

fn handle_edit_grid_visibility(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    q_viewport_node_hover: Query<(Entity, &Hovered), With<ViewportNode>>,
    q_linked_edit_grids: Query<&LinkedEditGrid>,
) {
    let visibility = if input.any_just_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
        Visibility::Inherited
    } else if input.any_just_released([KeyCode::ShiftLeft, KeyCode::ShiftRight])
        && !input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
    {
        Visibility::Hidden
    } else {
        return;
    };

    for (entity, hovered) in q_viewport_node_hover {
        if hovered.get()
            && let Ok(linked_edit_grid) = q_linked_edit_grids.get(entity)
        {
            commands.entity(linked_edit_grid.0).insert(visibility);
        }
    }
}
