# Easing

Easing curves define the **character** of an animation. Without easing, values move uniformly (linear). Easing changes the rate of change, making things start slow, end fast, bounce, snap, or overshoot.

In spanda, easing is controlled by the `Easing` enum — 31 standard named curves, CSS-compatible timing functions (`CubicBezier`, `Steps`), 5 advanced parametric easings, plus a `Custom` escape hatch for your own math.

---

## Using Built-in Easings

Supply an easing curve to a `TweenBuilder`:

```rust
use spanda::{Tween, Easing};

// Hits the end gently
let bounce = Tween::new(0.0_f32, 100.0)
    .easing(Easing::EaseOutBounce)
    .build();

// Extremely fast snappy transition
let snap = Tween::new(0.0_f32, 100.0)
    .easing(Easing::EaseInOutExpo)
    .build();

// Smooth, natural-feeling S-curve
let smooth = Tween::new(0.0_f32, 100.0)
    .easing(Easing::EaseInOutCubic)
    .build();
```

---

## Complete Easing Reference

For every curve type there are three variants:
1. **EaseIn**: Starts slow, accelerates
2. **EaseOut**: Starts fast, decelerates *(usually most natural for UI)*
3. **EaseInOut**: Starts slow, speeds up, slows down

### Standard Easing Curves (31)

| Group | Variants | Character |
|-------|----------|-----------|
| **Linear** | `Linear` | Constant speed — mechanical, robotic |
| **Polynomial** | `EaseIn/Out/InOutQuad` | Subtle acceleration (power of 2) |
| | `EaseIn/Out/InOutCubic` | Natural-feeling — **best general-purpose default** |
| | `EaseIn/Out/InOutQuart` | Sharper acceleration (power of 4) |
| | `EaseIn/Out/InOutQuint` | Very sharp (power of 5) |
| **Sinusoidal** | `EaseIn/Out/InOutSine` | Gentle, smooth, calming |
| **Exponential** | `EaseIn/Out/InOutExpo` | Extreme acceleration — snapping UI elements |
| **Circular** | `EaseIn/Out/InOutCirc` | Arc-shaped — abrupt starts or stops |
| **Back** | `EaseIn/Out/InOutBack` | Overshoots slightly — playful, satisfying |
| **Elastic** | `EaseIn/Out/InOutElastic` | Rubber-band snapping — dramatic |
| **Bounce** | `EaseIn/Out/InOutBounce` | Ball bouncing — game UIs |
| **Custom** | `Custom(fn(f32) -> f32)` | Your own curve |

### Choosing the Right Easing

| Use Case | Recommended Easing |
|----------|-------------------|
| General UI transitions | `EaseInOutCubic` |
| Element entrances | `EaseOutCubic` or `EaseOutBack` |
| Element exits | `EaseInCubic` |
| Button press feedback | `EaseOutBack` (overshoot) |
| Loading spinners | `EaseInOutSine` |
| Attention-grabbing | `EaseOutElastic` |
| Game / playful UI | `EaseOutBounce` |
| Notification pop-in | `EaseOutBack` |
| Snappy toggles | `EaseInOutExpo` |

---

## CSS-Compatible Timing Functions

These match the CSS `cubic-bezier()` and `steps()` timing functions exactly. Useful when porting web animations to Rust.

### `CubicBezier(x1, y1, x2, y2)`

A cubic Bezier curve on the unit square, evaluated with Newton-Raphson iteration (8 steps) and bisection fallback (20 steps) — the same algorithm used by browsers.

```rust
use spanda::{Tween, Easing};
use spanda::traits::Update;

// CSS "ease" equivalent
let mut tween = Tween::new(0.0_f32, 100.0)
    .duration(1.0)
    .easing(Easing::CubicBezier(0.25, 0.1, 0.25, 1.0))
    .build();

tween.update(0.5);
```

#### CSS Preset Reference

| CSS Keyword | spanda Equivalent |
|-------------|-------------------|
| `ease` | `CubicBezier(0.25, 0.1, 0.25, 1.0)` |
| `ease-in` | `CubicBezier(0.42, 0.0, 1.0, 1.0)` |
| `ease-out` | `CubicBezier(0.0, 0.0, 0.58, 1.0)` |
| `ease-in-out` | `CubicBezier(0.42, 0.0, 0.58, 1.0)` |
| `linear` | `Linear` or `CubicBezier(0.0, 0.0, 1.0, 1.0)` |

