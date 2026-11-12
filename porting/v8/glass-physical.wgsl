// V8-compatible physically rolled lens. Native V8 shader layout retained (six vec4s).
// Staging only: port to V8 source and validate on the ARM64 device before release.
// Original rounded glass optics. The background is live; foreground UI is drawn later.
struct Lens {
    frame: vec4<f32>, // physical viewport size, local interaction direction
    rect: vec4<f32>,
    clip: vec4<f32>,
    tint: vec4<f32>,
    optics: vec4<f32>, // corner radius, blur, refraction, physical dp
    quality: vec4<f32>, // V8: blur taps, group opacity, lens depth, dispersion
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
    let distance = length(outside) + min(max(q.x, q.y), 0.) - radius;
    let aa = max(fwidth(distance), 0.75);
    let coverage = clamp(0.5 - distance / aa, 0., 1.);
    if any(pos < lens.clip.xy) || any(pos >= lens.clip.xy + lens.clip.zw) { discard; }

    // A nearby contact shadow is the only opacity outside the dielectric body.
    let dp = max(lens.optics.w, 0.5);
    let shadow_dist = max(distance, 0.);
    let shadow = 0.10 * exp(-shadow_dist / (1.7 * dp))
        * (1. - smoothstep(5. * dp, 8. * dp, shadow_dist));
    if coverage <= 0. { return vec4(0., 0., 0., shadow * lens.quality.y); }

    // An analytic rounded-rectangle SDF provides a curved physical lens edge,
    // with a flat front face. Small pills show the roll; large panels stay clear.
    var edge_normal = select(vec2(0., sign(p.y)), vec2(sign(p.x), 0.), q.x > q.y);
    if length(outside) > 0.001 {
        edge_normal = normalize(outside) * sign(p);
    }
    let bevel = max(min(radius * 0.96, 25. * dp), dp * 0.65);
    let depth = max(-distance, 0.);
    let edge_fraction = clamp(depth / bevel, 0., 1.);
    let wall = pow(1. - edge_fraction, 0.88);
    let front = sqrt(max(1. - wall * wall, 0.0001));
    let normal = normalize(vec3(edge_normal * wall, front));

    // Snell refraction bends the scene only where the material actually curves.
    let ray = refract(vec3(0., 0., -1.), normal, 1. / 1.47);
    let thickness = (bevel * front * 1.65 + 1.8 * dp) * lens.quality.z;
    var offset = ray.xy / max(-ray.z, 0.15) * thickness
        * (lens.optics.z / max(14. * dp, 1.));
    offset -= p / max(half_size, vec2(1.)) * lens.optics.z * 0.045;
    let displacement_limit = min(lens.optics.z * 1.75, min(half_size.x, half_size.y) * 0.48);
    offset *= min(1., displacement_limit / max(length(offset), 0.001));
    let uv_pos = pos + offset;

    // Clear centres, soft blur at the rolled wall. Never blur the foreground UI.
    var c = blur_scene(uv_pos, lens.optics.y * (0.15 + 1.05 * wall * wall));
    if lens.quality.x >= 9. && lens.quality.w > 0.5 && lens.optics.z > 0. && wall > 0.01 {
        let dispersion = edge_normal * wall * min(lens.optics.z * 0.018, dp * 0.32);
        c.r = mix(c.r, sample_scene(uv_pos + dispersion).r, wall * 0.16);
        c.b = mix(c.b, sample_scene(uv_pos - dispersion).b, wall * 0.16);
    }

    // Dielectric Fresnel and microfacet reflections: no painted glass bands.
    let ior = 1.47;
    let f0 = pow((ior - 1.) / (ior + 1.), 2.);
    let fresnel = f0 + (1. - f0) * pow(1. - normal.z, 5.);
    let light_xy = normalize(vec2(-0.64, -0.77) + lens.frame.zw * 0.20);
    let light = normalize(vec3(light_xy, 1.15));
    let halfway = normalize(light + vec3(0., 0., 1.));
    let nv = max(normal.z, 0.001);
    let nl = max(dot(normal, light), 0.001);
    let nh = max(dot(normal, halfway), 0.);
    let roughness = 0.23;
    let a2 = pow(roughness, 4.);
    let denominator = nh * nh * (a2 - 1.) + 1.;
    let distribution = a2 / max(3.14159265 * denominator * denominator, 0.00001);
    let k = roughness * roughness * 0.5;
    let visibility = (nv / (nv * (1. - k) + k)) * (nl / (nl * (1. - k) + k));
    let specular = distribution * visibility * f0 / max(4. * nv, 0.004);

    // Low-colour, translucent material. Backdrop owns the central pixels.
    let luma = dot(c, vec3(0.2126, 0.7152, 0.0722));
    let tint_alpha = lens.tint.a * (0.82 + 0.18 * smoothstep(0.25, 0.9, luma));
    c = mix(c, lens.tint.rgb, tint_alpha);
    let reflected = reflect(vec3(0., 0., -1.), normal);
    let sky = smoothstep(-0.65, 0.8, -reflected.y);
    let environment = mix(vec3(0.10, 0.13, 0.18), vec3(0.88, 0.94, 1.), sky);
    c = mix(c, environment, fresnel * 0.69);

    // Contact rim + a small internal caustic make a pill read as a lens object.
    let rim = exp(-depth / max(0.6, dp * 0.62));
    let directional = pow(max(dot(edge_normal, light_xy), 0.), 9.);
    let caustic = exp(-pow((depth - bevel * 0.58) / max(bevel * 0.15, dp), 2.))
        * pow(max(-dot(edge_normal, light_xy), 0.), 3.) * wall * 0.065;
    let shine = clamp(rim * (0.055 + directional * 0.53)
        + (1. - exp(-specular * 0.30)) * smoothstep(0., 0.07, wall)
        + caustic, 0., 0.68);
    c = mix(c, vec3(0.97, 0.988, 1.), shine);
    return vec4(c, (coverage + shadow * (1. - coverage)) * lens.quality.y);
}
@fragment fn fs_blit(in: Out) -> @location(0) vec4<f32> {
    return textureSampleLevel(scene,scene_sampler,in.position.xy / lens.frame.xy,0.);
}
