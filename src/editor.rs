use bevy::{
    asset::asset_value,
    camera::{ImageRenderTargetTemplate, RenderTarget, visibility::RenderLayers},
    ecs::template::OptionTemplate,
    prelude::*,
    render::render_resource::TextureFormat,
    ui::widget::ViewportNode,
};

use crate::camera_controller::{CameraController, CameraControllerPlugin};

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CameraControllerPlugin);
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

        bsn! {
            ViewportNode { camera: OptionTemplate::Some(#Camera) }
            CameraController { camera: OptionTemplate::Some(#Camera) }
            RenderingCamera [
                #Camera
                Camera3d
                Camera { order: -1 }
                Transform::from_xyz(0., 0., 10.0).looking_at(Vec3::ZERO, Vec3::Y)
                props.render_layers
                RenderTarget::Image(ImageRenderTargetTemplate {
                    handle: asset_value(image),
                    scale_factor: 1.0, //XXX default is 0, panics
                })
            ]
        }
    }
}
