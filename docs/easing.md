# Easing

Easing curves define the **character** of an animation. Without easing, values move uniformly (linear). Easing changes the rate of change, making things start slow, end fast, bounce, snap, or overshoot.

In spanda, easing is controlled by the `Easing` enum — 31 standard named curves, plus a `Custom` escape hatch for your own math.

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

### All 31 Easing Curves

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

## Custom Easings

If the 31 built-in curves aren't enough, define your own using the `Custom` variant. An easing function takes a float in `[0.0, 1.0]` and returns a float:

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

This is useful in hot loops or when you need the raw mathematical function.

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

Returns a static slice of all 31 named variants (excludes `Custom`). Useful for building picker UIs or running benchmark sweeps:

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

---

## Serde Support

With the `serde` feature enabled, all named easing variants are serialisable/deserialisable:

```toml
[dependencies]
spanda = { version = "0.1", features = ["serde"] }
```

The `Custom` variant is `#[serde(skip)]` — function pointers cannot be serialised.
