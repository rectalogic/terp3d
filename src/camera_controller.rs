use bevy::{
    camera::visibility::RenderLayers,
    camera_controller::pan_orbit_camera::{
        DefaultPanOrbitCameraPlugins, controller::component::PanOrbitCamera,
    },
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    input::mouse::MouseScrollPixelsPerLine,
    picking::PickingSystems,
    prelude::*,
};

pub struct CameraControllerPlugin;

impl Plugin for CameraControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DefaultPanOrbitCameraPlugins, MeshPickingPlugin))
            .add_systems(Startup, setup_axes)
            .add_systems(Update, configure_pan_orbit_cameras)
            .add_systems(
                PreUpdate,
                end_viewport_zoom
                    .after(PickingSystems::Last)
                    .before(PanOrbitCamera::update_camera_positions),
            );
    }
}

#[derive(Component, Clone, Default, Debug, FromTemplate)]
#[component(on_add)]
pub struct CameraController {
    #[template(built_in)]
    pub camera: Option<Entity>,
}

impl CameraController {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        world
            .commands()
            .entity(context.entity)
            .observe(on_viewport_pointer_press)
            .observe(on_viewport_pointer_drag)
            .observe(on_viewport_pointer_release)
            .observe(on_viewport_pointer_scroll);
    }
}

fn configure_pan_orbit_cameras(
    mut commands: Commands,
    camera_controllers: Query<&CameraController>,
    cameras: Query<&Transform, (With<Camera>, Without<PanOrbitCamera>)>,
) {
    for camera_controller in camera_controllers {
        if let Some(camera_entity) = camera_controller.camera
            && let Ok(camera_transform) = cameras.get(camera_entity)
        {
            commands.entity(camera_entity).insert(PanOrbitCamera {
                last_anchor_depth: -camera_transform.translation.length() as f64,
                ..default()
            });
        }
    }
}

fn on_viewport_pointer_press(
    press: On<PointerPress>,
    mut commands: Commands,
    camera_controllers: Query<&CameraController>,
    mut pan_orbit_cameras: Query<&mut PanOrbitCamera>,
    layers: Query<&RenderLayers>,
    gizmo_axes: Single<Entity, With<GizmoAxes>>,
) {
    if let Ok(camera_controller) = camera_controllers.get(press.entity)
        && let Some(camera) = camera_controller.camera
        && let Ok(mut pan_orbit_camera) = pan_orbit_cameras.get_mut(camera)
        && !pan_orbit_camera.is_actively_controlled()
    {
        if let Ok(layer) = layers.get(camera) {
            commands.entity(*gizmo_axes).insert(layer.clone());
        } else {
            commands.entity(*gizmo_axes).remove::<RenderLayers>();
        }
        pan_orbit_camera.start_orbit(None);
    };
}

fn on_viewport_pointer_drag(
    drag: On<PointerDrag>,
    camera_controllers: Query<&CameraController>,
    mut pan_orbit_cameras: Query<&mut PanOrbitCamera>,
) {
    if let Ok(camera_controller) = camera_controllers.get(drag.entity)
        && let Some(camera) = camera_controller.camera
        && let Ok(mut pan_orbit_camera) = pan_orbit_cameras.get_mut(camera)
    {
        pan_orbit_camera.send_screenspace_input(drag.delta);
    };
}

fn on_viewport_pointer_release(
    release: On<PointerRelease>,
    mut commands: Commands,
    camera_controllers: Query<&CameraController>,
    mut pan_orbit_cameras: Query<&mut PanOrbitCamera>,
    gizmo_axes: Single<Entity, With<GizmoAxes>>,
) {
    if let Ok(camera_controller) = camera_controllers.get(release.entity)
        && let Some(camera) = camera_controller.camera
        && let Ok(mut pan_orbit_camera) = pan_orbit_cameras.get_mut(camera)
    {
        commands.entity(*gizmo_axes).insert(RenderLayers::none());
        pan_orbit_camera.end_move();
    };
}

fn on_viewport_pointer_scroll(
    scroll: On<PointerScroll>,
    mut commands: Commands,
    camera_controllers: Query<&CameraController>,
    mut pan_orbit_cameras: Query<&mut PanOrbitCamera>,
    pixels_per_line: Res<MouseScrollPixelsPerLine>,
    layers: Query<&RenderLayers>,
    gizmo_axes: Single<Entity, With<GizmoAxes>>,
) {
    if let Ok(camera_controller) = camera_controllers.get(scroll.entity)
        && let Some(camera) = camera_controller.camera
        && let Ok(mut pan_orbit_camera) = pan_orbit_cameras.get_mut(camera)
    {
        if let Ok(layer) = layers.get(camera) {
            commands.entity(*gizmo_axes).insert(layer.clone());
        }
        let scroll = scroll.to_pixels(&pixels_per_line);
        pan_orbit_camera.start_zoom(None);
        pan_orbit_camera.send_zoom_input(scroll.y);
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
            commands.entity(*gizmo_axes).insert(RenderLayers::none());
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
        RenderLayers::none()
    });
}