### `Steps(n)`

Discrete step easing — snaps the animation to `n` equal jumps instead of a smooth curve. Equivalent to CSS `steps(n)`.

```rust
use spanda::Easing;

let stepped = Easing::Steps(4);
// apply(0.0)  = 0.0
// apply(0.3)  = 0.25
// apply(0.6)  = 0.5
// apply(0.9)  = 0.75
// apply(1.0)  = 1.0
```

---

## Advanced Parametric Easings

These 5 easings take parameters to control their behaviour. They are the Rust equivalents of GSAP's premium easing plugins.

### `RoughEase { strength, points, seed }`

Adds deterministic noise to a linear curve, creating a hand-drawn or jittery feel. The noise is reproducible — the same `seed` always produces the same result.

| Parameter | Type | Range | Description |
|-----------|------|-------|-------------|
| `strength` | `f32` | `0.0..1.0` | Deviation amplitude (0 = linear, 1 = maximum jitter) |
| `points` | `u32` | `2..` | Number of noise sample points (more = finer grain) |
| `seed` | `u32` | any | PRNG seed for reproducibility |

```rust
use spanda::Easing;

let rough = Easing::RoughEase {
    strength: 0.3,
    points: 20,
    seed: 42,
};

// Direct function call (zero overhead):
use spanda::easing::rough_ease;
let value = rough_ease(0.5, 0.3, 20, 42);
```

**Use for:** hand-drawn animation, paper texture, organic imperfection.

### `SlowMo { ratio, power, yoyo_mode }`

A piecewise slow-fast-slow curve. The outer portions move slowly while the middle portion moves quickly — creating a dramatic pause-rush-pause effect.

| Parameter | Type | Range | Description |
|-----------|------|-------|-------------|
| `ratio` | `f32` | `0.0..1.0` | Fraction of the duration spent in "slow" portions (split between start and end) |
| `power` | `f32` | `0.0..` | How extreme the speed difference is (higher = more dramatic) |
| `yoyo_mode` | `bool` | | If true, the output returns to 0 at t=1 (pulse shape) |

```rust
use spanda::Easing;

let dramatic = Easing::SlowMo {
    ratio: 0.7,
    power: 0.8,
    yoyo_mode: false,
};
```

**Use for:** cinematic camera movements, dramatic reveals, slow-motion effects.

### `ExpoScale { start_scale, end_scale }`

Perceptual exponential scaling. Maps linear progress through a logarithmic curve so that proportional changes feel uniform. A zoom from 1x to 2x should feel the same as 2x to 4x.

| Parameter | Type | Range | Description |
|-----------|------|-------|-------------|
| `start_scale` | `f32` | `> 0.0` | Starting scale factor |
| `end_scale` | `f32` | `> 0.0` | Ending scale factor |

```rust
use spanda::Easing;

let zoom = Easing::ExpoScale {
    start_scale: 1.0,
    end_scale: 10.0,
};
```

**Use for:** zoom animations, frequency sweeps, volume faders, any logarithmic perception domain.

### `Wiggle { frequency, amplitude }`

Sinusoidal oscillation overlaid on the base curve. The output wiggles around the linear path instead of following it smoothly.

| Parameter | Type | Range | Description |
|-----------|------|-------|-------------|
| `frequency` | `f32` | `> 0.0` | Number of full oscillation cycles |
| `amplitude` | `f32` | `0.0..1.0` | Size of oscillation (0 = no wiggle, 1 = full deviation) |

```rust
use spanda::Easing;

let vibrate = Easing::Wiggle {
    frequency: 10.0,
    amplitude: 0.2,
};
```

**Use for:** vibration effects, shake/tremor, earthquake simulation, nervous energy.

### `CustomBounce { strength, squash }`

A parametric bounce with configurable decay and squash compression. More natural than the standard `EaseOutBounce` because you can tune bounce intensity and ground squash.

| Parameter | Type | Range | Description |
|-----------|------|-------|-------------|
| `strength` | `f32` | `0.0..1.0` | Bounce energy — how much each bounce returns (0 = no bounce, 1 = full) |
| `squash` | `f32` | `0.0..1.0` | Compression at bounce points (0 = none, 1 = maximum squash) |

