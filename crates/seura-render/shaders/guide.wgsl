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
