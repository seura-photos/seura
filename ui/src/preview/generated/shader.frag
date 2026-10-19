#version 300 es
// Generated from crates/seura-render/shaders by `npm run shaders`. Don't edit.

precision highp float;
precision highp int;

struct Params {
    float exposure_ev;
    uint output_srgb;
    uint _pad0_;
    uint _pad1_;
};
struct VertexOutput {
    vec4 position;
    vec2 uv;
};
const mat3x3 P3_TO_SRGB = mat3x3(vec3(1.2249401, -0.0420569, -0.0196376), vec3(-0.2249404, 1.0420572, -0.0786361), vec3(0.0, 0.0, 1.0982735));

layout(std140) uniform Params_block_0Fragment { Params _group_0_binding_0_fs; };

uniform highp sampler2D _group_0_binding_1_fs;

smooth in vec2 _vs2fs_location0;
layout(location = 0) out vec4 _fs2p_location0;

vec3 encode_transfer(vec3 linear) {
    vec3 v = abs(linear);
    vec3 low = (v * 12.92);
    vec3 high = ((1.055 * pow(v, vec3(0.41666666))) - vec3(0.055));
    return (sign(linear) * mix(high, low, lessThanEqual(v, vec3(0.0031308))));
}

vec3 exposure(vec3 color_1, float ev) {
    return (color_1 * exp2(ev));
}

void main() {
    VertexOutput in_ = VertexOutput(gl_FragCoord, _vs2fs_location0);
    vec3 color = vec3(0.0);
    vec4 sample_ = textureLod(_group_0_binding_1_fs, vec2(in_.uv), 0.0);
    float _e9 = _group_0_binding_0_fs.exposure_ev;
    vec3 _e10 = exposure(sample_.xyz, _e9);
    color = _e10;
    uint _e14 = _group_0_binding_0_fs.output_srgb;
    if ((_e14 == 1u)) {
        vec3 _e18 = color;
        color = (P3_TO_SRGB * _e18);
    }
    vec3 _e20 = color;
    vec3 _e21 = encode_transfer(_e20);
    _fs2p_location0 = vec4(clamp(_e21, vec3(0.0), vec3(1.0)), 1.0);
    return;
}

