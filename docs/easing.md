# Easing

Easing curves defines the "character" of an animation. By default, values move uniformly over time (linear easing). Easing changes the rate of change, making things start slow, end fast, bounce, or snap.

In `spanda`, easing is controlled by the `Easing` enum, which provides 31 standard named curves, plus an escape hatch for your own custom math.

## Using Built-in Easings

You can supply an easing curve directly into a `TweenBuilder`:

```rust
use spanda::{Tween, Easing};

let bounce = Tween::new(0.0_f32, 100.0)
    .easing(Easing::EaseOutBounce) // Hits the end and bounces back
    .build();

let snap = Tween::new(0.0_f32, 100.0)
    .easing(Easing::EaseInOutExpo) // Extremely fast snappy middle
    .build();

let smooth = Tween::new(0.0_f32, 100.0)
    .easing(Easing::EaseInOutCubic) // Smooth, natural-feeling s-curve
    .build();
```

## Easing Groups

Spanda bundles easing curves into the standard groupings. For every curve type, there are three variants:
1. `EaseIn`: Starts slow, accelerates.
2. `EaseOut`: Starts fast, decelerates (usually the most natural for UI).
3. `EaseInOut`: Starts slow, speeds up in the middle, and slows down at the end.

- **Sine / Quad / Cubic / Quart / Quint**: Simple mathematical powers. Cubic is generally the most pleasant default.
- **Expo**: Extreme snapping.
- **Circ**: Circular arcs, starting or ending very abruptly.
- **Back**: Pulls back slightly before accelerating, or overshoots slightly before settling.
- **Elastic**: Wobbly, rubber-band like snapping.
- **Bounce**: Dropping a ball realistically.

## Custom Easings

If the 31 built-in curves aren't enough, you can define your own using the `Custom` variant. An easing function takes a float from `[0.0, 1.0]` and returns a float.

*Note: Custom easings are not serializable with `serde`.*

```rust
use spanda::Easing;

// A custom "staircase" easing that snaps to 4 distinct steps
fn staircase(t: f32) -> f32 {
    let steps = 4.0;
    (t * steps).floor() / steps
}

let tween = Tween::new(0.0_f32, 100.0)
    .easing(Easing::Custom(staircase))
    .build();
```
