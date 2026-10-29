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
const vec3 P3_LUMINANCE = vec3(0.2289746, 0.6917385, 0.0792869);
const float MIDDLE_GRAY = 0.18;
const float PERCEPTUAL = 2.2;

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

vec3 white_balance(vec3 color_2, float temperature, float tint) {
    float t = (temperature / 100.0);
    float n = (tint / 100.0);
    vec3 gains = exp2(vec3((t / 2.0), (-(n) / 2.0), (-(t) / 2.0)));
    return (color_2 * (gains / vec3(dot(P3_LUMINANCE, gains))));
}

vec3 contrast(vec3 color_3, float amount) {
    float curved = 0.0;
    float y = dot(P3_LUMINANCE, color_3);
    if ((y <= 0.0)) {
        return color_3;
    }
    float x = pow(y, 0.45454544);
    if ((x >= 1.0)) {
        return color_3;
    }
    float g = exp2((amount / 100.0));
    if ((x < 0.45865646)) {
        curved = (0.45865646 * pow((x / 0.45865646), g));
    } else {
        curved = (1.0 - ((1.0 - 0.45865646) * pow(((1.0 - x) / (1.0 - 0.45865646)), g)));
    }
    float _e30 = curved;
    return (color_3 * (pow(_e30, PERCEPTUAL) / y));
}

vec3 saturation(vec3 color_4, float amount_1) {
    float y_1 = dot(P3_LUMINANCE, color_4);
    float s = (1.0 + (amount_1 / 100.0));
    return max((vec3(y_1) + ((color_4 - vec3(y_1)) * s)), vec3(0.0));
}

void main() {
    VertexOutput in_ = VertexOutput(gl_FragCoord, _vs2fs_location0);
    vec3 color = vec3(0.0);
    bool local = false;
    vec4 sample_ = textureLod(_group_0_binding_1_fs, vec2(in_.uv), 0.0);
    color = sample_.xyz;
    float _e10 = _group_0_binding_0_fs.exposure_ev;
    if ((_e10 != 0.0)) {
        vec3 _e13 = color;
        float _e16 = _group_0_binding_0_fs.exposure_ev;
        vec3 _e17 = exposure(_e13, _e16);
        color = _e17;
    }
    float _e20 = _group_0_binding_0_fs.temperature;
    if (!((_e20 != 0.0))) {
        float _e28 = _group_0_binding_0_fs.tint;
        local = (_e28 != 0.0);
    } else {
        local = true;
    }
    bool _e32 = local;
    if (_e32) {
        vec3 _e33 = color;
        float _e36 = _group_0_binding_0_fs.temperature;
        float _e39 = _group_0_binding_0_fs.tint;
        vec3 _e40 = white_balance(_e33, _e36, _e39);
        color = _e40;
    }
    float _e43 = _group_0_binding_0_fs.contrast;
    if ((_e43 != 0.0)) {
        vec3 _e46 = color;
        float _e49 = _group_0_binding_0_fs.contrast;
        vec3 _e50 = contrast(_e46, _e49);
        color = _e50;
    }
    float _e53 = _group_0_binding_0_fs.saturation;
    if ((_e53 != 0.0)) {
        vec3 _e56 = color;
        float _e59 = _group_0_binding_0_fs.saturation;
        vec3 _e60 = saturation(_e56, _e59);
        color = _e60;
    }
    uint _e63 = _group_0_binding_0_fs.output_srgb;
    if ((_e63 == 1u)) {
        vec3 _e67 = color;
        color = (P3_TO_SRGB * _e67);
    }
    vec3 _e69 = color;
    vec3 _e70 = encode_transfer(_e69);
    _fs2p_location0 = vec4(clamp(_e70, vec3(0.0), vec3(1.0)), 1.0);
    return;
}

