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
    uint draw_pass;
    float tile_x;
    float tile_y;
    float tile_width;
    float tile_height;
};
struct VertexOutput {
    vec4 position;
    vec2 uv;
};
const mat3x3 P3_TO_SRGB = mat3x3(vec3(1.2249401, -0.0420569, -0.0196376), vec3(-0.2249404, 1.0420572, -0.0786361), vec3(0.0, 0.0, 1.0982735));
const vec3 P3_LUMINANCE = vec3(0.2289746, 0.6917385, 0.0792869);
const float MIDDLE_GRAY = 0.18;
const float PERCEPTUAL = 2.2;
const float HIGHLIGHTS_SHADOWS_STOPS = 2.0;
const float GUIDE_FLOOR = 0.00024414063;
const float GUIDE_RADIUS = 0.02;
const float GUIDE_EPSILON = 0.1;

uniform highp sampler2D _group_0_binding_5_fs;

layout(std140) uniform Params_block_0Fragment { Params _group_0_binding_0_fs; };

uniform highp sampler2D _group_0_binding_1_fs;

uniform highp sampler2D _group_0_binding_3_fs;

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

float tone_position(float base_1) {
    return pow(clamp(exp2(base_1), 0.0, 1.0), 0.45454544);
}

float highlights_gain(float base_2, float amount_1) {
    float _e2 = tone_position(base_2);
    return (((((amount_1 / 100.0) * HIGHLIGHTS_SHADOWS_STOPS) * _e2) * _e2) * _e2);
}

float shadows_gain(float base_3, float amount_2) {
    float _e2 = tone_position(base_3);
    float x_1 = (1.0 - _e2);
    return (((((amount_2 / 100.0) * HIGHLIGHTS_SHADOWS_STOPS) * x_1) * x_1) * x_1);
}

vec3 saturation(vec3 color_4, float amount_3) {
    float y_1 = dot(P3_LUMINANCE, color_4);
    float s = (1.0 + (amount_3 / 100.0));
    return (vec3(y_1) + ((color_4 - vec3(y_1)) * s));
}

float log_luminance(vec3 color_5) {
    return log2(max(dot(P3_LUMINANCE, color_5), GUIDE_FLOOR));
}

vec2 guide_box(ivec2 at, ivec2 direction) {
    vec2 sum = vec2(0.0);
    int i = 0;
    ivec2 size = ivec2(uvec2(textureSize(_group_0_binding_5_fs, 0).xy));
    int r = max(1, int(roundEven((float(max(size.x, size.y)) * GUIDE_RADIUS))));
    i = -(r);
    bool loop_init = true;
    while(true) {
        if (!loop_init) {
            int _e38 = i;
            i = (_e38 + 1);
        }
        loop_init = false;
        int _e20 = i;
        if ((_e20 <= r)) {
        } else {
            break;
        }
        {
            int _e22 = i;
            ivec2 neighbour = min(max((at + (direction * _e22)), ivec2(0)), (size - ivec2(1)));
            vec2 _e31 = sum;
            vec4 _e34 = texelFetch(_group_0_binding_5_fs, neighbour, 0);
            sum = (_e31 + _e34.xy);
        }
    }
    vec2 _e40 = sum;
    return (_e40 / vec2(float(((2 * r) + 1))));
}

vec4 guide_pass(vec4 position) {
    vec3 color_6 = vec3(0.0);
    bool local_2 = false;
    ivec2 at_1 = ivec2(position.xy);
    uint _e5 = _group_0_binding_0_fs.draw_pass;
    switch(_e5) {
        case 1u: {
            vec4 _e8 = texelFetch(_group_0_binding_5_fs, at_1, 0);
            color_6 = _e8.xyz;
            float _e13 = _group_0_binding_0_fs.temperature;
            if (!((_e13 != 0.0))) {
                float _e21 = _group_0_binding_0_fs.tint;
                local_2 = (_e21 != 0.0);
            } else {
                local_2 = true;
            }
            bool _e25 = local_2;
            if (_e25) {
                vec3 _e26 = color_6;
                float _e29 = _group_0_binding_0_fs.temperature;
                float _e32 = _group_0_binding_0_fs.tint;
                vec3 _e33 = white_balance(_e26, _e29, _e32);
                color_6 = _e33;
            }
            vec3 _e34 = color_6;
            float _e35 = log_luminance(_e34);
            return vec4(_e35, (_e35 * _e35), 0.0, 1.0);
        }
        case 2u: {
            vec2 _e43 = guide_box(at_1, ivec2(1, 0));
            return vec4(_e43, 0.0, 1.0);
        }
        case 3u: {
            vec2 _e50 = guide_box(at_1, ivec2(0, 1));
            float variance = max((_e50.y - (_e50.x * _e50.x)), 0.0);
            float a = (variance / (variance + GUIDE_EPSILON));
            return vec4(a, (_e50.x - (a * _e50.x)), 0.0, 1.0);
        }
        default: {
            vec2 _e71 = guide_box(at_1, ivec2(0, 1));
            return vec4(_e71, 0.0, 1.0);
        }
    }
}

