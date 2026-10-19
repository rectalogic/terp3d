use bevy::{
    asset::asset_value,
    camera::{ImageRenderTargetTemplate, RenderTarget, visibility::RenderLayers},
    ecs::template::OptionTemplate,
    post_process::bloom::Bloom,
    prelude::*,
    render::{render_resource::TextureFormat, view::Tonemapping},
    ui::widget::ViewportNode,
};

use crate::camera_controller::{CameraController, CameraControllerPlugin};

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CameraControllerPlugin)
            .add_observer(on_add_drawing_editor);
    }
}

#[derive(Component, Default, Clone)]
#[require(RenderLayers)]
pub struct DrawingEditor {
    pub render_layers: RenderLayers,
}

fn on_add_drawing_editor(
    add: On<Add<DrawingEditor>>,
    mut commands: Commands,
    q_drawing_editors: Query<&DrawingEditor>,
) {
    let image = Image::new_target_texture(0, 0, TextureFormat::Bgra8UnormSrgb, None);
    if let Ok(drawing_editor) = q_drawing_editors.get(add.entity) {
        let render_layers = drawing_editor.render_layers.clone();
        let camera_entity = commands
            .spawn_scene(bsn! {
                Camera3d
                Tonemapping::TonyMcMapface
                Bloom::NATURAL
                Camera { order: -1 }
                Transform::from_xyz(0., 0., 10.0).looking_at(Vec3::ZERO, Vec3::Y)
                render_layers
                RenderTarget::Image(ImageRenderTargetTemplate {
                    handle: asset_value(image),
                    scale_factor: 1.0, //XXX default is 0, panics
                })
            })
            .id();

        commands.entity(add.entity).apply_scene(bsn! {
            ViewportNode { camera: OptionTemplate::Some(camera_entity) }
            CameraController { camera: OptionTemplate::Some(camera_entity) }
        });
    }
}
