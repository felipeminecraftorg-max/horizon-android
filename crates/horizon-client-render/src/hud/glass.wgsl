// Original rounded glass optics. The background is live; foreground UI is drawn later.
struct Lens {
    frame: vec4<f32>, // physical viewport size, local interaction direction
    rect: vec4<f32>,
    clip: vec4<f32>,
    tint: vec4<f32>,
    optics: vec4<f32>, // corner radius, blur, refraction, physical dp
    profile: array<vec4<f32>, 6>, // exact Glass Lab V7 style profile
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
// Port of Glass Lab V7's per-fragment dielectric/chrome material.
// The live backdrop is composited in ordered passes, not a WebGL screenshot.
// profile indices are documented in launcher_preset_pack.rs.
fn ggx_distribution(nh: f32, rough: f32) -> f32 {
    let a = rough * rough;
    let a2 = a * a;
    let den = nh * nh * (a2 - 1.) + 1.;
    return a2 / max(3.14159265 * den * den, 0.00008);
}
// Old School tiles should not mirror labels from adjacent controls.
// Only the console chrome is confined; Liquid Glass keeps its free refraction.
fn confined_sample_pos(point: vec2<f32>, oldschool: bool) -> vec2<f32> {
    let pad = max(2.5 * lens.optics.w, 2.0);
    let lo = lens.rect.xy + vec2(pad);
    let hi = max(lo, lens.rect.xy + lens.rect.zw - vec2(pad));
    return select(point, clamp(point, lo, hi), oldschool);
}
fn normal_for(q: vec2<f32>, outside: vec2<f32>, p: vec2<f32>) -> vec2<f32> {
    if length(outside) > 0.001 { return normalize(outside) * sign(p); }
    return select(vec2(0., sign(p.y)), vec2(sign(p.x), 0.), q.x > q.y);
}
@fragment fn fs_lens(in: Out) -> @location(0) vec4<f32> {
    let pos = in.position.xy;
    if any(pos < lens.clip.xy) || any(pos >= lens.clip.xy + lens.clip.zw) { discard; }
    let half_size = lens.rect.zw * 0.5;
    let p = pos - lens.rect.xy - half_size;
    let radius = max(min(lens.optics.x, min(half_size.x, half_size.y)), 0.5);
    let q = abs(p) - half_size + vec2(radius);
    let outside = max(q, vec2(0.));
    let sd = length(outside) + min(max(q.x, q.y), 0.) - radius;
    let aa = max(fwidth(sd), 0.75);
    let coverage = clamp(0.5 - sd / aa, 0., 1.);
    let outward = normal_for(q, outside, p);
    let dp = max(lens.optics.w, 0.5);
    let mode = lens.profile[0].x;
    let oldschool = mode > 0.5;
    let light_dir = normalize(lens.profile[1].xy + lens.frame.zw * 0.08);
    let key_intensity = pow(max(dot(outward, light_dir), 0.), 7.);
    let secondary = pow(max(dot(outward, -light_dir), 0.), 11.);
    let beyond = max(sd, 0.);
    let shadow = lens.profile[5].z * 0.30 * exp(-beyond / (2.4 * dp))
        * (1. - smoothstep(6. * dp, 10. * dp, beyond));
    let glow = lens.profile[3].z * 0.24 * exp(-pow(beyond / (7. * dp), 2.))
        * key_intensity;
    if coverage <= 0. {
        return vec4(
            select(vec3(0.60, 0.75, 0.88), vec3(0.72, 0.76, 0.83), oldschool),
            shadow * 0.45 + glow
        );
    }
    let depth = max(-sd, 0.);
    let roll = max(min(radius * 0.98, lens.profile[2].y * dp), 0.8 * dp);
    let edge_t = clamp(depth / roll, 0., 1.);
    let wall = pow(1. - edge_t, select(0.71, 0.72, oldschool));
    let front = sqrt(max(1. - wall * wall, 0.0005));
    let polarity = sign(lens.profile[2].z);
    var normal = normalize(vec3(outward * wall * polarity, front));
    let incident = vec3(0., 0., -1.);
    let ray = refract(incident, normal, select(1. / 1.48, 1. / 1.55, oldschool));
    let strength = lens.profile[2].x;
    let thickness = (roll * 0.7 + 4. * dp) * (0.38 + abs(lens.profile[2].z));
    var bend = ray.xy / max(-ray.z, 0.16) * thickness * strength;
    bend -= (p / max(half_size, vec2(1.))) * (1. - wall) * strength * 1.05 * polarity;
    let max_bend = min(min(half_size.x, half_size.y) * 0.66,
        select(11. * dp + strength * 38. * dp, 45. * dp, oldschool));
    bend *= min(1., max_bend / max(length(bend), 0.001));
    let refracted = confined_sample_pos(pos + bend, oldschool);
    let blur_radius = lens.profile[5].x * dp * (0.12 + wall * wall);
    var color = blur_scene(refracted, blur_radius);
    if wall > 0.02 {
        let fringe = outward * (wall * wall * lens.profile[2].w * dp);
        color.r = mix(color.r, sample_scene(confined_sample_pos(refracted + fringe, oldschool)).r, wall * 0.72);
        color.b = mix(color.b, sample_scene(confined_sample_pos(refracted - fringe, oldschool)).b, wall * 0.72);
    }
    let ambient = sample_scene(confined_sample_pos(pos + outward * (7. * dp + roll * 0.14), oldschool));
    let bounced = sample_scene(confined_sample_pos(pos - outward * (6. * dp + roll * 0.25), oldschool));
    let L = normalize(vec3(light_dir, lens.profile[1].z));
    let V = vec3(0., 0., 1.);
    let H = normalize(L + V);
    let nv = max(dot(normal, V), 0.003);
    let nl = max(dot(normal, L), 0.002);
    let nh = max(dot(normal, H), 0.);
    let gloss = lens.profile[3].x;
    let shine = lens.profile[1].w;
    let fresnel_strength = lens.profile[3].y;
    let light_spill = lens.profile[3].w;
    let rim = exp(-depth / max(1.2 * dp, roll * 0.11));
    let inner = exp(-pow((depth - roll * 0.64) / max(dp, roll * 0.17), 2.));
    if oldschool {
        // Horizon Jade is a separate original contemporary console material.
        // Finish 4 is compact satin graphite with physically calculated GGX
        // specular and cursor-driven light; it deliberately avoids the old
        // PS3-era mirrored chrome and blue pearl presets below.
        if lens.profile[0].y >= 3.5 {
            let tangent_grain = (sin(pos.y * 1.71 + sin(pos.x * 0.23) * 0.16)
                + sin(pos.x * 0.84)) * 0.0035;
            let n = normalize(normal + vec3(0., tangent_grain, 0.));
            let nl_jade = max(dot(n, L), 0.0);
            let nv_jade = max(dot(n, V), 0.02);
            let nh_jade = max(dot(n, H), 0.0);
            let vh_jade = max(dot(V, H), 0.0);
            let rough = clamp(0.37 - gloss * 0.045, 0.29, 0.47);
            let alpha = rough * rough;
            let k = (alpha + 1.) * (alpha + 1.) / 8.;
            let g = (nl_jade / (nl_jade * (1. - k) + k))
                * (nv_jade / (nv_jade * (1. - k) + k));
            let d = ggx_distribution(nh_jade, rough);
            let f0 = vec3(0.052, 0.083, 0.061);
            let f = f0 + (vec3(1.) - f0) * pow(1. - vh_jade, 5.);
            let spec = min(vec3(0.16), f * (d * g / max(4. * nv_jade * nl_jade, 0.02)));
            let lighting = 0.86 + 0.13 * nl_jade;
            let jade_base = mix(vec3(0.048, 0.073, 0.055),
                vec3(0.075, 0.118, 0.083), clamp(nl_jade, 0., 1.));
            // Reflection uses only a very small local scene contribution,
            // clamped to this tile so text from neighbours never ghosts.
            let reflection_dir = reflect(incident, n);
            let local_reflect = sample_scene(confined_sample_pos(
                pos + reflection_dir.xy * (4.0 * dp), true));
            let mask = clamp(0.1 + 0.28 * wall, 0., 0.35);
            var matte = mix(jade_base, color, 0.085) * lighting;
            matte += spec * (0.70 + 0.30 * lens.profile[1].w);
            matte += local_reflect * mask * f * 0.18;
            let edge = exp(-depth / max(1.5 * dp, roll * 0.17));
            matte += mix(vec3(0.13, 0.56, 0.27), lens.tint.rgb, 0.55) * edge
                * (0.028 + 0.065 * max(dot(outward, light_dir), 0.));
            matte += vec3(0.13, 0.19, 0.16) * inner * 0.08;
            return vec4(clamp(matte, vec3(0.), vec3(1.)),
                coverage + shadow * (1. - coverage));
        }
        // Legacy chrome finish remains available only to archival presets:
        // probes, two analytic white strip lights and true microfacet GGX.
        let finish = lens.profile[0].y;
        let metal = lens.profile[0].z;
        let chrome = lens.profile[0].w;
        let grain_strength = lens.profile[4].x;
        let edge = lens.profile[4].z;
        let accent = lens.profile[4].w;
        let grain = (sin(pos.y * 0.77 + sin(pos.x * 0.12) * 0.14) * 0.58
            + sin(pos.y * 2.21 + pos.x * 0.013) * 0.20)
            * grain_strength * select(0., 1., finish > 2.5);
        normal = normalize(normal + vec3(0., grain * 0.085, 0.));
        let rough = clamp(mix(0.40, 0.085, gloss) + grain_strength * 0.08, 0.06, 0.48);
        let k = rough * rough * 0.5;
        let nh2 = max(dot(normal, H), 0.);
        let ggx = clamp(ggx_distribution(nh2, rough)
            * (nv / (nv * (1. - k) + k))
            * (nl / (nl * (1. - k) + k)) / max(4. * nv, 0.01), 0., 15.);
        var base = vec3(0.010, 0.012, 0.019);
        var f0 = vec3(0.034);
        var transmission = 0.065;
        if finish > 0.5 && finish < 1.5 {
            base = vec3(0.080, 0.089, 0.108);
            f0 = vec3(0.24, 0.25, 0.29);
            transmission = 0.16;
        } else if finish > 1.5 && finish < 2.5 {
            base = vec3(0.024, 0.037, 0.049);
            f0 = vec3(0.039);
            transmission = 0.72;
        } else if finish > 2.5 {
            base = vec3(0.068, 0.072, 0.081);
            f0 = vec3(0.42, 0.43, 0.46);
            transmission = 0.035;
        }
        let fresnel = f0 + (vec3(1.) - f0) * pow(1. - nv, 5.);
        let reflected = reflect(incident, normal);
        let reflected_scene = sample_scene(confined_sample_pos(pos + reflected.xy * (17. * dp + roll * 0.76), true));
        let env = mix(vec3(0.10, 0.13, 0.18), reflected_scene, 0.70);
        let reflection_amount = clamp(0.25 + 0.56 * chrome + 0.23 * metal, 0., 0.97);
        let strip_a = pow(max(dot(normal, normalize(vec3(-0.75, 0.34, 1.35)
            + V)), 0.), mix(16., 97., gloss));
        let strip_b = pow(max(dot(normal, normalize(vec3(0.64, -0.37, 1.07)
            + V)), 0.), mix(22., 140., gloss));
        let vertical_band = pow(max(1. - abs(normal.y + 0.10) * 1.14, 0.), 13.);
        let horizontal_band = pow(max(1. - abs(normal.x - 0.17) * 1.22, 0.), 17.);
        let studio = clamp(strip_a * 0.64 + strip_b * 0.57
            + vertical_band * 0.18 + horizontal_band * 0.14, 0., 1.);
        let rim_key = pow(max(dot(outward, normalize(vec2(-0.74, -0.67))), 0.), 7.);
        let rim_fill = pow(max(dot(outward, normalize(vec2(0.68, 0.72))), 0.), 11.);
        let inside = (1. - front) * 0.19 + inner * 0.12 * (1. - rim_key);
        // The Old School calibration adjusts metal solidity without painting
        // a second flat rectangle over the refractive chrome. A low setting
        // preserves the original Glass Lab V7 transmission and highlights.
        let calibrated_solidness = clamp(lens.profile[5].y, 0., 0.56);
        color = mix(base, color, transmission * (1. - calibrated_solidness));
        if finish > 1.5 && finish < 2.5 {
            color = mix(color, vec3(0.09, 0.12, 0.16), 0.10);
        }
        color += env * reflection_amount * fresnel * 0.95;
        color += reflected_scene * (0.10 + 0.15 * chrome) * nv;
        color *= 1. - inside;
        color += fresnel * ggx * min(1.2, shine) * (0.31 + 0.35 * chrome);
        color += vec3(0.98, 0.985, 1.) * studio * (0.13 + 0.56 * chrome) * shine;
        color += vec3(0.92, 0.95, 1.) * ((rim_key * 0.65 + rim_fill * 0.36)
            * rim * (0.16 + 0.51 * edge) + inner * 0.065);
        color += vec3(0.34, 0.37, 0.41) * vertical_band * 0.12 * chrome;
        color += vec3(0.13, 0.62, 0.32) * accent
            * (rim_fill * rim * 0.28 + inner * 0.09);
    } else {
        // Clear, deep 1.48 IOR lens. Refraction and chromatic dispersion are
        // limited to the rounded rim; white text is rendered afterwards.
        let f0 = pow((1.48 - 1.) / (1.48 + 1.), 2.);
        let F = f0 + (1. - f0) * pow(1. - nv, 5.);
        color = mix(color, lens.tint.rgb, lens.profile[5].y
            * lens.tint.a * (0.65 + 0.20 * smoothstep(0.2, 0.85,
                dot(color, vec3(0.2126, 0.7152, 0.0722)))));
        color = mix(color, mix(vec3(0.66, 0.81, 0.92), ambient, 0.50),
            clamp(F * fresnel_strength * 0.82, 0., 0.62));
        let exponent = mix(20., 180., gloss);
        let spec = pow(max(dot(normal, H), 0.), exponent) * 0.48
            + pow(max(dot(normal, normalize(vec3(-light_dir * 0.7, 0.82) + V)),
                0.), exponent * 1.55) * 0.24;
        let rim_light = key_intensity * exp(-depth / (3.2 * dp)) * 0.54;
        let caustic = exp(-pow((depth - roll * 0.57) /
            max(dp, roll * 0.18), 2.)) * pow(secondary, 0.33) * 0.14;
        color += bounced * (wall * wall * 0.07 + caustic * 0.15) * light_spill;
        color = mix(color, vec3(0.97, 0.988, 1.),
            clamp((spec + rim_light + caustic) * shine, 0., 0.82));
    }
    return vec4(clamp(color, vec3(0.), vec3(1.)),
        coverage + shadow * (1. - coverage));
}
@fragment fn fs_blit(in: Out) -> @location(0) vec4<f32> {
    return textureSampleLevel(scene,scene_sampler,in.position.xy / lens.frame.xy,0.);
}
