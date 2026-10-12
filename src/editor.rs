use bevy::{
    asset::asset_value,
    camera::{ImageRenderTargetTemplate, RenderTarget, visibility::RenderLayers},
    camera_controller::pan_orbit_camera::{
        DefaultPanOrbitCameraPlugins, controller::component::PanOrbitCamera,
    },
    ecs::template::OptionTemplate,
    input::mouse::MouseScrollPixelsPerLine,
    picking::PickingSystems,
    prelude::*,
    render::render_resource::TextureFormat,
    ui::widget::ViewportNode,
};

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DefaultPanOrbitCameraPlugins, MeshPickingPlugin))
            .add_systems(Startup, setup_axes)
            .add_systems(
                PreUpdate,
                end_viewport_zoom
                    .after(PickingSystems::Last)
                    .before(PanOrbitCamera::update_camera_positions),
            );
    }
}

#[derive(SceneComponent, Default, Clone)]
#[scene(DrawingEditorProps)]
pub struct DrawingEditor;

#[derive(Default)]
pub struct DrawingEditorProps {
    pub render_layers: RenderLayers,
}

#[derive(Component)]
#[relationship(relationship_target = RenderingCamera)]
struct ViewportOfRenderingCamera(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = ViewportOfRenderingCamera, linked_spawn)]
struct RenderingCamera(Entity);

impl DrawingEditor {
    fn scene(props: DrawingEditorProps) -> impl Scene {
        let image = Image::new_target_texture(0, 0, TextureFormat::Bgra8UnormSrgb, None);
        let cam_trans = Transform::from_xyz(0., 0., 10.0).looking_at(Vec3::ZERO, Vec3::Y);

        bsn! {
            ViewportNode { camera: OptionTemplate::Some(#Camera) }
            on(on_viewport_pointer_press)
            on(on_viewport_pointer_drag)
            on(on_viewport_pointer_release)
            on(on_viewport_pointer_scroll)
            RenderingCamera [
                #Camera
                Camera3d
                Camera { order: -1 }
                cam_trans
                PanOrbitCamera { last_anchor_depth: {-cam_trans.translation.length() as f64} }
                props.render_layers
                RenderTarget::Image(ImageRenderTargetTemplate {
                    handle: asset_value(image),
                    scale_factor: 1.0, //XXX default is 0, panics
                })
            ]
        }
    }
}

fn on_viewport_pointer_press(
    press: On<PointerPress>,
    mut commands: Commands,
    viewports: Query<&ViewportNode>,
    mut cameras: Query<&mut PanOrbitCamera>,
    layers: Query<&RenderLayers>,
    gizmo_axes: Single<Entity, With<GizmoAxes>>,
) {
    if let Ok(viewport) = viewports.get(press.entity)
        && let Some(camera) = viewport.camera
        && let Ok(mut controller) = cameras.get_mut(camera)
        && !controller.is_actively_controlled()
    {
        if let Ok(layer) = layers.get(camera) {
            commands.entity(*gizmo_axes).insert(layer.clone());
        }
        controller.start_orbit(None);
    };
}

fn on_viewport_pointer_drag(
    drag: On<PointerDrag>,
    viewports: Query<&ViewportNode>,
    mut cameras: Query<&mut PanOrbitCamera>,
) {
    if let Ok(viewport) = viewports.get(drag.entity)
        && let Some(camera) = viewport.camera
        && let Ok(mut controller) = cameras.get_mut(camera)
    {
        controller.send_screenspace_input(drag.delta);
    };
}

fn on_viewport_pointer_release(
    drag: On<PointerRelease>,
    mut commands: Commands,
    viewports: Query<&ViewportNode>,
    mut cameras: Query<&mut PanOrbitCamera>,
    gizmo_axes: Single<Entity, With<GizmoAxes>>,
) {
    if let Ok(viewport) = viewports.get(drag.entity)
        && let Some(camera) = viewport.camera
        && let Ok(mut controller) = cameras.get_mut(camera)
    {
        commands.entity(*gizmo_axes).remove::<RenderLayers>();
        controller.end_move();
    };
}

fn on_viewport_pointer_scroll(
    scroll: On<PointerScroll>,
    mut commands: Commands,
    viewports: Query<&ViewportNode>,
    mut cameras: Query<&mut PanOrbitCamera>,
    pixels_per_line: Res<MouseScrollPixelsPerLine>,
    layers: Query<&RenderLayers>,
    gizmo_axes: Single<Entity, With<GizmoAxes>>,
) {
    if let Ok(viewport) = viewports.get(scroll.entity)
        && let Some(camera) = viewport.camera
        && let Ok(mut controller) = cameras.get_mut(camera)
    {
        if let Ok(layer) = layers.get(camera) {
            commands.entity(*gizmo_axes).insert(layer.clone());
        }
        let scroll = scroll.to_pixels(&pixels_per_line);
        controller.start_zoom(None);
        controller.send_zoom_input(scroll.y);
    };
}

fn end_viewport_zoom(
    mut commands: Commands,
    mut cameras: Query<&mut PanOrbitCamera>,
    gizmo_axes: Single<Entity, With<GizmoAxes>>,
) {
    for mut controller in &mut cameras {
        let zoom_settled = controller.current_motion.is_zooming_only()
            && controller
                .current_motion
                .inputs()
                .map(|inputs| inputs.zoom_velocity_abs(controller.smoothing.zoom.mul_f32(2.0)))
                .unwrap_or(0.0)
                <= 0.0;
        if zoom_settled {
            commands.entity(*gizmo_axes).remove::<RenderLayers>();
            controller.end_move();
        }
    }
}

#[derive(Component, Default, Clone)]
struct GizmoAxes;

fn setup_axes(mut commands: Commands, mut gizmo_assets: ResMut<Assets<GizmoAsset>>) {
    let mut gizmo_asset = GizmoAsset::default();
    let cells = UVec2::splat(15);
    let spacing = Vec2::splat(0.3);
    let alpha = 0.3;

    gizmo_asset
        .grid(
            Isometry3d::IDENTITY,
            cells,
            spacing,
            Srgba::new(1.0, 0.0, 0.0, alpha),
        )
        .outer_edges();

    let xz_iso = Isometry3d::new(
        Vec3::ZERO,
        Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
    );
    gizmo_asset
        .grid(xz_iso, cells, spacing, Srgba::new(0.0, 1.0, 0.0, alpha))
        .outer_edges();

    let yz_iso = Isometry3d::new(
        Vec3::ZERO,
        Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
    );
    gizmo_asset
        .grid(yz_iso, cells, spacing, Srgba::new(0.0, 0.0, 1.0, alpha))
        .outer_edges();

    let gizmo_handle = gizmo_assets.add(gizmo_asset);
    commands.spawn_scene(bsn! {
        GizmoAxes
        Gizmo {
            handle: gizmo_handle,
            line_config: GizmoLineConfig { width: 3.0, perspective: true }
        }
    });
}
