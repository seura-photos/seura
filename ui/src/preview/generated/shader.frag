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
    float full_width;
    float full_height;
    float view_x;
    float view_y;
    float view_width;
    float view_height;
    float crop_left;
    float crop_top;
    float crop_pixels_width;
    float crop_pixels_height;
    uint red_eye_count;
    vec4 red_eye[64];
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
const vec3 SEPIA_TONE = vec3(1.2479966, 0.9684454, 0.5591025);
const float GUIDE_FLOOR = 0.00024414063;
const float GUIDE_RADIUS = 0.02;
const float GUIDE_EPSILON = 0.1;

layout(std140) uniform Params_block_0Fragment { Params _group_0_binding_0_fs; };

uniform highp sampler2D _group_0_binding_5_fs;

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

vec3 red_eye(vec3 color_1, vec2 position) {
    float weight = 0.0;
    uint i = 0u;
    bool local_2 = false;
    bool local_3 = false;
    float _e4 = _group_0_binding_0_fs.full_width;
    float _e7 = _group_0_binding_0_fs.full_height;
    vec2 full = vec2(_e4, _e7);
    float shorter = min(full.x, full.y);
    bool loop_init = true;
    while(true) {
        if (!loop_init) {
            uint _e45 = i;
            i = (_e45 + 1u);
        }
        loop_init = false;
        uint _e16 = i;
        uint _e19 = _group_0_binding_0_fs.red_eye_count;
        if ((_e16 < min(_e19, 64u))) {
        } else {
            break;
        }
        {
            uint _e25 = i;
            vec4 region = _group_0_binding_0_fs.red_eye[_e25];
            if ((region.z <= 0.0)) {
                continue;
            }
            float d = (distance(position, (region.xy * full)) / (region.z * shorter));
            float _e37 = weight;
            weight = max(_e37, (1.0 - smoothstep(0.8, 1.0, d)));
        }
    }
    float m_1 = max(color_1.y, color_1.z);
    float _e50 = weight;
    if (!((_e50 == 0.0))) {
        local_2 = (color_1.x <= 0.0);
    } else {
        local_2 = true;
    }
    bool _e60 = local_2;
    if (!(_e60)) {
        local_3 = (color_1.x <= m_1);
    } else {
        local_3 = true;
    }
    bool _e67 = local_3;
    if (_e67) {
        return color_1;
    }
    float k = smoothstep(0.7, 0.9, ((color_1.x - m_1) / color_1.x));
    float _e78 = weight;
    return vec3((color_1.x + (((m_1 - color_1.x) * _e78) * k)), color_1.y, color_1.z);
}

vec3 exposure(vec3 color_2, float ev) {
    return (color_2 * exp2(ev));
}

vec3 white_balance(vec3 color_3, float temperature, float tint) {
    float t_1 = (temperature / 100.0);
    float n_1 = (tint / 100.0);
    vec3 gains = exp2(vec3((t_1 / 2.0), (-(n_1) / 2.0), (-(t_1) / 2.0)));
    return (color_3 * (gains / vec3(dot(P3_LUMINANCE, gains))));
}

