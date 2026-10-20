// https://ameye.dev/notes/rendering-outlines/
// https://github.com/vadimb892/bevy_rim_effect/blob/master/src/rim_effect.wgsl

import bevy_pbr::render::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{alpha_discard, apply_pbr_lighting, main_pass_post_lighting_processing},
    pbr_types::PbrInput,
};

struct RimEffectMaterial {
    disabled: u32,
    width: f32,
    rim_color: vec4<f32>,
    // Web examples WebGL2 support: structs must be 16 byte aligned.
    @if(SIXTEEN_BYTE_ALIGNMENT)
    _webgl2_padding_8b: vec2<u32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(100)
var<uniform> rim_effect_material: RimEffectMaterial;

fn rim_effect(pbr_input: PbrInput, width: f32) -> f32 {
    var N : vec3f = pbr_input.N;
    let V : vec3f = pbr_input.V;
    var fresnel =  1.0 - max(dot(N, V), 0.0001);
    fresnel = 2.0 * pow(saturate(fresnel), width);
    return fresnel;
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    pbr_input.material.base_color = alpha_discard(
        pbr_input.material.flags,
        pbr_input.material.alpha_cutoff,
        pbr_input.material.base_color
    );

    var out: FragmentOutput;

    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);

    if rim_effect_material.disabled == 0 {
        out.color = mix(out.color, rim_effect_material.rim_color, rim_effect(pbr_input, rim_effect_material.width));
    }

    return out;
}
