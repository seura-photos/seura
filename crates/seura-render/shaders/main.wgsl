// The render pass: draws the photo with its edits, in the canonical order
// (decision 0003), then encodes it for the preview or for export.

// Every core operation's parameters but red-eye's regions, in the edit
// format's canonical order (seura-render's `Settings`). An operation whose
// parameters are neutral is skipped: the branch is on a uniform, so it costs
// nothing, and there are no separate flags to keep in step. Operations arrive
// one by one in M4; so far orient, exposure, white balance, highlights,
// shadows, contrast and saturation are rendered.
struct Params {
    // 0: Display P3, for the preview. 1: sRGB, for export (EXP-5).
    output_srgb: u32,
    // seura.orient: 0, 90, 180, or 270 degrees clockwise, then flips (0 or
    // 1) as the turned photo is seen.
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
    // Which pass this draw is: 0 draws the photo; 1 to 4 build the luminance
    // guide (guide.wgsl).
    draw_pass: u32,
    // The part of the upright photo this draw covers, as fractions of it;
    // 0, 0, 1, 1 is all of it. Export draws large photos in tiles.
    tile_x: f32,
    tile_y: f32,
    tile_width: f32,
    tile_height: f32,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var photo: texture_2d<f32>;
@group(0) @binding(2) var photo_sampler: sampler;
// The luminance guide's a and b, for highlights and shadows, sampled
// bilinearly.
@group(0) @binding(3) var guide: texture_2d<f32>;
@group(0) @binding(4) var guide_sampler: sampler;
// While the guide is built, the last pass's result.
@group(0) @binding(5) var stage: texture_2d<f32>;

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

// Geometry is a coordinate transform when sampling (technical design,
// "Pipeline"): where a point of the drawn photo, as fractions of it, is in
// the upright photo. seura.orient turns the photo clockwise, then flips it
// as it's then seen, so this undoes the flips, then the turn. A part of the
// photo maps the same way within itself, since its part of the upright
// photo is a rectangle too.
fn upright_position(uv: vec2<f32>) -> vec2<f32> {
    var p = uv;
    if params.flip_h == 1u {
        p.x = 1.0 - p.x;
    }
    if params.flip_v == 1u {
        p.y = 1.0 - p.y;
    }
    switch params.rotate {
        case 90u: {
            return vec2<f32>(p.y, 1.0 - p.x);
        }
        case 180u: {
            return vec2<f32>(1.0 - p.x, 1.0 - p.y);
        }
        case 270u: {
            return vec2<f32>(1.0 - p.y, p.x);
        }
        default: {
            return p;
        }
    }
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if params.draw_pass != 0u {
        return guide_pass(in.position);
    }
    let position = upright_position(in.uv);
    let sample = textureSampleLevel(photo, photo_sampler, position, 0.0);
    var color = sample.rgb;
    if params.exposure_ev != 0.0 {
        color = exposure(color, params.exposure_ev);
    }
    if params.temperature != 0.0 || params.tint != 0.0 {
        color = white_balance(color, params.temperature, params.tint);
    }
    if params.highlights != 0.0 || params.shadows != 0.0 {
        // Where this pixel is in the whole photo, which the guide covers.
        let at = vec2<f32>(params.tile_x, params.tile_y)
            + position * vec2<f32>(params.tile_width, params.tile_height);
        let ab = textureSampleLevel(guide, guide_sampler, at, 0.0).xy;
        // The guide was built without exposure, which adds ev to every log
        // luminance.
        let l = log_luminance(color * exp2(-params.exposure_ev));
        var base = ab.x * l + ab.y + params.exposure_ev;
        if params.highlights != 0.0 {
            let gain = highlights_gain(base, params.highlights);
            color *= exp2(gain);
            base += gain;
        }
        if params.shadows != 0.0 {
            color *= exp2(shadows_gain(base, params.shadows));
        }
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
