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
    viewports: Query<&ViewportNode>,
    mut cameras: Query<&mut PanOrbitCamera>,
) {
    if let Ok(viewport) = viewports.get(press.entity)
        && let Some(camera) = viewport.camera
        && let Ok(mut controller) = cameras.get_mut(camera)
        && !controller.is_actively_controlled()
    {
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
    viewports: Query<&ViewportNode>,
    mut cameras: Query<&mut PanOrbitCamera>,
) {
    if let Ok(viewport) = viewports.get(drag.entity)
        && let Some(camera) = viewport.camera
        && let Ok(mut controller) = cameras.get_mut(camera)
    {
        controller.end_move();
    };
}

fn on_viewport_pointer_scroll(
    scroll: On<PointerScroll>,
    viewports: Query<&ViewportNode>,
    mut cameras: Query<&mut PanOrbitCamera>,
    pixels_per_line: Res<MouseScrollPixelsPerLine>,
) {
    if let Ok(viewport) = viewports.get(scroll.entity)
        && let Some(camera) = viewport.camera
        && let Ok(mut controller) = cameras.get_mut(camera)
    {
        let scroll = scroll.to_pixels(&pixels_per_line);
        controller.start_zoom(None);
        controller.send_zoom_input(scroll.y);
    };
}

fn end_viewport_zoom(mut cameras: Query<&mut PanOrbitCamera>) {
    for mut controller in &mut cameras {
        let zoom_settled = controller.current_motion.is_zooming_only()
            && controller
                .current_motion
                .inputs()
                .map(|inputs| inputs.zoom_velocity_abs(controller.smoothing.zoom.mul_f32(2.0)))
                .unwrap_or(0.0)
                <= 0.0;
        if zoom_settled {
            controller.end_move();
        }
    }
}
