// https://ameye.dev/notes/rendering-outlines/
// https://github.com/vadimb892/bevy_rim_effect/blob/master/src/rim_effect.wgsl
// https://github.com/SametKaras/Unity-Rim-Light-Shader/blob/master/Assets/RimLightShader.shader

import bevy_pbr::render::{
    forward_io::{FragmentOutput, VertexOutput},
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{alpha_discard, apply_pbr_lighting, main_pass_post_lighting_processing},
    pbr_types::PbrInput
};

struct RimEffectMaterial {
    color: vec4<f32>,
    power: f32,
    threshold: f32,
    // Web examples WebGL2 support: structs must be 16 byte aligned.
    @if(SIXTEEN_BYTE_ALIGNMENT)
    _webgl2_padding_8b: vec2<u32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(100)
var<uniform> rim_effect_material: RimEffectMaterial;

fn rim_effect(pbr_input: PbrInput) -> vec4<f32> {
    let ndv = dot(normalize(pbr_input.N), normalize(pbr_input.V));
    let rim = 1.0 - ndv;
    let rimT = smoothstep(rim_effect_material.threshold, 1.0, rim);
    let rimFinal = pow(rimT, rim_effect_material.power);
    return vec4<f32>(rim_effect_material.color.rgb * rimFinal, 1.0);
}

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    pbr_input.material.base_color = alpha_discard(
            pbr_input.material.flags,
            pbr_input.material.alpha_cutoff,
            pbr_input.material.base_color,
        );

    var out: FragmentOutput;

    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);

    @if(!DISABLED)
    out.color += rim_effect(pbr_input);

    return out;
}
