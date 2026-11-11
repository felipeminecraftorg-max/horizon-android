// Original rounded-lens optics. Samples the already-composited launcher scene.
struct Lens {
    frame: vec4<f32>,
    rect: vec4<f32>,
    clip: vec4<f32>,
    tint: vec4<f32>,
    optics: vec4<f32>, // corner radius, blur, refraction, physical dp
}
@group(0) @binding(0) var<uniform> lens: Lens;
@group(0) @binding(1) var scene: texture_2d<f32>;
@group(0) @binding(2) var scene_sampler: sampler;
struct Out { @builtin(position) position: vec4<f32> }
@vertex fn vs_lens(@builtin(vertex_index) i: u32) -> Out {
    let corners = array<vec2<f32>,6>(vec2(0.,0.),vec2(1.,0.),vec2(1.,1.),vec2(0.,0.),vec2(1.,1.),vec2(0.,1.));
    let p = lens.rect.xy + corners[i] * lens.rect.zw;
    return Out(vec4(p.x / lens.frame.x * 2. - 1., 1. - p.y / lens.frame.y * 2., 0., 1.));
}
@vertex fn vs_blit(@builtin(vertex_index) i: u32) -> Out {
    let p = array<vec2<f32>,3>(vec2(-1.,-1.),vec2(3.,-1.),vec2(-1.,3.));
    return Out(vec4(p[i],0.,1.));
}
fn sample_scene(p: vec2<f32>) -> vec3<f32> {
    let uv = clamp(p / lens.frame.xy, vec2(0.5) / lens.frame.xy, vec2(1.) - vec2(0.5) / lens.frame.xy);
    return textureSampleLevel(scene, scene_sampler, uv, 0.).rgb;
}
fn blur_scene(p: vec2<f32>, radius: f32) -> vec3<f32> {
    if radius < 0.1 { return sample_scene(p); }
    let r = radius * 0.70710678;
    var c = sample_scene(p) * 0.20;
    c += (sample_scene(p+vec2(r,0.)) + sample_scene(p-vec2(r,0.))
        + sample_scene(p+vec2(0.,r)) + sample_scene(p-vec2(0.,r))) * 0.12;
    c += (sample_scene(p+vec2(r,r)) + sample_scene(p-vec2(r,r))
        + sample_scene(p+vec2(r,-r)) + sample_scene(p+vec2(-r,r))) * 0.08;
    return c;
}
@fragment fn fs_lens(in: Out) -> @location(0) vec4<f32> {
    let pos = in.position.xy;
    let half_size = lens.rect.zw * 0.5;
    let p = pos - (lens.rect.xy + half_size);
    let radius = min(lens.optics.x, min(half_size.x, half_size.y));
    let q = abs(p) - half_size + vec2(radius);
    let outside = max(q, vec2(0.));
    let distance = length(outside) + min(max(q.x,q.y),0.) - radius;
    let aa = max(fwidth(distance),0.75);
    let coverage = clamp(0.5 - distance / aa,0.,1.);
    if coverage <= 0. || any(pos < lens.clip.xy) || any(pos >= lens.clip.xy + lens.clip.zw) { discard; }
    var n = select(vec2(0.,sign(p.y)), vec2(sign(p.x),0.), q.x > q.y);
    if length(outside) > 0.001 { n = normalize(outside) * sign(p); }
    let bevel = max(min(radius * 0.75,18. * lens.optics.w),1.);
    let depth = max(-distance,0.);
    let edge = exp(-depth / bevel);
    // Convex surface normal and Snell refraction, fading smoothly into clear glass.
    let normal = normalize(vec3(n * edge * 1.6,1.));
    let ray = refract(vec3(0.,0.,-1.),normal,1. / 1.45);
    let offset = ray.xy * lens.optics.z;
    let uv_pos = pos + offset;
    var c = blur_scene(uv_pos, lens.optics.y * (0.4 + 0.6 * (1. - edge)));
    // Subtle wavelength separation at the curved perimeter, never on foreground text.
    let fringe = n * edge * min(lens.optics.z * 0.055, lens.optics.w * 0.7);
    c.r = mix(c.r,sample_scene(uv_pos + fringe).r,edge * 0.32);
    c.b = mix(c.b,sample_scene(uv_pos - fringe).b,edge * 0.32);
    c = mix(c,lens.tint.rgb,lens.tint.a);
    let lighting = dot(n,normalize(vec2(-0.65,-0.76)));
    let rim = exp(-depth / max(0.75,lens.optics.w * 0.55));
    let fresnel = pow(1. - normal.z,2.);
    let shine = rim * (0.13 + 0.55 * pow(abs(lighting),5.)) + fresnel * max(lighting,0.) * 0.18;
    c = mix(c,vec3(0.94,0.97,1.),clamp(shine,0.,0.66));
    return vec4(c,coverage);
}
@fragment fn fs_blit(in: Out) -> @location(0) vec4<f32> {
    return textureSampleLevel(scene,scene_sampler,in.position.xy / lens.frame.xy,0.);
}