vec3 contrast(vec3 color_4, float amount) {
    float curved = 0.0;
    float y = dot(P3_LUMINANCE, color_4);
    if ((y <= 0.0)) {
        return color_4;
    }
    float x = pow(y, 0.45454544);
    if ((x >= 1.0)) {
        return color_4;
    }
    float g = exp2((amount / 100.0));
    if ((x < 0.45865646)) {
        curved = (0.45865646 * pow((x / 0.45865646), g));
    } else {
        curved = (1.0 - ((1.0 - 0.45865646) * pow(((1.0 - x) / (1.0 - 0.45865646)), g)));
    }
    float _e30 = curved;
    return (color_4 * (pow(_e30, PERCEPTUAL) / y));
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

vec3 saturation(vec3 color_5, float amount_3) {
    float y_1 = dot(P3_LUMINANCE, color_5);
    float s = (1.0 + (amount_3 / 100.0));
    return (vec3(y_1) + ((color_5 - vec3(y_1)) * s));
}

vec3 black_and_white(vec3 color_6, float strength) {
    float y_2 = dot(P3_LUMINANCE, color_6);
    return (color_6 + ((vec3(y_2) - color_6) * (strength / 100.0)));
}

vec3 sepia(vec3 color_7, float strength_1) {
    float y_3 = dot(P3_LUMINANCE, color_7);
    return (color_7 + (((y_3 * SEPIA_TONE) - color_7) * (strength_1 / 100.0)));
}

vec3 vignette(vec3 color_8, float amount_4, float size, vec2 at) {
    float r = length(((at * 2.0) - vec2(1.0)));
    float start = (1.0 - (size / 100.0));
    float t_2 = clamp(((r - start) / (1.4142135 - start)), 0.0, 1.0);
    float m_2 = ((((amount_4 / 100.0) * t_2) * t_2) * (3.0 - (2.0 * t_2)));
    if ((m_2 < 0.0)) {
        return (color_8 * (1.0 + m_2));
    }
    return (color_8 + ((vec3(1.0) - color_8) * m_2));
}

float log_luminance(vec3 color_9) {
    return log2(max(dot(P3_LUMINANCE, color_9), GUIDE_FLOOR));
}

vec2 guide_box(ivec2 at_1, ivec2 direction) {
    vec2 sum = vec2(0.0);
    int i_1 = 0;
    ivec2 size_1 = ivec2(uvec2(textureSize(_group_0_binding_5_fs, 0).xy));
    int r_1 = max(1, int(roundEven((float(max(size_1.x, size_1.y)) * GUIDE_RADIUS))));
    i_1 = -(r_1);
    bool loop_init_1 = true;
    while(true) {
        if (!loop_init_1) {
            int _e38 = i_1;
            i_1 = (_e38 + 1);
        }
        loop_init_1 = false;
        int _e20 = i_1;
        if ((_e20 <= r_1)) {
        } else {
            break;
        }
        {
            int _e22 = i_1;
            ivec2 neighbour = min(max((at_1 + (direction * _e22)), ivec2(0)), (size_1 - ivec2(1)));
            vec2 _e31 = sum;
            vec4 _e34 = texelFetch(_group_0_binding_5_fs, neighbour, 0);
            sum = (_e31 + _e34.xy);
        }
    }
    vec2 _e40 = sum;
    return (_e40 / vec2(float(((2 * r_1) + 1))));
}

vec4 guide_pass(vec4 position_1) {
    vec3 color_10 = vec3(0.0);
    bool local_4 = false;
    ivec2 at_2 = ivec2(position_1.xy);
    uint _e5 = _group_0_binding_0_fs.draw_pass;
    switch(_e5) {
        case 1u: {
            vec4 _e8 = texelFetch(_group_0_binding_5_fs, at_2, 0);
            color_10 = _e8.xyz;
            float _e13 = _group_0_binding_0_fs.temperature;
            if (!((_e13 != 0.0))) {
                float _e21 = _group_0_binding_0_fs.tint;
                local_4 = (_e21 != 0.0);
            } else {
                local_4 = true;
            }
            bool _e25 = local_4;
            if (_e25) {
                vec3 _e26 = color_10;
                float _e29 = _group_0_binding_0_fs.temperature;
                float _e32 = _group_0_binding_0_fs.tint;
                vec3 _e33 = white_balance(_e26, _e29, _e32);
                color_10 = _e33;
            }
            vec3 _e34 = color_10;
            float _e35 = log_luminance(_e34);
            return vec4(_e35, (_e35 * _e35), 0.0, 1.0);
        }
        case 2u: {
            vec2 _e43 = guide_box(at_2, ivec2(1, 0));
            return vec4(_e43, 0.0, 1.0);
        }
        case 3u: {
            vec2 _e50 = guide_box(at_2, ivec2(0, 1));
            float variance = max((_e50.y - (_e50.x * _e50.x)), 0.0);
            float a = (variance / (variance + GUIDE_EPSILON));
            return vec4(a, (_e50.x - (a * _e50.x)), 0.0, 1.0);
        }
        default: {
            vec2 _e71 = guide_box(at_2, ivec2(0, 1));
            return vec4(_e71, 0.0, 1.0);
        }
    }
}

vec2 result_position(vec2 uv) {
    float _e3 = _group_0_binding_0_fs.crop_pixels_width;
    float _e6 = _group_0_binding_0_fs.crop_pixels_height;
    vec2 crop = vec2(_e3, _e6);
    float _e10 = _group_0_binding_0_fs.view_x;
    float _e13 = _group_0_binding_0_fs.view_y;
    vec2 view = roundEven((vec2(_e10, _e13) * crop));
    float _e19 = _group_0_binding_0_fs.view_width;
    float _e22 = _group_0_binding_0_fs.view_height;
    vec2 view_size = roundEven((vec2(_e19, _e22) * crop));
    return (view + (uv * view_size));
}

vec2 upright_position(vec2 result) {
    vec2 frame = vec2(0.0);
    bool local_5 = false;
    vec2 q = vec2(0.0);
    float _e3 = _group_0_binding_0_fs.full_width;
    float _e6 = _group_0_binding_0_fs.full_height;
    vec2 full_1 = vec2(_e3, _e6);
    frame = full_1;
    uint _e11 = _group_0_binding_0_fs.rotate;
    if (!((_e11 == 90u))) {
        uint _e19 = _group_0_binding_0_fs.rotate;
        local_5 = (_e19 == 270u);
    } else {
        local_5 = true;
    }
    bool _e23 = local_5;
    if (_e23) {
        frame = full_1.yx;
    }
    float _e27 = _group_0_binding_0_fs.crop_left;
    float _e30 = _group_0_binding_0_fs.crop_top;
    vec2 p = (vec2(_e27, _e30) + result);
    vec2 _e33 = frame;
    vec2 c = (_e33 * 0.5);
    vec2 d_1 = (p - c);
    float _e39 = _group_0_binding_0_fs.straighten;
    float theta = radians(_e39);
    float s_1 = sin(theta);
    float k_1 = cos(theta);
    q = (c + vec2(((d_1.x * k_1) + (d_1.y * s_1)), ((-(d_1.x) * s_1) + (d_1.y * k_1))));
    uint _e59 = _group_0_binding_0_fs.flip_h;
    if ((_e59 == 1u)) {
        float _e64 = frame.x;
        float _e66 = q.x;
        q.x = (_e64 - _e66);
    }
    uint _e70 = _group_0_binding_0_fs.flip_v;
    if ((_e70 == 1u)) {
        float _e75 = frame.y;
        float _e77 = q.y;
        q.y = (_e75 - _e77);
    }
    uint _e81 = _group_0_binding_0_fs.rotate;
    switch(_e81) {
        case 90u: {
            float _e83 = q.y;
            float _e85 = frame.x;
            float _e87 = q.x;
            return vec2(_e83, (_e85 - _e87));
        }
        case 180u: {
            vec2 _e90 = frame;
            vec2 _e91 = q;
            return (_e90 - _e91);
        }
        case 270u: {
            float _e94 = frame.y;
            float _e96 = q.y;
            float _e99 = q.x;
            return vec2((_e94 - _e96), _e99);
        }
        default: {
            vec2 _e101 = q;
            return _e101;
        }
    }
}

vec4 catmull_rom_weights(float t) {
    float t2_ = (t * t);
    float t3_ = (t2_ * t);
    return vec4((((-(t3_) + (2.0 * t2_)) - t) * 0.5), ((((3.0 * t3_) - (5.0 * t2_)) + 2.0) * 0.5), ((((-3.0 * t3_) + (4.0 * t2_)) + t) * 0.5), ((t3_ - t2_) * 0.5));
}

vec3 sample_photo(vec2 u) {
    vec3 color_11 = vec3(0.0);
    int n = 0;
    vec3 row = vec3(0.0);
    int m = 0;
    float _e3 = _group_0_binding_0_fs.full_width;
    float _e6 = _group_0_binding_0_fs.full_height;
    vec2 full_2 = vec2(_e3, _e6);
    vec2 size_2 = vec2(uvec2(textureSize(_group_0_binding_1_fs, 0).xy));
    float _e13 = _group_0_binding_0_fs.tile_x;
    float _e16 = _group_0_binding_0_fs.tile_y;
    vec2 part = roundEven((vec2(_e13, _e16) * full_2));
    float _e22 = _group_0_binding_0_fs.tile_width;
    float _e25 = _group_0_binding_0_fs.tile_height;
    vec2 part_size = roundEven((vec2(_e22, _e25) * full_2));
    vec2 at_3 = clamp(u, vec2(0.5), (full_2 - vec2(0.5)));
    vec2 texel = clamp((((at_3 - part) * size_2) / part_size), vec2(0.5), (size_2 - vec2(0.5)));
    vec2 f = (texel - vec2(0.5));
    vec2 first = floor(f);
    vec4 _e51 = catmull_rom_weights((f.x - first.x));
    vec4 _e55 = catmull_rom_weights((f.y - first.y));
    ivec2 last = (ivec2(size_2) - ivec2(1));
    bool loop_init_2 = true;
    while(true) {
        if (!loop_init_2) {
            int _e114 = n;
            n = (_e114 + 1);
        }
        loop_init_2 = false;
        int _e65 = n;
        if ((_e65 < 4)) {
        } else {
            break;
        }
        {
            int _e72 = n;
            int y_4 = min(max(((int(first.y) - 1) + _e72), 0), last.y);
            row = vec3(0.0);
            m = 0;
            bool loop_init_3 = true;
            while(true) {
                if (!loop_init_3) {
                    int _e105 = m;
                    m = (_e105 + 1);
                }
                loop_init_3 = false;
                int _e82 = m;
                if ((_e82 < 4)) {
                } else {
                    break;
                }
                {
                    int _e89 = m;
                    int x_2 = min(max(((int(first.x) - 1) + _e89), 0), last.x);
                    vec3 _e94 = row;
                    int _e95 = m;
                    vec4 _e100 = texelFetch(_group_0_binding_1_fs, ivec2(x_2, y_4), 0);
                    row = (_e94 + (_e51[_e95] * _e100.xyz));
                }
            }
            vec3 _e107 = color_11;
            int _e108 = n;
            vec3 _e110 = row;
            color_11 = (_e107 + (_e55[_e108] * _e110));
        }
    }
    vec3 _e116 = color_11;
    return _e116;
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
    vec2 _e9 = result_position(in_.uv);
    vec2 _e10 = upright_position(_e9);
    vec3 _e11 = sample_photo(_e10);
    color = _e11;
    uint _e15 = _group_0_binding_0_fs.red_eye_count;
    if ((_e15 != 0u)) {
        vec3 _e18 = color;
        vec3 _e19 = red_eye(_e18, _e10);
        color = _e19;
    }
    float _e22 = _group_0_binding_0_fs.exposure_ev;
    if ((_e22 != 0.0)) {
        vec3 _e25 = color;
        float _e28 = _group_0_binding_0_fs.exposure_ev;
        vec3 _e29 = exposure(_e25, _e28);
        color = _e29;
    }
    float _e32 = _group_0_binding_0_fs.temperature;
    if (!((_e32 != 0.0))) {
        float _e40 = _group_0_binding_0_fs.tint;
        local = (_e40 != 0.0);
    } else {
        local = true;
    }
    bool _e44 = local;
    if (_e44) {
        vec3 _e45 = color;
        float _e48 = _group_0_binding_0_fs.temperature;
        float _e51 = _group_0_binding_0_fs.tint;
        vec3 _e52 = white_balance(_e45, _e48, _e51);
        color = _e52;
    }
    float _e55 = _group_0_binding_0_fs.highlights;
    if (!((_e55 != 0.0))) {
        float _e63 = _group_0_binding_0_fs.shadows;
        local_1 = (_e63 != 0.0);
    } else {
        local_1 = true;
    }
    bool _e67 = local_1;
    if (_e67) {
        float _e70 = _group_0_binding_0_fs.full_width;
        float _e73 = _group_0_binding_0_fs.full_height;
        vec2 at_4 = (_e10 / vec2(_e70, _e73));
        vec4 _e79 = textureLod(_group_0_binding_3_fs, vec2(at_4), 0.0);
        vec2 ab = _e79.xy;
        vec3 _e81 = color;
        float _e84 = _group_0_binding_0_fs.exposure_ev;
        float _e88 = log_luminance((_e81 * exp2(-(_e84))));
        float _e95 = _group_0_binding_0_fs.exposure_ev;
        base = (((ab.x * _e88) + ab.y) + _e95);
        float _e100 = _group_0_binding_0_fs.highlights;
        if ((_e100 != 0.0)) {
            float _e103 = base;
            float _e106 = _group_0_binding_0_fs.highlights;
            float _e107 = highlights_gain(_e103, _e106);
            vec3 _e108 = color;
            color = (_e108 * exp2(_e107));
            float _e111 = base;
            base = (_e111 + _e107);
        }
        float _e115 = _group_0_binding_0_fs.shadows;
        if ((_e115 != 0.0)) {
            vec3 _e118 = color;
            float _e119 = base;
            float _e122 = _group_0_binding_0_fs.shadows;
            float _e123 = shadows_gain(_e119, _e122);
            color = (_e118 * exp2(_e123));
        }
    }
    float _e128 = _group_0_binding_0_fs.contrast;
    if ((_e128 != 0.0)) {
        vec3 _e131 = color;
        float _e134 = _group_0_binding_0_fs.contrast;
        vec3 _e135 = contrast(_e131, _e134);
        color = _e135;
    }
    float _e138 = _group_0_binding_0_fs.saturation;
    if ((_e138 != 0.0)) {
        vec3 _e141 = color;
        float _e144 = _group_0_binding_0_fs.saturation;
        vec3 _e145 = saturation(_e141, _e144);
        color = _e145;
    }
    float _e148 = _group_0_binding_0_fs.black_and_white;
    if ((_e148 != 0.0)) {
        vec3 _e151 = color;
        float _e154 = _group_0_binding_0_fs.black_and_white;
        vec3 _e155 = black_and_white(_e151, _e154);
        color = _e155;
    }
    float _e158 = _group_0_binding_0_fs.sepia;
    if ((_e158 != 0.0)) {
        vec3 _e161 = color;
        float _e164 = _group_0_binding_0_fs.sepia;
        vec3 _e165 = sepia(_e161, _e164);
        color = _e165;
    }
    float _e168 = _group_0_binding_0_fs.vignette_amount;
    if ((_e168 != 0.0)) {
        float _e173 = _group_0_binding_0_fs.crop_pixels_width;
        float _e176 = _group_0_binding_0_fs.crop_pixels_height;
        vec2 crop_1 = vec2(_e173, _e176);
        vec3 _e178 = color;
        float _e181 = _group_0_binding_0_fs.vignette_amount;
        float _e184 = _group_0_binding_0_fs.vignette_size;
        vec3 _e186 = vignette(_e178, _e181, _e184, (_e9 / crop_1));
        color = _e186;
    }
    uint _e189 = _group_0_binding_0_fs.output_srgb;
    if ((_e189 == 1u)) {
        vec3 _e193 = color;
        color = (P3_TO_SRGB * _e193);
    }
    vec3 _e195 = color;
    vec3 _e196 = encode_transfer(_e195);
    _fs2p_location0 = vec4(clamp(_e196, vec3(0.0), vec3(1.0)), 1.0);
    return;
}

