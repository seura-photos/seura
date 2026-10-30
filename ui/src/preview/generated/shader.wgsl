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

// Highlights and shadows each move a pixel's brightness by a gain in stops,
// weighed by its base (guide.wgsl): x, the base as linear light up to 1,
// raised to 1/2.2. At 100 or −100 the gain is up to this many stops.
const HIGHLIGHTS_SHADOWS_STOPS: f32 = 2.0;

fn tone_position(base: f32) -> f32 {
    return pow(clamp(exp2(base), 0.0, 1.0), 1.0 / PERCEPTUAL);
}

// seura.highlights: the gain for a pixel with this base, in stops: x³ of
// the full amount, so it works on bright areas, such as a sky, and barely
// on dark ones.
fn highlights_gain(base: f32, amount: f32) -> f32 {
    let x = tone_position(base);
    return amount / 100.0 * HIGHLIGHTS_SHADOWS_STOPS * x * x * x;
}

// seura.shadows: (1 − x)³ of the full amount, so it works on dark areas,
// such as a backlit face, and barely on bright ones.
fn shadows_gain(base: f32, amount: f32) -> f32 {
    let x = 1.0 - tone_position(base);
    return amount / 100.0 * HIGHLIGHTS_SHADOWS_STOPS * x * x * x;
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

// The luminance guide for highlights and shadows (technical design,
// "Neighborhood operations"): the photo's log luminance smoothed without
// crossing strong edges, by a guided filter guided by itself (He, Sun and
// Tang, 2010). Chosen over a bilateral grid and a plain blur in M4 slice 4:
// the blur haloes, and the grid needs 3D textures (journal, 2026-10-29).
//
// It's built once per photo, and again when white balance changes, from
// the photo scaled to fit 1024 pixels (seura_image::GUIDE_LONG_EDGE), in
// four passes. It holds two coefficients per guide pixel, a and b: a
// pixel's base, its surroundings' log luminance, is a × l + b, with a and b
// sampled bilinearly where the pixel is and l its own log luminance. Taking
// l from the full-size pixel keeps edges sharp however small the guide is.

// Luminance is floored here before its log, about 1/16 of a level of 8-bit
// black, so black doesn't count as infinitely dark.
const GUIDE_FLOOR: f32 = 0.000244140625;

// The filter's radius, as a fraction of the guide's long side: 20 pixels
// at 1024.
const GUIDE_RADIUS: f32 = 0.02;

// How much variation, in stops squared, counts as texture to smooth over
// rather than an edge to keep.
const GUIDE_EPSILON: f32 = 0.1;

fn log_luminance(color: vec3<f32>) -> f32 {
    return log2(max(dot(P3_LUMINANCE, color), GUIDE_FLOOR));
}

// The mean over a line of 2r + 1 pixels along `direction`; pixels past the
// edge repeat the edge's.
fn guide_box(at: vec2<i32>, direction: vec2<i32>) -> vec2<f32> {
    let size = vec2<i32>(textureDimensions(stage));
    let r = max(1, i32(round(f32(max(size.x, size.y)) * GUIDE_RADIUS)));
    var sum = vec2<f32>(0.0);
    for (var i = -r; i <= r; i++) {
        let neighbour = clamp(at + direction * i, vec2<i32>(0), size - vec2<i32>(1));
        sum += textureLoad(stage, neighbour, 0).xy;
    }
    return sum / f32(2 * r + 1);
}

// One pass of building the guide, at the guide's size, reading the last
// pass's result from `stage`:
// 1. Log luminance l, and l², from the scaled photo with white balance.
//    Exposure is added to the base later: it shifts every l equally.
// 2. Their means across (and, the second time, a and b's).
// 3. Their means down, then each pixel's a and b: a = v / (v + ε), where v
//    is l's variance around it, and b = mean − a × mean.
// 4. a and b's means down, after pass 2 across: the guide.
fn guide_pass(position: vec4<f32>) -> vec4<f32> {
    let at = vec2<i32>(position.xy);
    switch params.draw_pass {
        case 1u: {
            var color = textureLoad(stage, at, 0).rgb;
            if params.temperature != 0.0 || params.tint != 0.0 {
                color = white_balance(color, params.temperature, params.tint);
            }
            let l = log_luminance(color);
            return vec4<f32>(l, l * l, 0.0, 1.0);
        }
        case 2u: {
            return vec4<f32>(guide_box(at, vec2<i32>(1, 0)), 0.0, 1.0);
        }
        case 3u: {
            let mean = guide_box(at, vec2<i32>(0, 1));
            let variance = max(mean.y - mean.x * mean.x, 0.0);
            let a = variance / (variance + GUIDE_EPSILON);
            return vec4<f32>(a, mean.x - a * mean.x, 0.0, 1.0);
        }
        default: {
            return vec4<f32>(guide_box(at, vec2<i32>(0, 1)), 0.0, 1.0);
        }
    }
}

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
