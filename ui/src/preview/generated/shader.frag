#version 300 es
// Generated from crates/seura-render/shaders by `npm run shaders`. Don't edit.

precision highp float;
precision highp int;

struct Params {
    uint output_srgb;
    uint rotate;
    uint flip_h;
    uint flip_v;
    float straighten;
    float crop_x;
    float crop_y;
    float crop_width;
    float crop_height;
    float exposure_ev;
    float temperature;
    float tint;
    float highlights;
    float shadows;
    float contrast;
    float saturation;
    float black_and_white;
    float sepia;
    float vignette_amount;
    float vignette_size;
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
    color = sample_.xyz;
    float _e10 = _group_0_binding_0_fs.exposure_ev;
    if ((_e10 != 0.0)) {
        vec3 _e13 = color;
        float _e16 = _group_0_binding_0_fs.exposure_ev;
        vec3 _e17 = exposure(_e13, _e16);
        color = _e17;
    }
    uint _e20 = _group_0_binding_0_fs.output_srgb;
    if ((_e20 == 1u)) {
        vec3 _e24 = color;
        color = (P3_TO_SRGB * _e24);
    }
    vec3 _e26 = color;
    vec3 _e27 = encode_transfer(_e26);
    _fs2p_location0 = vec4(clamp(_e27, vec3(0.0), vec3(1.0)), 1.0);
    return;
}

