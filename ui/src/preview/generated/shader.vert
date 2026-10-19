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

void main() {
    uint index = uint(gl_VertexID);
    VertexOutput out_ = VertexOutput(vec4(0.0), vec2(0.0));
    float x = float(((index << 1u) & 2u));
    float y = float((index & 2u));
    out_.position = vec4(((x * 2.0) - 1.0), (1.0 - (y * 2.0)), 0.0, 1.0);
    out_.uv = vec2(x, y);
    VertexOutput _e24 = out_;
    gl_Position = _e24.position;
    _vs2fs_location0 = _e24.uv;
    return;
}

