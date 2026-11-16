use bevy::{
    camera::Hdr, ecs::template::OptionTemplate, light::Skybox, prelude::*,
    render::view::Tonemapping,
};

#[derive(SceneComponent, Clone, Default)]
pub struct SkyboxCamera;

impl SkyboxCamera {
    fn scene() -> impl Scene {
        bsn! {
            Camera3d
            Hdr
            Tonemapping::TonyMcMapface
            Skybox {
                brightness: 1500.0,
                image: OptionTemplate::Some("ibl/spiaggia_di_mondello_4k-specular.ktx2")
            }
            EnvironmentMapLight {
                // Images created with "ibl/ibl.sh spiaggia_di_mondello_4k.hdr" from
                // https://polyhaven.com/a/spiaggia_di_mondello
                diffuse_map: "ibl/spiaggia_di_mondello_4k-diffuse.ktx2",
                specular_map: "ibl/spiaggia_di_mondello_4k-specular.ktx2",
                intensity: 2500.0,
            }
        }
    }
}
