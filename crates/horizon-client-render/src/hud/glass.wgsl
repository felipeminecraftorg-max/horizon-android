// Original rounded glass optics. The background is live; foreground UI is drawn later.
struct Lens {
    frame: vec4<f32>, // physical viewport size, local interaction direction
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
    let margin = 4. * lens.optics.w;
    let p = lens.rect.xy - vec2(margin) + corners[i] * (lens.rect.zw + vec2(2. * margin));
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
    if any(pos < lens.clip.xy) || any(pos >= lens.clip.xy + lens.clip.zw) { discard; }
    // A short contact shadow anchors the clear rim without covering its centre.
    let shadow = 0.13 * exp(-max(distance,0.) / max(lens.optics.w * 1.6,0.5));
    if coverage <= 0. { return vec4(0.,0.,0.,shadow); }
    var n = select(vec2(0.,sign(p.y)), vec2(sign(p.x),0.), q.x > q.y);
    if length(outside) > 0.001 { n = normalize(outside) * sign(p); }
    let bevel = max(min(radius * 0.90,24. * lens.optics.w),1.);
    let depth = max(-distance,0.);
    // Circular cross-section: a steep outside wall rolls into a clear, nearly flat top.
    let wall = 1. - clamp(depth / bevel,0.,1.);
    let height = sqrt(max(1. - wall * wall,0.015));
    let normal = normalize(vec3(n * wall, height));
    let ray = refract(vec3(0.,0.,-1.),normal,1. / 1.45);
    var offset = ray.xy / max(-ray.z,0.1) * lens.optics.z * height * 2.4;
    // Small face magnification joins the stronger curved-edge displacement.
    offset -= p / max(half_size,vec2(1.)) * lens.optics.z * 0.18;
    offset *= min(1., lens.optics.z * 1.5 / max(length(offset),0.001));
    let uv_pos = pos + offset;
    // The face stays clearer than the rolled edge. Blur remains a user setting.
    var c = blur_scene(uv_pos, lens.optics.y * (0.45 + 0.55 * wall));
    if lens.optics.z > 0. && wall > 0.01 {
        let fringe = n * wall * min(lens.optics.z * 0.025, lens.optics.w * 0.45);
        c.r = mix(c.r,sample_scene(uv_pos + fringe).r,wall * 0.22);
        c.b = mix(c.b,sample_scene(uv_pos - fringe).b,wall * 0.22);
    }
    c = mix(c,lens.tint.rgb,lens.tint.a);
    let light_xy = normalize(vec2(-0.65,-0.76) + lens.frame.zw * 0.20);
    let lighting = dot(n,light_xy);
    let rim = exp(-depth / max(0.5,lens.optics.w * 0.38));
    let light = normalize(vec3(light_xy,0.85));
    let half_vector = normalize(light + vec3(0.,0.,1.));
    let specular = pow(max(dot(normal,half_vector),0.),36.) * wall;
    let fresnel = pow(1. - normal.z,5.);
    let caustic = exp(-pow((depth - bevel * 0.48) / max(bevel * 0.18,1.),2.))
        * pow(max(-lighting,0.),2.);
    c *= 1. - rim * (0.08 + max(-lighting,0.) * 0.15);
    let shine = rim * (0.10 + 0.72 * pow(max(lighting,0.),8.)
        + 0.18 * pow(max(-lighting,0.),8.))
        + specular * 0.32 + fresnel * 0.055 + caustic * 0.055;
    c = mix(c,vec3(0.97,0.985,1.),clamp(shine,0.,0.84));
    return vec4(c,coverage + shadow * (1. - coverage));
}
@fragment fn fs_blit(in: Out) -> @location(0) vec4<f32> {
    return textureSampleLevel(scene,scene_sampler,in.position.xy / lens.frame.xy,0.);
}
