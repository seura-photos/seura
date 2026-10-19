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
