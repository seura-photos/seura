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
// returns linear Display P3. The formulas are the edit format's appendix,
// written from these.

// The luminance of linear Display P3 (D65): the middle row of its matrix to
// CIE XYZ.
const P3_LUMINANCE = vec3<f32>(0.2289746, 0.6917385, 0.0792869);

// Middle gray in linear light, the point contrast turns around.
const MIDDLE_GRAY: f32 = 0.18;

// The power that makes linear light roughly perceptually even, for shaping
// contrast's curve.
const PERCEPTUAL: f32 = 2.2;

// seura.exposure: brightness in photographic stops; multiplies linear light
// by 2^ev.
fn exposure(color: vec3<f32>, ev: f32) -> vec3<f32> {
    return color * exp2(ev);
}

// seura.whiteBalance: a gain on each channel. Temperature (t = temperature
// / 100) shifts red up and blue down by t/2 stops each; tint (n = tint /
// 100) shifts green down by n/2 stops, toward magenta. The gains are then
// scaled together so gray keeps its luminance.
fn white_balance(color: vec3<f32>, temperature: f32, tint: f32) -> vec3<f32> {
    let t = temperature / 100.0;
    let n = tint / 100.0;
    let gains = exp2(vec3<f32>(t / 2.0, -n / 2.0, -t / 2.0));
    return color * (gains / dot(P3_LUMINANCE, gains));
}

// seura.contrast: an S-shaped curve around middle gray, on luminance so hues
// don't shift. On luminance raised to 1/2.2, x, with middle gray at p, the
// curve is p (x / p)^g below p and 1 − (1 − p) ((1 − x) / (1 − p))^g above
// it, where g = 2^(amount / 100) is its slope at p. Black and white stay
// where they are; light above white is left as it is. Each channel is then
// scaled by the luminance's change.
fn contrast(color: vec3<f32>, amount: f32) -> vec3<f32> {
    let y = dot(P3_LUMINANCE, color);
    if y <= 0.0 {
        return color;
    }
    let x = pow(y, 1.0 / PERCEPTUAL);
    if x >= 1.0 {
        return color;
    }
    let g = exp2(amount / 100.0);
    let p = pow(MIDDLE_GRAY, 1.0 / PERCEPTUAL);
    var curved: f32;
    if x < p {
        curved = p * pow(x / p, g);
    } else {
        curved = 1.0 - (1.0 - p) * pow((1.0 - x) / (1.0 - p), g);
    }
    return color * (pow(curved, PERCEPTUAL) / y);
}

// seura.saturation: moves each channel away from the pixel's luminance by
// s = 1 + amount / 100, so −100 is gray and 100 doubles the color. Colors
// outside Display P3 have components below 0, which are kept, and a vivid
// color made more so can go below 0 itself (the formulas appendix,
// "Working space").
fn saturation(color: vec3<f32>, amount: f32) -> vec3<f32> {
    let y = dot(P3_LUMINANCE, color);
    let s = 1.0 + amount / 100.0;
    return vec3<f32>(y) + (color - vec3<f32>(y)) * s;
}

// The render pass: draws the photo with its edits, in the canonical order
// (decision 0003), then encodes it for the preview or for export.

// Every core operation's parameters but red-eye's regions, in the edit
// format's canonical order (seura-render's `Settings`). An operation whose
// parameters are neutral is skipped: the branch is on a uniform, so it costs
// nothing, and there are no separate flags to keep in step. Operations arrive
// one by one in M4; so far exposure, white balance, contrast and saturation
// are rendered.
struct Params {
    // 0: Display P3, for the preview. 1: sRGB, for export (EXP-5).
    output_srgb: u32,
    // seura.orient: 0, 90, 180, or 270 degrees clockwise; flips are 0 or 1.
    rotate: u32,
    flip_h: u32,
    flip_v: u32,
    // seura.straighten, degrees clockwise.
    straighten: f32,
    // seura.crop, fractions of the straightened photo; 0, 0, 1, 1 is all of it.
    crop_x: f32,
    crop_y: f32,
    crop_width: f32,
    crop_height: f32,
    // seura.exposure's ev; 0 is neutral.
    exposure_ev: f32,
    // seura.whiteBalance, −100 to 100.
    temperature: f32,
    tint: f32,
    // seura.highlights, seura.shadows, seura.contrast, seura.saturation:
    // −100 to 100.
    highlights: f32,
    shadows: f32,
    contrast: f32,
    saturation: f32,
    // seura.blackAndWhite and seura.sepia: 0 to 100.
    black_and_white: f32,
    sepia: f32,
    // seura.vignette: amount −100 to 100, size 0 to 100.
    vignette_amount: f32,
    vignette_size: f32,
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
    var color = sample.rgb;
    if params.exposure_ev != 0.0 {
        color = exposure(color, params.exposure_ev);
    }
    if params.temperature != 0.0 || params.tint != 0.0 {
        color = white_balance(color, params.temperature, params.tint);
    }
    if params.contrast != 0.0 {
        color = contrast(color, params.contrast);
    }
    if params.saturation != 0.0 {
        color = saturation(color, params.saturation);
    }
    if params.output_srgb == 1u {
        color = P3_TO_SRGB * color;
    }
    // Colors outside the output space are clipped, for now.
    return vec4<f32>(clamp(encode_transfer(color), vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
