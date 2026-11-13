// Horizon XMB Mobile — original procedural ribbon atmosphere.
// Inspired by the *visual grammar* of the late-2000s console XMB, not its assets.
// Evaluated only for the private UI sentinel (uv.x < -2.0) in hud.wgsl.
// p: normalized physical framebuffer coordinates (top-left origin).
// phase: slowly advancing launcher seconds, zero when motion is disabled.
// quality: 0..1, used only to skip fine detail on power-saving paths.
//
// A single fullscreen HUD quad; no extra GPU framebuffer, texture,
// fullscreen pass, CPU image generation, uploads or per-frame allocations.
fn xmb_gaussian(distance: f32, width: f32) -> f32 {
    let x = distance / max(width, 0.0001);
    return exp(-x * x);
}
fn xmb_ribbon(p: vec2<f32>, time: f32, seed: f32) -> vec3<f32> {
    let x = p.x;
    let a = time * 0.062 + seed * 1.78;
    // Several frequencies create silk-like folds without procedural noise.
    // The center gently bends and drifts at PS3-like background speeds.
    let center = 0.665
        + 0.094 * sin(x * 4.27 + a * 0.68 + seed * 1.7)
        + 0.048 * sin(x * 8.32 - a * 0.47 + seed * 2.4)
        + (seed - 1.0) * 0.065;
    let distance = p.y - center;
    let film = xmb_gaussian(distance, 0.052 + seed * 0.011);
    let silk = xmb_gaussian(distance - 0.026, 0.125) * 0.26;
    let core = xmb_gaussian(distance + 0.014, 0.008 + seed * 0.003);
    let rim = xmb_gaussian(distance - 0.045, 0.015);
    let longitudinal = 0.60 + 0.40 * sin(x * 3.15 - time * 0.045 + seed * 2.3);
    return vec3<f32>(film + silk, core + rim * 0.45, longitudinal);
}

// The UI's appearance must come from this fragment, never from old Chrome,
// a cached sheet, tiled material or opaque dashboard rects.
fn xmb_wallpaper(p: vec2<f32>, phase: f32, quality: f32) -> vec3<f32> {
    let x = clamp(p.x, 0.0, 1.0);
    let y = clamp(p.y, 0.0, 1.0);
    let time = phase;
    let detail = clamp(quality, 0.0, 1.0);

    // Optical deep navy: bottom bloom is broad, atmosphere remains nearly
    // black in the upper half so system text has real contrast.
    let lower = smoothstep(0.25, 1.05, y);
    let left_ambient = xmb_gaussian(x - 0.18, 0.58) * lower * 0.16;
    let base = mix(
        vec3<f32>(0.006, 0.013, 0.035),
        vec3<f32>(0.020, 0.046, 0.102),
        lower * 0.70
    );
    var color = base
        + vec3<f32>(0.004, 0.013, 0.037) * lower * lower
        + vec3<f32>(0.012, 0.032, 0.067) * left_ambient;

    // The broad, quiet aurora makes the luminous ribbons seem embedded in
    // the distance rather than pasted on top of the interface.
    let quiet_center = 0.76 + 0.048 * sin(x * 3.1 + time * 0.019);
    let aurora = xmb_gaussian(y - quiet_center, 0.24);
    color += vec3<f32>(0.014, 0.047, 0.104) * aurora * (0.35 + 0.65 * lower);

    // Three original interlaced light veils, rendered as spectral layers.
    let back = xmb_ribbon(vec2<f32>(x, y), time, 0.22);
    let mid = xmb_ribbon(vec2<f32>(x, y), time, 1.12);
    let front = xmb_ribbon(vec2<f32>(x, y), time, 1.92);

    // Semi-transparent sheet body: soft blue-aqua; thin silver-blue crests;
    // independent per-ribbon modulation for a flowing translucent look.
    color += vec3<f32>(0.015, 0.065, 0.125)
        * back.x * (0.63 + 0.37 * back.z);
    color += vec3<f32>(0.012, 0.102, 0.171)
        * mid.x * (0.55 + 0.45 * mid.z);
    color += vec3<f32>(0.025, 0.079, 0.154)
        * front.x * (0.68 + 0.32 * front.z);
    color += vec3<f32>(0.084, 0.177, 0.253)
        * (back.y * 0.33 + mid.y * 0.55 + front.y * 0.47);

    // Long, faint cross-light rather than bloom sprites or glassmorphism.
    let halo = xmb_gaussian(y - (0.71 + 0.032 * sin(x * 6.0 + time * 0.02)), 0.12);
    color += vec3<f32>(0.008, 0.026, 0.046) * halo;

    // Finest silver glint only on full detail; conditional is uniform across
    // the fullscreen quad and avoids texture lookups on all mobile GPUs.
    if detail > 0.65 {
        let seam = 0.688
            + 0.096 * sin(x * 4.27 + time * 0.042 + 1.9)
            + 0.018 * sin(x * 12.6 + time * 0.035);
        let glint = xmb_gaussian(y - seam, 0.0045);
        color += vec3<f32>(0.033, 0.080, 0.116) * glint;
    }

    // Subpixel-safe deterministic dither (no time-dependent shimmer).
    // Faint enough to suppress 8-bit banding in the navy gradients.
    let grain = fract(sin(dot(p * vec2<f32>(1920.0, 1080.0),
        vec2<f32>(12.9898, 78.233))) * 43758.5453) - 0.5;
    color += vec3<f32>(grain * 0.00165);

    let vignette = 1.0
        - 0.14 * pow(abs(x * 2.0 - 1.0), 2.0)
        - 0.08 * pow(abs(y * 2.0 - 1.0), 2.0);
    // Never approach white; all UI glyphs remain brighter than the backdrop.
    return clamp(color * vignette, vec3<f32>(0.002), vec3<f32>(0.48));
}
