// Tone and color, the edit format's stage 3. Each operation takes and
// returns linear Display P3.

// seura.exposure: brightness in photographic stops; multiplies linear light
// by 2^ev.
fn exposure(color: vec3<f32>, ev: f32) -> vec3<f32> {
    return color * exp2(ev);
}
