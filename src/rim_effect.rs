use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
};

pub struct RimEffectPlugin;

impl Plugin for RimEffectPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, RimEffect>,
        >::default());
    }
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct RimEffect {
    #[uniform(100)]
    disabled: u32,
    #[uniform(100)]
    width: f32,
    #[uniform(100)]
    rim_color: LinearRgba,
    #[cfg(feature = "webgl2")]
    #[uniform(100)]
    _webgl2_padding_8b: UVec2,
}

impl RimEffect {
    pub fn new(disabled: bool, width: f32, rim_color: LinearRgba) -> Self {
        Self {
            disabled: disabled.into(),
            width,
            rim_color,
        }
    }

    pub fn disable(&mut self, disabled: bool) {
        self.disabled = disabled.into();
    }
}

impl Default for RimEffect {
    fn default() -> Self {
        Self::new(false, 2.0, LinearRgba::rgb(1.0, 1.0, 0.0))
    }
}

const SHADER_ASSET_PATH: &str = "shaders/rim_effect.wesl";

impl MaterialExtension for RimEffect {
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}