```rust
use spanda::Easing;

let squishy = Easing::CustomBounce {
    strength: 0.6,
    squash: 0.3,
};
```

**Use for:** dropping objects, bouncing balls, playful UI elements, landing animations.

### Choosing the Right Advanced Easing

| Use Case | Recommended |
|----------|-------------|
| Hand-drawn feel | `RoughEase { strength: 0.2, points: 20, seed: 42 }` |
| Cinematic reveal | `SlowMo { ratio: 0.7, power: 0.8, yoyo_mode: false }` |
| Zoom animations | `ExpoScale { start_scale: 1.0, end_scale: 10.0 }` |
| Shake / vibration | `Wiggle { frequency: 8.0, amplitude: 0.15 }` |
| Bouncing drop | `CustomBounce { strength: 0.5, squash: 0.2 }` |
| Precise CSS match | `CubicBezier(0.25, 0.1, 0.25, 1.0)` |
| Sprite frame steps | `Steps(8)` |

---

## Custom Easings

If the built-in curves aren't enough, define your own using the `Custom` variant. An easing function takes a float in `[0.0, 1.0]` and returns a float:

```rust
use spanda::Easing;

// A "staircase" easing that snaps to 4 distinct steps
fn staircase(t: f32) -> f32 {
    let steps = 4.0;
    (t * steps).floor() / steps
}

let tween = Tween::new(0.0_f32, 100.0)
    .easing(Easing::Custom(staircase))
    .build();
```

> **Note**: Custom easings are **not serialisable** with `serde`. If you need to persist custom easings, store the name as a string separately and map it back on load.

---

## Direct Function Calls

For zero-overhead use (no enum dispatch), call the free functions directly:

```rust
use spanda::easing::{ease_out_cubic, ease_in_out_elastic};

let t = 0.5;
let curved = ease_out_cubic(t);       // 0.875
let elastic = ease_in_out_elastic(t); // varies
```

The advanced easings also have free functions:

```rust
use spanda::easing::{rough_ease, slow_mo, expo_scale, wiggle_ease, custom_bounce};

let noisy = rough_ease(0.5, 0.3, 20, 42);
let dramatic = slow_mo(0.5, 0.7, 0.8, false);
let zoomed = expo_scale(0.5, 1.0, 10.0);
let shaky = wiggle_ease(0.5, 10.0, 0.2);
let bouncy = custom_bounce(0.5, 0.6, 0.3);
```

---

## Utility Methods

### `Easing::apply(t)`

Evaluate any easing at a given `t ∈ [0.0, 1.0]`. Input is **automatically clamped** — you never need to guard the caller:

```rust
let value = Easing::EaseOutBounce.apply(0.7);
```

### `Easing::name()`

Returns a human-readable string — useful for debug UIs and serialisation:

```rust
assert_eq!(Easing::EaseOutCubic.name(), "EaseOutCubic");
```

### `Easing::all_named()`

Returns a static slice of all 31 standard named variants (excludes `Custom`, `CubicBezier`, `Steps`, and the 5 advanced parametric easings, since those require constructor arguments). Useful for building picker UIs or running benchmark sweeps:

```rust
for easing in Easing::all_named() {
    println!("{}: apply(0.5) = {}", easing.name(), easing.apply(0.5));
}
```

---

## Guarantees

All built-in easing functions guarantee:
- `apply(0.0)` returns exactly `0.0`
- `apply(1.0)` returns exactly `1.0`
- Input is clamped to `[0.0, 1.0]` before evaluation
- No panics for any input value

Some curves (like `Back` and `Elastic`) may produce values **outside `[0.0, 1.0]`** between the endpoints — this is intentional (overshoot/undershoot). The output range for these curves is approximately `[-0.5, 1.5]`.

The `Easing` enum is marked `#[non_exhaustive]`, meaning new variants may be added in future minor releases without a breaking change.

---

## Serde Support

With the `serde` feature enabled, all named easing variants are serialisable/deserialisable:

```toml
[dependencies]
spanda = { version = "0.8", features = ["serde"] }
```

The `Custom` variant is `#[serde(skip)]` — function pointers cannot be serialised.
