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
    uint index = uint(gl_VertexID);
    VertexOutput out_ = VertexOutput(vec4(0.0), vec2(0.0));
    float x_1 = float(((index << 1u) & 2u));
    float y_2 = float((index & 2u));
    out_.position = vec4(((x_1 * 2.0) - 1.0), (1.0 - (y_2 * 2.0)), 0.0, 1.0);
    out_.uv = vec2(x_1, y_2);
    VertexOutput _e24 = out_;
    gl_Position = _e24.position;
    _vs2fs_location0 = _e24.uv;
    return;
}

