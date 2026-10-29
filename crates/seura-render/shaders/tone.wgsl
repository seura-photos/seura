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
// s = 1 + amount / 100, so −100 is gray and 100 doubles the color. Light
// can't be negative, so channels stop at 0.
fn saturation(color: vec3<f32>, amount: f32) -> vec3<f32> {
    let y = dot(P3_LUMINANCE, color);
    let s = 1.0 + amount / 100.0;
    return max(vec3<f32>(y) + (color - vec3<f32>(y)) * s, vec3<f32>(0.0));
}
