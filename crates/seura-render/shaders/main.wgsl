// The render pass: draws the photo with its edits, in the canonical order
// (decision 0003), then encodes it for the preview or for export.

// Every core operation's parameters but red-eye's regions, in the edit
// format's canonical order (seura-render's `Settings`). An operation whose
// parameters are neutral is skipped: the branch is on a uniform, so it costs
// nothing, and there are no separate flags to keep in step. Operations arrive
// one by one in M4; so far geometry (orient, straighten, crop), exposure,
// white balance, highlights, shadows, contrast and saturation are rendered.
struct Params {
    // 0: Display P3, for the preview. 1: sRGB, for export (EXP-5).
    output_srgb: u32,
    // seura.orient: 0, 90, 180, or 270 degrees clockwise, then flips (0 or
    // 1) as the turned photo is seen.
    rotate: u32,
    flip_h: u32,
    flip_v: u32,
    // seura.straighten, degrees clockwise as seen.
    straighten: f32,
    // seura.crop, fractions of the oriented photo; 0, 0, 1, 1 is all of it.
    // Drawn from crop_left and the others below, in whole pixels.
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
    // The upright photo's full size in pixels, which geometry is measured
    // in, whatever size the photo texture is.
    full_width: f32,
    full_height: f32,
    // The part of the result this draw covers, as fractions of it; 0, 0,
    // 1, 1 is all of it.
    view_x: f32,
    view_y: f32,
    view_width: f32,
    view_height: f32,
    // The crop in whole pixels of the oriented photo (formulas appendix,
    // geometry.md, step 1).
    crop_left: f32,
    crop_top: f32,
    crop_pixels_width: f32,
    crop_pixels_height: f32,
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
// "Pipeline"; formulas appendix, geometry.md): where a point of this draw,
// as fractions of it, is in the upright photo, in the photo's own pixels.
// Through the crop, the turn back about the oriented photo's center, then
// seura.orient's flips undone and its turn.
fn upright_position(uv: vec2<f32>) -> vec2<f32> {
    let full = vec2<f32>(params.full_width, params.full_height);
    var frame = full;
    if params.rotate == 90u || params.rotate == 270u {
        frame = full.yx;
    }
    // Where this point is in the result, then in the oriented photo.
    let crop = vec2<f32>(params.crop_pixels_width, params.crop_pixels_height);
    let view = round(vec2<f32>(params.view_x, params.view_y) * crop);
    let view_size = round(vec2<f32>(params.view_width, params.view_height) * crop);
    let p = vec2<f32>(params.crop_left, params.crop_top) + view + uv * view_size;
    // The turn back: counter-clockwise as seen, with y down.
    let c = frame * 0.5;
    let d = p - c;
    let theta = radians(params.straighten);
    let s = sin(theta);
    let k = cos(theta);
    var q = c + vec2<f32>(d.x * k + d.y * s, -d.x * s + d.y * k);
    if params.flip_h == 1u {
        q.x = frame.x - q.x;
    }
    if params.flip_v == 1u {
        q.y = frame.y - q.y;
    }
    switch params.rotate {
        case 90u: {
            return vec2<f32>(q.y, frame.x - q.x);
        }
        case 180u: {
            return frame - q;
        }
        case 270u: {
            return vec2<f32>(frame.y - q.y, q.x);
        }
        default: {
            return q;
        }
    }
}

// The four Catmull-Rom weights for a position t of the way from the second
// pixel to the third.
fn catmull_rom_weights(t: f32) -> vec4<f32> {
    let t2 = t * t;
    let t3 = t2 * t;
    return vec4<f32>(
        (-t3 + 2.0 * t2 - t) * 0.5,
        (3.0 * t3 - 5.0 * t2 + 2.0) * 0.5,
        (-3.0 * t3 + 4.0 * t2 + t) * 0.5,
        (t3 - t2) * 0.5,
    );
}

// The photo at position u of the upright photo, in its pixels, with
// Catmull-Rom (geometry.md). A position past the outermost pixel centers
// takes the nearest one, so past the edge is the edge pixel; pixels the
// filter reaches past the edge repeat it. The photo texture holds the
// part tile_x... of the photo, at its own resolution: the whole photo
// fitted to the screen, or a tile of export with room for the filter.
fn sample_photo(u: vec2<f32>) -> vec3<f32> {
    let full = vec2<f32>(params.full_width, params.full_height);
    let size = vec2<f32>(textureDimensions(photo));
    let part = round(vec2<f32>(params.tile_x, params.tile_y) * full);
    let part_size = round(vec2<f32>(params.tile_width, params.tile_height) * full);
    let at = clamp(u, vec2<f32>(0.5), full - vec2<f32>(0.5));
    let texel = clamp((at - part) * size / part_size, vec2<f32>(0.5), size - vec2<f32>(0.5));
    let f = texel - vec2<f32>(0.5);
    let first = floor(f);
    let wx = catmull_rom_weights(f.x - first.x);
    let wy = catmull_rom_weights(f.y - first.y);
    let last = vec2<i32>(size) - vec2<i32>(1);
    var color = vec3<f32>(0.0);
    for (var n = 0; n < 4; n++) {
        let y = clamp(i32(first.y) - 1 + n, 0, last.y);
        var row = vec3<f32>(0.0);
        for (var m = 0; m < 4; m++) {
            let x = clamp(i32(first.x) - 1 + m, 0, last.x);
            row += wx[m] * textureLoad(photo, vec2<i32>(x, y), 0).rgb;
        }
        color += wy[n] * row;
    }
    return color;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if params.draw_pass != 0u {
        return guide_pass(in.position);
    }
    let position = upright_position(in.uv);
    var color = sample_photo(position);
    if params.exposure_ev != 0.0 {
        color = exposure(color, params.exposure_ev);
    }
    if params.temperature != 0.0 || params.tint != 0.0 {
        color = white_balance(color, params.temperature, params.tint);
    }
    if params.highlights != 0.0 || params.shadows != 0.0 {
        // Where this pixel is in the whole photo, which the guide covers.
        let at = position / vec2<f32>(params.full_width, params.full_height);
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
