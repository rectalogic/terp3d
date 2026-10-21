use bevy::{
    material::{descriptor::RenderPipelineDescriptor, specialize::SpecializedMeshPipelineError},
    mesh::MeshVertexBufferLayoutRef,
    pbr::{ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline},
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
#[bind_group_data(DisabledRimEffectKey)]
pub struct RimEffect {
    #[uniform(100)]
    color: LinearRgba,
    #[uniform(100)]
    power: f32, // 0.5..10
    #[uniform(100)]
    threshold: f32, // 0..1
    #[cfg(feature = "webgl2")]
    #[uniform(100)]
    _webgl2_padding_8b: UVec2,

    disabled: bool,
}

impl RimEffect {
    pub fn new(disabled: bool, color: LinearRgba, power: f32, threshold: f32) -> Self {
        Self {
            disabled: disabled.into(),
            color,
            power,
            threshold,
            #[cfg(feature = "webgl2")]
            _webgl2_padding_8b: UVec2::default(),
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

    fn specialize(
        _pipeline: &MaterialExtensionPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        key: MaterialExtensionKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if key.bind_group_data.disabled {
            let fragment = descriptor.fragment.as_mut().unwrap();
            fragment.shader_defs.push("DISABLED".into());
        }
        Ok(())
    }
}

#[repr(C)]
#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub struct DisabledRimEffectKey {
    disabled: bool,
}

impl From<&RimEffect> for DisabledRimEffectKey {
    fn from(material: &RimEffect) -> Self {
        Self {
            disabled: material.disabled,
        }
    }
}