void main() {
    VertexOutput in_ = VertexOutput(gl_FragCoord, _vs2fs_location0);
    vec3 color = vec3(0.0);
    bool local = false;
    bool local_1 = false;
    float base = 0.0;
    uint _e3 = _group_0_binding_0_fs.draw_pass;
    if ((_e3 != 0u)) {
        vec4 _e7 = guide_pass(in_.position);
        _fs2p_location0 = _e7;
        return;
    }
    vec4 sample_ = textureLod(_group_0_binding_1_fs, vec2(in_.uv), 0.0);
    color = sample_.xyz;
    float _e17 = _group_0_binding_0_fs.exposure_ev;
    if ((_e17 != 0.0)) {
        vec3 _e20 = color;
        float _e23 = _group_0_binding_0_fs.exposure_ev;
        vec3 _e24 = exposure(_e20, _e23);
        color = _e24;
    }
    float _e27 = _group_0_binding_0_fs.temperature;
    if (!((_e27 != 0.0))) {
        float _e35 = _group_0_binding_0_fs.tint;
        local = (_e35 != 0.0);
    } else {
        local = true;
    }
    bool _e39 = local;
    if (_e39) {
        vec3 _e40 = color;
        float _e43 = _group_0_binding_0_fs.temperature;
        float _e46 = _group_0_binding_0_fs.tint;
        vec3 _e47 = white_balance(_e40, _e43, _e46);
        color = _e47;
    }
    float _e50 = _group_0_binding_0_fs.highlights;
    if (!((_e50 != 0.0))) {
        float _e58 = _group_0_binding_0_fs.shadows;
        local_1 = (_e58 != 0.0);
    } else {
        local_1 = true;
    }
    bool _e62 = local_1;
    if (_e62) {
        float _e65 = _group_0_binding_0_fs.tile_x;
        float _e68 = _group_0_binding_0_fs.tile_y;
        float _e73 = _group_0_binding_0_fs.tile_width;
        float _e76 = _group_0_binding_0_fs.tile_height;
        vec2 at_2 = (vec2(_e65, _e68) + (in_.uv * vec2(_e73, _e76)));
        vec4 _e83 = textureLod(_group_0_binding_3_fs, vec2(at_2), 0.0);
        vec2 ab = _e83.xy;
        vec3 _e85 = color;
        float _e88 = _group_0_binding_0_fs.exposure_ev;
        float _e92 = log_luminance((_e85 * exp2(-(_e88))));
        float _e99 = _group_0_binding_0_fs.exposure_ev;
        base = (((ab.x * _e92) + ab.y) + _e99);
        float _e104 = _group_0_binding_0_fs.highlights;
        if ((_e104 != 0.0)) {
            float _e107 = base;
            float _e110 = _group_0_binding_0_fs.highlights;
            float _e111 = highlights_gain(_e107, _e110);
            vec3 _e112 = color;
            color = (_e112 * exp2(_e111));
            float _e115 = base;
            base = (_e115 + _e111);
        }
        float _e119 = _group_0_binding_0_fs.shadows;
        if ((_e119 != 0.0)) {
            vec3 _e122 = color;
            float _e123 = base;
            float _e126 = _group_0_binding_0_fs.shadows;
            float _e127 = shadows_gain(_e123, _e126);
            color = (_e122 * exp2(_e127));
        }
    }
    float _e132 = _group_0_binding_0_fs.contrast;
    if ((_e132 != 0.0)) {
        vec3 _e135 = color;
        float _e138 = _group_0_binding_0_fs.contrast;
        vec3 _e139 = contrast(_e135, _e138);
        color = _e139;
    }
    float _e142 = _group_0_binding_0_fs.saturation;
    if ((_e142 != 0.0)) {
        vec3 _e145 = color;
        float _e148 = _group_0_binding_0_fs.saturation;
        vec3 _e149 = saturation(_e145, _e148);
        color = _e149;
    }
    uint _e152 = _group_0_binding_0_fs.output_srgb;
    if ((_e152 == 1u)) {
        vec3 _e156 = color;
        color = (P3_TO_SRGB * _e156);
    }
    vec3 _e158 = color;
    vec3 _e159 = encode_transfer(_e158);
    _fs2p_location0 = vec4(clamp(_e159, vec3(0.0), vec3(1.0)), 1.0);
    return;
}

