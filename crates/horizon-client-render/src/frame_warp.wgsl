// Adaptive frame generation: a made frame is the last real one, moved to where the
// camera is now. Every pixel looks up where its point was in the last real frame:
// first as if the point were at the far plane (exact while the camera only turns),
// then corrected a few times through the last frame's depth for the camera's own
// movement. What the hand and the screen effects covered (the difference between the
// two kept frames) stays where it was on the screen, as they move with the camera.
@group(0) @binding(0) var world: texture_2d<f32>;
@group(0) @binding(1) var overlay: texture_2d<f32>;
@group(0) @binding(2) var depth: texture_depth_2d;
@group(0) @binding(3) var linear_sampler: sampler;

struct Warp {
    // This frame's clip space to the last real frame's.
    to_previous: mat4x4<f32>,
    // The last real frame's clip space to this frame's.
    to_current: mat4x4<f32>,
    // x: how far the overlay may differ from the world before it is kept on the
    // screen; y: 1 while the camera has not moved since the real frame.
    params: vec4<f32>,
};
@group(0) @binding(4) var<uniform> warp: Warp;

struct WarpOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex fn vs_warp(@builtin(vertex_index) i: u32) -> WarpOut {
    let p = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    var out: WarpOut;
    out.clip = vec4(p * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2(p.x, 1.0 - p.y);
    return out;
}

fn to_ndc(uv: vec2<f32>) -> vec2<f32> {
    return vec2(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0);
}

fn to_uv(clip: vec4<f32>) -> vec2<f32> {
    return vec2(0.5, -0.5) * clip.xy / clip.w + 0.5;
}

// A turn shows a strip the last frame never saw. Clamping smears its edge row across
// the strip; folding the neighbouring strip back once fills it with something that
// reads as scenery. Folding only once keeps far-off lookups from wrapping around.
fn fold(uv: vec2<f32>) -> vec2<f32> {
    let m = abs(uv);
    let folded = select(m, 2.0 - m, m > vec2(1.0));
    return clamp(folded, vec2(0.0), vec2(1.0));
}

// Reversed depth: 0 is the far plane and the sky.
fn depth_at(uv: vec2<f32>) -> f32 {
    let size = vec2<f32>(textureDimensions(depth));
    let p = clamp(vec2<i32>(floor(uv * size)), vec2(0), vec2<i32>(size) - 1);
    return textureLoad(depth, p, 0);
}

@fragment fn fs_warp(in: WarpOut) -> @location(0) vec4<f32> {
    let here = in.uv;
    let before = textureSampleLevel(world, linear_sampler, here, 0.0);
    let covered = textureSampleLevel(overlay, linear_sampler, here, 0.0);
    let difference = abs(covered.rgb - before.rgb);
    if max(difference.r, max(difference.g, difference.b)) > warp.params.x
        || warp.params.y > 0.5 {
        return vec4(covered.rgb, 1.0);
    }
    let far = warp.to_previous * vec4(to_ndc(here), 0.0, 1.0);
    var uv = here;
    if abs(far.w) > 1e-6 {
        uv = fold(to_uv(far));
    }
    for (var i = 0; i < 3; i++) {
        let d = depth_at(uv);
        if d <= 0.0 {
            break;
        }
        let landed = warp.to_current * vec4(to_ndc(uv), d, 1.0);
        if landed.w <= 1e-6 {
            break;
        }
        uv = fold(uv + (here - to_uv(landed)));
    }
    return vec4(textureSampleLevel(world, linear_sampler, uv, 0.0).rgb, 1.0);
}
