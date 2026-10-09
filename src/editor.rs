use bevy::{
    asset::{RenderAssetUsages, asset_value},
    camera::{ImageRenderTargetTemplate, RenderTarget, visibility::RenderLayers},
    camera_controller::pan_orbit_camera::{
        DefaultPanOrbitCameraPlugins, controller::component::PanOrbitCamera,
    },
    ecs::template::OptionTemplate,
    prelude::*,
    render::render_resource::{TextureDimension, TextureFormat, TextureUsages},
    ui::widget::ViewportNode,
};

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DefaultPanOrbitCameraPlugins, MeshPickingPlugin));
    }
}

#[derive(SceneComponent, Default, Clone)]
#[scene(DrawingEditorProps)]
pub struct DrawingEditor;

#[derive(Default)]
pub struct DrawingEditorProps {
    layer: usize,
}

#[derive(Component)]
#[relationship(relationship_target = RenderingCamera)]
struct ViewportOfRenderingCamera(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = ViewportOfRenderingCamera, linked_spawn)]
struct RenderingCamera(Entity);

impl DrawingEditor {
    fn scene(props: DrawingEditorProps) -> impl Scene {
        let mut image = Image::new_uninit(
            default(),
            TextureDimension::D2,
            TextureFormat::Bgra8UnormSrgb,
            RenderAssetUsages::all(),
        );
        image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
            | TextureUsages::COPY_DST
            | TextureUsages::RENDER_ATTACHMENT;

        bsn! {
            ViewportNode { camera: OptionTemplate::Some(#Camera) }
            RenderingCamera [
                #Camera
                Camera3d
                Camera { order: -1 }
                PanOrbitCamera
                RenderLayers::layer(props.layer)
                RenderTarget::Image(ImageRenderTargetTemplate {
                    handle: asset_value(image)
                })
            ]
        }
    }
}
