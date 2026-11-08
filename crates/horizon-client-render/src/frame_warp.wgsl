// Frame generation: a frame between the last two real ones, A and B (`frame_warp.rs`).
//
// The world: every pixel of the made frame looks up where its point is in B, through
// B's depth (a few fixed-point steps from where the point would be at the far plane,
// exact for a turn), and where B never saw it (the edge a turn uncovers, what was behind
// something), where it is in A. What is in front of the world on the screen (the hand,
// the HUD, the vignette, a translucent chat box) is per pixel `frame = k * world + c`;
// with A's and B's frames and worlds the two give k, and the made frame is B's frame
// with its world swapped for the made one: `frame_b + k * (world - world_b)`.
@group(0) @binding(0) var frame_a: texture_2d<f32>;
@group(0) @binding(1) var frame_b: texture_2d<f32>;
@group(0) @binding(2) var world_a: texture_2d<f32>;
@group(0) @binding(3) var world_b: texture_2d<f32>;
// Reversed depth (0: the far plane and the sky), packed (`unpack_depth`).
@group(0) @binding(4) var depth_a: texture_2d<f32>;
@group(0) @binding(5) var depth_b: texture_2d<f32>;
@group(0) @binding(6) var linear_sampler: sampler;

struct Interp {
    // The made frame's clip space to A's and back, the same for B.
    f_to_a: mat4x4<f32>,
    a_to_f: mat4x4<f32>,
    f_to_b: mat4x4<f32>,
    b_to_f: mat4x4<f32>,
    // A's clip space to B's: whether what A saw is still where it was.
    a_to_b: mat4x4<f32>,
    // x: how far from A (0) to B (1); y: how far the frame may differ from the world
    // before a pixel counts as covered; z: 1 shows which frames are made (a debug tint);
    // w: the made frame's depth a dozen blocks away (nearer, A is asked too).
    params: vec4<f32>,
    // xy: the screen's pixels the frame is drawn in; zw: the kept frames' size.
    size: vec4<f32>,
};
@group(0) @binding(7) var<uniform> u: Interp;

