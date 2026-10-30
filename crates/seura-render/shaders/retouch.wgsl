// The retouch stage (EDT-7), before tone and color, on the photo's own
// colors.

// seura.redEye at position, in pixels of the upright photo, where geometry
// sampled it (formulas appendix, seura.redEye.md). Each circle's weight is
// 1 inside four fifths of its radius and falls smoothly to 0 at its edge;
// overlapping circles take the largest. How red the pixel is, its red's
// lead over the larger of green and blue as a share of its red, decides how
// far its red comes down to that larger one: from 0.7, fully at 0.9, so
// red pupils change and skin and irises don't. A circle with no radius
// corrects nothing.
fn red_eye(color: vec3<f32>, position: vec2<f32>) -> vec3<f32> {
    let full = vec2<f32>(params.full_width, params.full_height);
    let shorter = min(full.x, full.y);
    var weight = 0.0;
    for (var i = 0u; i < min(params.red_eye_count, 64u); i++) {
        let region = params.red_eye[i];
        if region.z <= 0.0 {
            continue;
        }
        let d = distance(position, region.xy * full) / (region.z * shorter);
        weight = max(weight, 1.0 - smoothstep(0.8, 1.0, d));
    }
    let m = max(color.g, color.b);
    if weight == 0.0 || color.r <= 0.0 || color.r <= m {
        return color;
    }
    let k = smoothstep(0.7, 0.9, (color.r - m) / color.r);
    return vec3<f32>(color.r + (m - color.r) * weight * k, color.g, color.b);
}
