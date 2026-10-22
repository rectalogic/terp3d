use bevy::{
    asset::asset_value,
    camera::{ImageRenderTargetTemplate, RenderTarget, visibility::RenderLayers},
    ecs::template::OptionTemplate,
    feathers::controls::{ColorInputValue, FeathersColorInput, color_input_self_update},
    prelude::*,
    render::render_resource::TextureFormat,
    ui::widget::ViewportNode,
};

use crate::{
    app::Interpolation,
    camera_controller::{CameraController, CameraControllerPlugin},
};

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CameraControllerPlugin);
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

impl DrawingEditor {
    fn scene(props: DrawingEditorProps) -> impl Scene {
        let image = Image::new_target_texture(0, 0, TextureFormat::Bgra8UnormSrgb, None);

        bsn! {
            Node {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
            }
            Children [
                Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::FlexStart,
                }
                Children [
                    @FeathersColorInput
                    ColorInputValue
                    props.interpolation
                    on(color_input_self_update)
                ]
                --
                Node {
                    flex_grow: 1.0,
                }
                ViewportNode { camera: OptionTemplate::Some(#Camera) }
                CameraController { camera: OptionTemplate::Some(#Camera) }
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
                ]
            ]
        }
    }
}
