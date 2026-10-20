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
    color: LinearRgba,
    #[uniform(100)]
    power: f32, // 0.5..10
    #[uniform(100)]
    threshold: f32, // 0..1
    #[cfg(feature = "webgl2")]
    #[uniform(100)]
    _webgl2_padding_4b: u32,
}

impl RimEffect {
    pub fn new(disabled: bool, color: LinearRgba, power: f32, threshold: f32) -> Self {
        Self {
            disabled: disabled.into(),
            color,
            power,
            threshold,
        }
    }

    pub fn disable(&mut self, disabled: bool) {
        self.disabled = disabled.into();
    }
}

impl Default for RimEffect {
    fn default() -> Self {
        Self::new(false, LinearRgba::rgb(1.0, 1.0, 0.0), 3.0, 0.3)
    }
}

const SHADER_ASSET_PATH: &str = "shaders/rim_effect.wesl";

impl MaterialExtension for RimEffect {
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}
