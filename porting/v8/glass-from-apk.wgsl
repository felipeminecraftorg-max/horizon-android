// Recovered from the user-supplied Horizon_V8_Teste_ARM64.apk native library.
// Baseline V8 shader (unchanged). Reference only: DO NOT replace the source
// tree's current shader without the V8 renderer's 96-byte Uniform layout.
// Original rounded glass optics. The background is live; foreground UI is drawn later.
struct Lens {
    frame: vec4<f32>, // physical viewport size, local interaction direction
    rect: vec4<f32>,
    clip: vec4<f32>,
    tint: vec4<f32>,
    optics: vec4<f32>, // corner radius, blur, refraction, physical dp
    quality: vec4<f32>, // blur taps, group opacity, lens depth, dispersion enabled
}
@group(0) @binding(0) var<uniform> lens: Lens;
@group(0) @binding(1) var scene: texture_2d<f32>;
@group(0) @binding(2) var scene_sampler: sampler;
struct Out { @builtin(position) position: vec4<f32> }
@vertex fn vs_lens(@builtin(vertex_index) i: u32) -> Out {
    let corners = array<vec2<f32>,6>(vec2(0.,0.),vec2(1.,0.),vec2(1.,1.),vec2(0.,0.),vec2(1.,1.),vec2(0.,1.));
    let margin = 8. * lens.optics.w;
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
    if radius < 0.1 || lens.quality.x < 2. { return sample_scene(p); }
    let r = radius * 0.70710678;
    if lens.quality.x < 6. {
        return sample_scene(p) * 0.4 + (sample_scene(p+vec2(r,0.)) + sample_scene(p-vec2(r,0.))
            + sample_scene(p+vec2(0.,r)) + sample_scene(p-vec2(0.,r))) * 0.15;
    }
    var c = sample_scene(p) * 0.20;
    c += (sample_scene(p+vec2(r,0.)) + sample_scene(p-vec2(r,0.))
        + sample_scene(p+vec2(0.,r)) + sample_scene(p-vec2(0.,r))) * 0.12;
    c += (sample_scene(p+vec2(r,r)) + sample_scene(p-vec2(r,r))
        + sample_scene(p+vec2(r,-r)) + sample_scene(p-vec2(-r,r))) * 0.08;
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
    // A short shadow separates the optical body without tinting the whole face.
    let shadow = 0.09 * exp(-max(distance,0.) / max(lens.optics.w * 1.8,0.5))
        * (1. - smoothstep(5. * lens.optics.w,8. * lens.optics.w,max(distance,0.)));
    if coverage <= 0. { return vec4(0.,0.,0.,shadow * lens.quality.y); }
    var n = select(vec2(0.,sign(p.y)), vec2(sign(p.x),0.), q.x > q.y);
    if length(outside) > 0.001 { n = normalize(outside) * sign(p); }
    let dp = lens.optics.w;
    // Fit the edge roll within the capsule: zero slope at its medial axis.
    let bevel = max(min(radius,24. * dp),0.5 * dp);
    let depth = max(-distance,0.);
    let edge_t = clamp(depth / bevel,0.,1.);
    let wall = 1. - edge_t;
    let arc = sqrt(max(1. - wall * wall,0.0001));
    let normal = normalize(vec3(n * wall,arc));
    let ray = refract(vec3(0.,0.,-1.),normal,1. / 1.46);
    let thickness = (bevel * arc * 1.5 + 2. * dp) * lens.quality.z;
    var offset = ray.xy / max(-ray.z,0.15) * thickness * (lens.optics.z / max(14. * dp,1.));
    // Almost-flat centre: small magnification, while the rolled edge bends the scene.
    offset -= p / max(half_size,vec2(1.)) * lens.optics.z * 0.035;
    let limit = min(lens.optics.z * 1.6,min(half_size.x,half_size.y) * 0.45);
    offset *= min(1., limit / max(length(offset),0.001));
    let uv_pos = pos + offset;
    var c = blur_scene(uv_pos,lens.optics.y * (0.20 + 1.10 * wall * wall));
    if lens.quality.x >= 9. && lens.quality.w > 0.5 && lens.optics.z > 0. && wall > 0.01 {
        let fringe = n * wall * min(lens.optics.z * 0.025, dp * 0.40);
        c.r = mix(c.r,sample_scene(uv_pos + fringe).r,wall * 0.20);
        c.b = mix(c.b,sample_scene(uv_pos - fringe).b,wall * 0.20);
    }
    // Original dielectric Fresnel/GGX light model. No painted inner bands
    // or reflected copies of already composited controls.
    let f0 = pow((1.46 - 1.) / (1.46 + 1.),2.);
    let fresnel = f0 + (1. - f0) * pow(1. - normal.z,5.);
    let luminance = dot(c,vec3(0.2126,0.7152,0.0722));
    let tint_alpha = lens.tint.a * (0.85 + 0.15 * smoothstep(0.3,0.9,luminance));
    c = mix(c,lens.tint.rgb,tint_alpha);
    let light_xy = normalize(vec2(-0.65,-0.76) + lens.frame.zw * 0.20);
    let light = normalize(vec3(light_xy,1.2));
    let half_vector = normalize(light + vec3(0.,0.,1.));
    let nv = max(normal.z,0.001);
    let nl = max(dot(normal,light),0.001);
    let nh = max(dot(normal,half_vector),0.);
    let roughness = 0.18;
    let a2 = pow(roughness,4.);
    let denominator = nh * nh * (a2 - 1.) + 1.;
    let distribution = a2 / max(3.14159265 * denominator * denominator,0.00001);
    let k = roughness * roughness * 0.5;
    let visibility = (nv / (nv * (1. - k) + k)) * (nl / (nl * (1. - k) + k));
    let specular = distribution * visibility * f0 / max(4. * nv,0.004);
    let reflected = reflect(vec3(0.,0.,-1.),normal);
    let sky = smoothstep(-0.65,0.8,-reflected.y);
    let environment = mix(vec3(0.12,0.16,0.20),vec3(0.84,0.91,1.),sky);
    c = mix(c,environment,fresnel * 0.65);
    c = mix(c,vec3(0.97,0.985,1.),(1. - exp(-specular * 0.32)) * smoothstep(0.,0.08,wall));
    return vec4(c,(coverage + shadow * (1. - coverage)) * lens.quality.y);
}
@fragment fn fs_blit(in: Out) -> @location(0) vec4<f32> {
    return textureSampleLevel(scene,scene_sampler,in.position.xy / lens.frame.xy,0.);
}