struct InterpOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex fn vs_interp(@builtin(vertex_index) i: u32) -> InterpOut {
    let p = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    var out: InterpOut;
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

fn texel(size: vec2<u32>, uv: vec2<f32>) -> vec2<i32> {
    let s = vec2<i32>(size);
    return clamp(vec2<i32>(floor(uv * vec2<f32>(size))), vec2(0), s - 1);
}

// The depth copies hold each float's bits in the four bytes of a colour.
fn unpack_depth(c: vec4<f32>) -> f32 {
    let b = vec4<u32>(round(c * 255.0));
    return bitcast<f32>(b.x | (b.y << 8u) | (b.z << 16u) | (b.w << 24u));
}

fn depth_at_a(uv: vec2<f32>) -> f32 {
    return unpack_depth(textureLoad(depth_a, texel(textureDimensions(depth_a), uv), 0));
}

fn depth_at_b(uv: vec2<f32>) -> f32 {
    return unpack_depth(textureLoad(depth_b, texel(textureDimensions(depth_b), uv), 0));
}

fn inside(uv: vec2<f32>) -> bool {
    return all(uv >= vec2(0.0)) && all(uv <= vec2(1.0));
}

// Where `here` (the made frame's uv) is in A or B: the uv there, the point's depth there
// and in the made frame, and whether it lands back on `here` (within a pixel and a half
// of the kept frames).
struct Found {
    uv: vec2<f32>,
    depth: f32,
    z: f32,
    ok: bool,
};

const STEPS: i32 = 4;

fn find_in_b(here: vec2<f32>) -> Found {
    var uv = here;
    let far = u.f_to_b * vec4(to_ndc(here), 0.0, 1.0);
    if far.w > 1e-6 {
        uv = to_uv(far);
    }
    for (var i = 0; i < STEPS; i++) {
        let p = u.b_to_f * vec4(to_ndc(uv), depth_at_b(uv), 1.0);
        if p.w <= 1e-6 {
            break;
        }
        uv = uv + (here - to_uv(p));
    }
    let d = depth_at_b(uv);
    let p = u.b_to_f * vec4(to_ndc(uv), d, 1.0);
    var found: Found;
    found.uv = uv;
    found.depth = d;
    found.z = p.z / max(p.w, 1e-6);
    let miss = length((here - to_uv(p)) * u.size.zw);
    found.ok = p.w > 1e-6 && inside(uv) && miss < 1.5;
    return found;
}

fn find_in_a(here: vec2<f32>) -> Found {
    var uv = here;
    let far = u.f_to_a * vec4(to_ndc(here), 0.0, 1.0);
    if far.w > 1e-6 {
        uv = to_uv(far);
    }
    for (var i = 0; i < STEPS; i++) {
        let p = u.a_to_f * vec4(to_ndc(uv), depth_at_a(uv), 1.0);
        if p.w <= 1e-6 {
            break;
        }
        uv = uv + (here - to_uv(p));
    }
    let d = depth_at_a(uv);
    let p = u.a_to_f * vec4(to_ndc(uv), d, 1.0);
    var found: Found;
    found.uv = uv;
    found.depth = d;
    found.z = p.z / max(p.w, 1e-6);
    let miss = length((here - to_uv(p)) * u.size.zw);
    found.ok = p.w > 1e-6 && inside(uv) && miss < 1.5;
    return found;
}

// Catmull-Rom from five bilinear taps (the corners left out): sharper than one bilinear
// tap, so that made frames don't look softer than the real ones between them.
fn sharp_b(uv: vec2<f32>) -> vec3<f32> {
    let size = vec2<f32>(textureDimensions(world_b));
    let p = uv * size - 0.5;
    let t = fract(p);
    let start = floor(p) + 0.5;
    let w0 = t * (-0.5 + t * (1.0 - 0.5 * t));
    let w1 = 1.0 + t * t * (-2.5 + 1.5 * t);
    let w2 = t * (0.5 + t * (2.0 - 1.5 * t));
    let w3 = t * t * (-0.5 + 0.5 * t);
    let w12 = w1 + w2;
    let c0 = (start - 1.0) / size;
    let c3 = (start + 2.0) / size;
    let c12 = (start + w2 / w12) / size;
    var c = textureSampleLevel(world_b, linear_sampler, vec2(c12.x, c0.y), 0.0).rgb * (w12.x * w0.y);
    c += textureSampleLevel(world_b, linear_sampler, vec2(c0.x, c12.y), 0.0).rgb * (w0.x * w12.y);
    c += textureSampleLevel(world_b, linear_sampler, c12, 0.0).rgb * (w12.x * w12.y);
    c += textureSampleLevel(world_b, linear_sampler, vec2(c3.x, c12.y), 0.0).rgb * (w3.x * w12.y);
    c += textureSampleLevel(world_b, linear_sampler, vec2(c12.x, c3.y), 0.0).rgb * (w12.x * w3.y);
    let total = w12.x * w0.y + w0.x * w12.y + w12.x * w12.y + w3.x * w12.y + w12.x * w3.y;
    return max(c / total, vec3(0.0));
}

// What A saw at `a` is still there in B (seen, or hidden behind something nearer): it
// stands still. Where B sees further, it moved away (a mob).
fn still_in_b(a: Found) -> bool {
    let q = u.a_to_b * vec4(to_ndc(a.uv), a.depth, 1.0);
    if q.w <= 1e-6 {
        return false;
    }
    let at = to_uv(q);
    if !inside(at) {
        return true;
    }
    return depth_at_b(at) >= (q.z / q.w) * 0.97;
}

fn made_world(here: vec2<f32>) -> vec3<f32> {
    let b = find_in_b(here);
    if b.ok {
        // Near the camera a step aside shows what B saw at a slant or not at all: A's
        // point is in front of B's there, unless A's moved away since.
        if b.z > u.params.w {
            let a = find_in_a(here);
            if a.ok && a.z > b.z * 1.03 && still_in_b(a) {
                return textureSampleLevel(world_a, linear_sampler, a.uv, 0.0).rgb;
            }
        }
        return sharp_b(b.uv);
    }
    let a = find_in_a(here);
    if a.ok {
        return textureSampleLevel(world_a, linear_sampler, a.uv, 0.0).rgb;
    }
    // Neither saw it (a fast turn's far edge): B's nearest scenery.
    return textureSampleLevel(world_b, linear_sampler, clamp(b.uv, vec2(0.0), vec2(1.0)), 0.0).rgb;
}

// How much of the world shows through what is drawn over it at `here` (the k of
// `frame = k * world + c`): from how the frame changed with the world between A and B,
// fitted over the pixel and its four neighbours (a vignette or a chat box is smooth, and
// a pixel whose world stayed the same tells nothing). 0 where the overlay itself changed
// (new text, another slot selected): it is B's, kept still.
fn show_through(here: vec2<f32>) -> f32 {
    let texel_size = 1.0 / vec2<f32>(textureDimensions(frame_b));
    var offsets = array<vec2<f32>, 5>(
        vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(-1.0, 0.0), vec2(0.0, 1.0), vec2(0.0, -1.0)
    );
    var num = 0.0;
    var den = 0.0;
    var centre_dw = vec3(0.0);
    var centre_df = vec3(0.0);
    for (var i = 0; i < 5; i++) {
        let at = here + offsets[i] * texel_size;
        let dw = textureSampleLevel(world_a, linear_sampler, at, 0.0).rgb
            - textureSampleLevel(world_b, linear_sampler, at, 0.0).rgb;
        let df = textureSampleLevel(frame_a, linear_sampler, at, 0.0).rgb
            - textureSampleLevel(frame_b, linear_sampler, at, 0.0).rgb;
        num += dot(df, dw);
        den += dot(dw, dw);
        if i == 0 {
            centre_dw = dw;
            centre_df = df;
        }
    }
    if den <= 0.004 {
        return 0.0;
    }
    let k = clamp(num / den, 0.0, 1.0);
    let rest = centre_df - k * centre_dw;
    return select(0.0, k, dot(rest, rest) < 0.02);
}

@fragment fn fs_interp(in: InterpOut) -> @location(0) vec4<f32> {
    let here = in.uv;
    let world = made_world(here);
    let fb = textureSampleLevel(frame_b, linear_sampler, here, 0.0).rgb;
    let wb = textureSampleLevel(world_b, linear_sampler, here, 0.0).rgb;
    let cover = fb - wb;
    var k = 1.0;
    if max(abs(cover.r), max(abs(cover.g), abs(cover.b))) > u.params.y {
        k = show_through(here);
    }
    var colour = fb + k * (world - wb);
    if u.params.z > 0.5 {
        colour = mix(colour, vec3(1.0, 0.0, 1.0), 0.15);
    }
    return vec4(clamp(colour, vec3(0.0), vec3(1.0)), 1.0);
}
