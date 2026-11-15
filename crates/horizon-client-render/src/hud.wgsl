struct Hud {
    size: vec4<f32>,
    // Blend::EndPortal: atlas rects (u0, v0, width, height) of end_sky.png and
    // end_portal.png, then vanilla's GameTime in x.
    end_portal: array<vec4<f32>, 3>,
}
@group(0) @binding(0) var<uniform> hud: Hud;
@group(0) @binding(1) var atlas: texture_2d<f32>;
@group(0) @binding(2) var atlas_sampler: sampler;
// enchanted_glint_item on its own repeating texture (vanilla's GlintSampler).
@group(0) @binding(3) var glint_tex: texture_2d<f32>;
@group(0) @binding(4) var glint_sampler: sampler;
struct Out {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    // projection_from_position(gl_Position): the End portal's texProj0.
    @location(2) proj: vec4<f32>,
    @location(3) glint_uv: vec2<f32>,
}
@vertex fn vs_main(@location(0) pos: vec2<f32>, @location(1) uv: vec2<f32>, @location(2) color: vec4<f32>, @location(3) depth: f32, @location(4) glint_uv: vec2<f32>) -> Out {
    let pixels = pos * hud.size.z;
    var out: Out;
    out.clip = vec4(pixels.x / hud.size.x * 2.0 - 1.0, 1.0 - pixels.y / hud.size.y * 2.0, depth, 1.0);
    out.uv = uv;
    out.color = color;
    out.proj = vec4(out.clip.xy * 0.5 + 0.5, out.clip.zw);
    out.glint_uv = glint_uv;
    return out;
}
@fragment fn fs_main(in: Out) -> @location(0) vec4<f32> {
    // u >= 2: the GLINT_TEXTURING sample over the item, masked by the item's
    // texture alpha (item.fsh's ALPHA_CUTOUT and GLINT path).
    if in.uv.x >= 2.0 {
        let mask = textureSample(atlas, atlas_sampler, in.uv - vec2(2.0, 0.0)).a;
        if mask < 0.1 { discard; }
        let glint = hud.size.w * textureSample(glint_tex, glint_sampler, in.glint_uv);
        return vec4(glint.rgb * glint.rgb, mask);
    }
    var c = textureSample(atlas, atlas_sampler, in.uv) * in.color;
    if c.a < 0.004 { discard; }

    // Horizon Jade pictograms are original PNG assets in the shared atlas.
    // glint_uv.x in [-5, -3] is reserved *only* for Jade UI icon vertices;
    // vanilla world/glint/text quads and Liquid Glass keep their previous path.
    // Compute a true per-fragment emboss normal from the PNG's alpha field,
    // not from manually scripted outlines or pre-baked shine in the image.
    if in.glint_uv.x < -2.0 {
        let texel = 1. / vec2<f32>(textureDimensions(atlas));
        let l = textureSampleLevel(atlas, atlas_sampler,
            in.uv - vec2(texel.x, 0.), 0.).a;
        let r = textureSampleLevel(atlas, atlas_sampler,
            in.uv + vec2(texel.x, 0.), 0.).a;
        let u = textureSampleLevel(atlas, atlas_sampler,
            in.uv - vec2(0., texel.y), 0.).a;
        let d = textureSampleLevel(atlas, atlas_sampler,
            in.uv + vec2(0., texel.y), 0.).a;
        let bevel = normalize(vec3((l - r) * 1.45, (u - d) * 1.45, 0.76));
        let cursor_light = vec2(clamp(in.glint_uv.x + 4., -1., 1.),
                                clamp(in.glint_uv.y, -1., 1.));
        let light_dir = normalize(vec3(
            -0.48 + 0.65 * cursor_light.x,
            -0.52 + 0.65 * cursor_light.y,
            1.25
        ));
        let half_v = normalize(light_dir + vec3(0., 0., 1.));
        let diffuse = 0.79 + 0.19 * max(dot(bevel, light_dir), 0.);
        let spec = pow(max(dot(bevel, half_v), 0.), 36.) * 0.08;
        let edge = clamp(1. - bevel.z, 0., 1.) * 0.085;
        let jade = vec3(0.38, 0.77, 0.46);
        c = vec4(clamp(c.rgb * diffuse + (vec3(spec) + jade * edge) * c.a,
            vec3(0.), vec3(1.)), c.a);
    }
    return c;
}
@fragment fn fs_crosshair(in: Out) -> @location(0) vec4<f32> {
    let c = textureSample(atlas, atlas_sampler, in.uv) * in.color;
    if c.a == 0.0 { discard; }
    return c;
}
// rendertype_end_portal.fsh (entity.wgsl has the same for portal blocks).
const PORTAL_COLORS = array<vec3<f32>, 16>(
    vec3(0.022087, 0.098399, 0.110818),
    vec3(0.011892, 0.095924, 0.089485),
    vec3(0.027636, 0.101689, 0.100326),
    vec3(0.046564, 0.109883, 0.114838),
    vec3(0.064901, 0.117696, 0.097189),
    vec3(0.063761, 0.086895, 0.123646),
    vec3(0.084817, 0.111994, 0.166380),
    vec3(0.097489, 0.154120, 0.091064),
    vec3(0.106152, 0.131144, 0.195191),
    vec3(0.097721, 0.110188, 0.187229),
    vec3(0.133516, 0.138278, 0.148582),
    vec3(0.070006, 0.243332, 0.235792),
    vec3(0.196766, 0.142899, 0.214696),
    vec3(0.047281, 0.315338, 0.321970),
    vec3(0.204675, 0.390010, 0.302066),
    vec3(0.080955, 0.314821, 0.661491),
);
fn portal_sample(rect: vec4<f32>, p: vec4<f32>) -> vec3<f32> {
    let st = fract(p.xy / p.w);
    return textureSampleLevel(atlas, atlas_sampler, rect.xy + st * rect.zw, 0.0).rgb;
}
fn portal_layer(p: vec4<f32>, layer: f32) -> vec4<f32> {
    let a = radians((layer * layer * 4321.0 + layer * 9.0) * 2.0);
    let c = cos(a);
    let s = sin(a);
    let k = (4.5 - layer / 4.0) * 2.0;
    var v = vec4(k * (c * p.x - s * p.y), k * (s * p.x + c * p.y), p.z, p.w);
    v = vec4(v.x + 17.0 / layer * v.w, v.y + (2.0 + layer / 1.5) * (hud.end_portal[2].x * 1.5) * v.w, v.z, v.w);
    return vec4(0.5 * v.x + 0.25 * v.w, 0.5 * v.y + 0.25 * v.w, v.z, v.w);
}
// RenderPipelines.END_PORTAL (WinScreen's poem background): end_sky and 15
// layers of end_portal.png.
@fragment fn fs_end_portal(in: Out) -> @location(0) vec4<f32> {
    var c = portal_sample(hud.end_portal[0], in.proj) * PORTAL_COLORS[0];
    for (var i = 0; i < 15; i++) {
        c += portal_sample(hud.end_portal[1], portal_layer(in.proj, f32(i + 1))) * PORTAL_COLORS[i];
    }
    return vec4(c, 1.0);
}
