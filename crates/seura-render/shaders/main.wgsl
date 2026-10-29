// The render pass: draws the photo with its edits, in the canonical order
// (decision 0003), then encodes it for the preview or for export.

// Every core operation's parameters but red-eye's regions, in the edit
// format's canonical order (seura-render's `Settings`). An operation whose
// parameters are neutral is skipped: the branch is on a uniform, so it costs
// nothing, and there are no separate flags to keep in step. Operations arrive
// one by one in M4; so far only exposure is rendered.
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
    if params.output_srgb == 1u {
        color = P3_TO_SRGB * color;
    }
    // Colors outside the output space are clipped, for now.
    return vec4<f32>(clamp(encode_transfer(color), vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
