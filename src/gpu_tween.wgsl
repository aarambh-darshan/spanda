// ── Tween parameters (per-tween) ────────────────────────────────────────────

struct TweenParams {
    start: f32,
    end_val: f32,
    duration: f32,
    delay: f32,
    easing_id: u32,
    _pad1: u32,
    _pad2: u32,
    _pad3: u32,
}

// ── Global uniforms ─────────────────────────────────────────────────────────

struct Globals {
    elapsed: f32,
    count: u32,
}

@group(0) @binding(0) var<storage, read> params: array<TweenParams>;
@group(0) @binding(1) var<storage, read_write> results: array<f32>;
@group(0) @binding(2) var<uniform> globals: Globals;

// ── Easing functions (GPU-side) ─────────────────────────────────────────────

fn ease_linear(t: f32) -> f32 { return t; }

fn ease_in_quad(t: f32) -> f32 { return t * t; }
fn ease_out_quad(t: f32) -> f32 { return t * (2.0 - t); }
fn ease_in_out_quad(t: f32) -> f32 {
    if t < 0.5 { return 2.0 * t * t; }
    return -1.0 + (4.0 - 2.0 * t) * t;
}

fn ease_in_cubic(t: f32) -> f32 { return t * t * t; }
fn ease_out_cubic(t: f32) -> f32 {
    let u = t - 1.0;
    return u * u * u + 1.0;
}
fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 { return 4.0 * t * t * t; }
    let u = 2.0 * t - 2.0;
    return 0.5 * u * u * u + 1.0;
}

fn ease_in_quart(t: f32) -> f32 { return t * t * t * t; }
fn ease_out_quart(t: f32) -> f32 {
    let u = t - 1.0;
    return 1.0 - u * u * u * u;
}
fn ease_in_out_quart(t: f32) -> f32 {
    if t < 0.5 { return 8.0 * t * t * t * t; }
    let u = t - 1.0;
    return 1.0 - 8.0 * u * u * u * u;
}

fn apply_easing(id: u32, t: f32) -> f32 {
    switch id {
        case 0u: { return ease_linear(t); }
        case 1u: { return ease_in_quad(t); }
        case 2u: { return ease_out_quad(t); }
        case 3u: { return ease_in_out_quad(t); }
        case 4u: { return ease_in_cubic(t); }
        case 5u: { return ease_out_cubic(t); }
        case 6u: { return ease_in_out_cubic(t); }
        case 7u: { return ease_in_quart(t); }
        case 8u: { return ease_out_quart(t); }
        case 9u: { return ease_in_out_quart(t); }
        default: { return t; }
    }
}

// ── Main compute kernel ─────────────────────────────────────────────────────

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3u) {
    let idx = id.x;
    if idx >= globals.count { return; }

    let p = params[idx];
    let effective_elapsed = globals.elapsed - p.delay;

    if effective_elapsed <= 0.0 {
        results[idx] = p.start;
        return;
    }

    var raw_t = effective_elapsed / p.duration;
    raw_t = clamp(raw_t, 0.0, 1.0);

    let eased_t = apply_easing(p.easing_id, raw_t);
    results[idx] = p.start + (p.end_val - p.start) * eased_t;
}
