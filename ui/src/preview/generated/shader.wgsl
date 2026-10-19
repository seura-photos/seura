// Generated from crates/seura-render/shaders by `npm run shaders`. Don't edit.

// Shared color functions. Seura works in linear light with Display P3
// primaries (the edit format's working space); Display P3 and sRGB share
// the sRGB transfer function and white point.

// Linear light to the encoded values a display or file expects, extended to
// values outside 0 to 1 by symmetry.
fn encode_transfer(linear: vec3<f32>) -> vec3<f32> {
    let v = abs(linear);
    let low = v * 12.92;
    let high = 1.055 * pow(v, vec3<f32>(1.0 / 2.4)) - 0.055;
    return sign(linear) * select(high, low, v <= vec3<f32>(0.0031308));
}

// Linear Display P3 to linear sRGB, from the two spaces' primaries (D65).
// Columns, as WGSL writes matrices.
const P3_TO_SRGB = mat3x3<f32>(
    vec3<f32>(1.2249401, -0.0420569, -0.0196376),
    vec3<f32>(-0.2249404, 1.0420571, -0.0786361),
    vec3<f32>(0.0, 0.0, 1.0982735),
);

// Tone and color, the edit format's stage 3. Each operation takes and
// returns linear Display P3.

// seura.exposure: brightness in photographic stops; multiplies linear light
// by 2^ev.
fn exposure(color: vec3<f32>, ev: f32) -> vec3<f32> {
    return color * exp2(ev);
}

// The render pass: draws the photo with its edits, in the canonical order
// (decision 0003), then encodes it for the preview or for export.

struct Params {
    // seura.exposure's ev; 0 is neutral.
    exposure_ev: f32,
    // 0: Display P3, for the preview. 1: sRGB, for export (EXP-5).
    output_srgb: u32,
    _pad0: u32,
    _pad1: u32,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var photo: texture_2d<f32>;
@group(0) @binding(2) var photo_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    // 0,0 is the photo's top left corner.
    @location(0) uv: vec2<f32>,
}

// One triangle covering the whole target; no vertex buffer.
@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VertexOutput {
    let x = f32((index << 1u) & 2u);
    let y = f32(index & 2u);
    var out: VertexOutput;
    out.position = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    out.uv = vec2<f32>(x, y);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let sample = textureSampleLevel(photo, photo_sampler, in.uv, 0.0);
    var color = exposure(sample.rgb, params.exposure_ev);
    if params.output_srgb == 1u {
        color = P3_TO_SRGB * color;
    }
    // Colors outside the output space are clipped, for now.
    return vec4<f32>(clamp(encode_transfer(color), vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
