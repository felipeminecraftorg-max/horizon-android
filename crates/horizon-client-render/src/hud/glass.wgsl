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
    if radius < 0.45 { return sample_scene(p); }
    let r = radius * 0.70710678;
    if radius < 1.8 {
        return sample_scene(p) * 0.40 + (sample_scene(p + vec2(r,0.))
            + sample_scene(p - vec2(r,0.)) + sample_scene(p + vec2(0.,r))
            + sample_scene(p - vec2(0.,r))) * 0.15;
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
    if any(pos < lens.clip.xy) || any(pos >= lens.clip.xy + lens.clip.zw) {
        discard;
    }
    let half_size = lens.rect.zw * 0.5;
    let p = pos - (lens.rect.xy + half_size);
    let radius = max(min(lens.optics.x, min(half_size.x, half_size.y)), 0.5);
    // Rounded-box SDF: the same geometry defines both the shape and the optics.
    let q = abs(p) - half_size + vec2(radius);
    let outside = max(q, vec2(0.));
    let signed_d = length(outside) + min(max(q.x, q.y), 0.) - radius;
    let aa = max(fwidth(signed_d), 0.7);
    let coverage = clamp(0.5 - signed_d / aa, 0., 1.);
    let dp = max(lens.optics.w, 0.5);
    var outward = select(vec2(0., sign(p.y)), vec2(sign(p.x), 0.), q.x > q.y);
    if length(outside) > 0.001 {
        outward = normalize(outside) * sign(p);
    }
    // Two offset area lights, with a restrained pointer-following component.
    let key = normalize(vec2(-0.72, -0.69) + lens.frame.zw * 0.22);
    let fill = normalize(vec2(0.78, 0.63));
    let key_amount = pow(max(dot(outward, key), 0.), 7.);
    let fill_amount = pow(max(dot(outward, fill), 0.), 12.);
    let beyond = max(signed_d, 0.);
    let shadow = 0.09 * exp(-beyond / (2.25 * dp))
        * (1. - smoothstep(6. * dp, 9. * dp, beyond));
    // Perceptual bloom/bleed is LOCAL to the lens halo; no fullscreen bloom pass.
    let halo = exp(-pow(beyond / max(4.0 * dp, 0.1), 2.))
        * (key_amount * 0.16 + fill_amount * 0.045);
    if coverage <= 0. {
        return vec4(vec3(0.57, 0.73, 0.88), halo + shadow * 0.50);
    }

    let depth = max(-signed_d, 0.);
    // Rounded cross-section. Circular lenses curve across most of their radius;
    // a wide panel has a flat central plateau for readable foreground UI.
    let roll = max(min(radius * 0.95, 29.0 * dp), 0.75 * dp);
    let t = clamp(depth / roll, 0., 1.);
    let slope = pow(1. - t, 0.68);
    let front = sqrt(max(1. - slope * slope, 0.0004));
    let normal = normalize(vec3(outward * slope, front));

    // Screen-space thin-lens refraction: Snell + the finite slab depth.
    // The displacement is zero in the central plateau but pronounced at edges.
    let incident = vec3(0., 0., -1.);
    let transmitted = refract(incident, normal, 1. / 1.48);
    let slab = (roll * front * 1.75 + 3.0 * dp) * (lens.optics.z / (14.0 * dp));
    var displacement = transmitted.xy / max(-transmitted.z, 0.12) * slab * 1.44;
    displacement -= p / max(half_size, vec2(1.)) * lens.optics.z * 0.028;
    // Never sample beyond the snapshot's sampling halo or across a whole button.
    let max_dist = min(lens.optics.z * 2.1, min(half_size.x, half_size.y) * 0.74);
    displacement *= min(1., max_dist / max(length(displacement), 0.001));
    let refracted = pos + displacement;

    // Adaptive blur: crystal-clear centre, subtle scatter in thicker glass.
    let blur = lens.optics.y * (0.08 + 0.83 * slope * slope);
    var rgb = blur_scene(refracted, blur);
    if lens.optics.z > 0. && slope > 0.04 {
        let chroma = outward * pow(slope, 1.7)
            * min(lens.optics.z * 0.038, 0.77 * dp);
        // Chromatic aberration on the bezel only: foreground remains sharp.
        rgb.r = mix(rgb.r, sample_scene(refracted + chroma).r, 0.60 * slope);
        rgb.b = mix(rgb.b, sample_scene(refracted - chroma).b, 0.60 * slope);
    }

    // Fresnel-Schlick of clear dielectric (F0 around four percent).
    let ior = 1.48;
    let f0 = pow((ior - 1.) / (ior + 1.), 2.);
    let fresnel = f0 + (1. - f0) * pow(1. - normal.z, 5.);
    let luma = dot(rgb, vec3(0.2126, 0.7152, 0.0722));
    rgb = mix(rgb, lens.tint.rgb,
        lens.tint.a * (0.65 + 0.20 * smoothstep(0.2, 0.85, luma)));

    // Reflection takes environment colour from the scene, not a flat white rim.
    let ambient = sample_scene(pos + outward * (6.0 * dp));
    let bounced = sample_scene(pos - outward * min(roll * 0.8, 18.0 * dp));
    let environment = mix(vec3(0.44, 0.55, 0.66), ambient, 0.44);
    rgb = mix(rgb, environment, clamp(fresnel * 0.76, 0., 0.69));

    // Specular lobe from two directional sources, spread into a soft bloom.
    let light = normalize(vec3(key, 0.92));
    let half_vec = normalize(light + vec3(0., 0., 1.));
    let hdot = max(dot(normal, half_vec), 0.);
    let lobe = pow(hdot, 88.) * pow(slope, 0.65);
    let rim = exp(-depth / max(1.1 * dp, 0.5));
    let key_reflection = key_amount * (0.39 * rim + 0.13 * slope);
    let fill_reflection = fill_amount * (0.20 * rim + 0.06 * slope);
    let speck = lobe * 0.38;
    // Inner caustic / directional colour bleed where the refracted ray exits.
    let caustic_band = exp(-pow((depth - roll * 0.61) /
        max(roll * 0.16, dp), 2.));
    let caustic = caustic_band * pow(max(dot(-outward, key), 0.), 3.)
        * slope * 0.20;
    let spill = (ambient * 0.37 + bounced * 0.63)
        * (0.06 * slope + 0.075 * caustic_band) * (0.5 + 0.5 * luma);
    rgb = clamp(rgb + spill + caustic * vec3(0.60, 0.77, 0.94), vec3(0.), vec3(1.));
    let shine = clamp(key_reflection + fill_reflection + speck
        + 0.12 * rim * pow(max(dot(outward, key), 0.), 20.), 0., 0.82);
    rgb = mix(rgb, vec3(0.96, 0.985, 1.), shine);
    return vec4(rgb, coverage + shadow * (1. - coverage));
}
@fragment fn fs_blit(in: Out) -> @location(0) vec4<f32> {
    return textureSampleLevel(scene,scene_sampler,in.position.xy / lens.frame.xy,0.);
}
