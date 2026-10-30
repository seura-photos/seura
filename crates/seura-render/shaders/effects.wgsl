// The effects stage (EDT-6), after tone and color, in the edit format's
// order: black and white, sepia, then the vignette.

// seura.blackAndWhite: each channel toward the pixel's luminance by
// strength / 100, so 100 is gray and the brightness stays.
fn black_and_white(color: vec3<f32>, strength: f32) -> vec3<f32> {
    let y = dot(P3_LUMINANCE, color);
    return color + (vec3<f32>(y) - color) * (strength / 100.0);
}

// Sepia's tone, (1.25, 0.97, 0.56) scaled so its luminance is 1.
const SEPIA_TONE = vec3<f32>(1.2479966, 0.9684454, 0.5591025);

// seura.sepia: each channel toward the pixel's luminance in sepia's tone,
// by strength / 100, so 100 is a warm brown monochrome as bright as the
// photo. Nothing is clipped: white comes out with red above 1.
fn sepia(color: vec3<f32>, strength: f32) -> vec3<f32> {
    let y = dot(P3_LUMINANCE, color);
    return color + (y * SEPIA_TONE - color) * (strength / 100.0);
}

// seura.vignette, at position at of the cropped frame, as fractions of it:
// r is 0 at its center, 1 at the middle of each edge and √2 at the corners,
// an ellipse fitted to the frame. It starts at r = 1 − size / 100 and grows
// smoothly to the full amount at the corners, scaling toward black when
// negative and blending toward white when positive, so bright colors keep
// their hue.
fn vignette(color: vec3<f32>, amount: f32, size: f32, at: vec2<f32>) -> vec3<f32> {
    let r = length(at * 2.0 - vec2<f32>(1.0));
    let start = 1.0 - size / 100.0;
    let t = clamp((r - start) / (sqrt(2.0) - start), 0.0, 1.0);
    let m = amount / 100.0 * t * t * (3.0 - 2.0 * t);
    if m < 0.0 {
        return color * (1.0 + m);
    }
    return color + (vec3<f32>(1.0) - color) * m;
}
