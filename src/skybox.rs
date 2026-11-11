use bevy::{camera::Hdr, ecs::template::OptionTemplate, light::Skybox, prelude::*};

#[derive(SceneComponent, Clone, Default)]
pub struct SkyboxCamera;

impl SkyboxCamera {
    fn scene() -> impl Scene {
        bsn! {
            Camera3d
            Hdr
            Skybox {
                brightness: 5000.0,
                image: OptionTemplate::Some("ibl/field-specular.ktx2")
            }
            EnvironmentMapLight {
                // Images created with "ibl/ibl.sh field.hdr" from
                // https://github.com/KhronosGroup/glTF-Sample-Environments/blob/main/field.hdr
                diffuse_map: "ibl/field-diffuse.ktx2",
                specular_map: "ibl/field-specular.ktx2",
                intensity: 2500.0,
            }
        }
    }
}
