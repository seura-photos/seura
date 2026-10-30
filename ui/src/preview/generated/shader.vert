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

smooth out vec2 _vs2fs_location0;

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
    float t_1 = (temperature / 100.0);
    float n_1 = (tint / 100.0);
    vec3 gains = exp2(vec3((t_1 / 2.0), (-(n_1) / 2.0), (-(t_1) / 2.0)));
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

vec4 catmull_rom_weights(float t) {
    float t2_ = (t * t);
    float t3_ = (t2_ * t);
    return vec4((((-(t3_) + (2.0 * t2_)) - t) * 0.5), ((((3.0 * t3_) - (5.0 * t2_)) + 2.0) * 0.5), ((((-3.0 * t3_) + (4.0 * t2_)) + t) * 0.5), ((t3_ - t2_) * 0.5));
}

void main() {
    uint index = uint(gl_VertexID);
    VertexOutput out_ = VertexOutput(vec4(0.0), vec2(0.0));
    float x_2 = float(((index << 1u) & 2u));
    float y_2 = float((index & 2u));
    out_.position = vec4(((x_2 * 2.0) - 1.0), (1.0 - (y_2 * 2.0)), 0.0, 1.0);
    out_.uv = vec2(x_2, y_2);
    VertexOutput _e24 = out_;
    gl_Position = _e24.position;
    _vs2fs_location0 = _e24.uv;
    return;
}

