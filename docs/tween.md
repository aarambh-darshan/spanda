# Tweens

A `Tween<T>` is the fundamental building block of **spanda**. It smoothly interpolates a value of type `T` from a `start` point to an `end` point over a given `duration`, applying an easing curve.

---

## Creating a Tween

Tweens use a **builder pattern**. You provide start and end values, then chain configuration methods:

```rust
use spanda::tween::Tween;
use spanda::easing::Easing;

let tween = Tween::new(0.0_f32, 100.0)
    .duration(2.5)        // 2.5 seconds (default: 1.0)
    .easing(Easing::EaseOutCubic)  // smooth deceleration (default: Linear)
    .delay(0.3)           // wait 0.3s before starting (default: 0.0)
    .build();
```

### Animatable Types

The generic type `T` can be anything that implements the `Animatable` (and therefore `Interpolate`) trait. Spanda ships blanket implementations for:

| Type | Use Case |
|------|----------|
| `f32` | Opacity, single-axis position, scale |
| `f64` | High-precision values |
| `[f32; 2]` | 2D position, size |
| `[f32; 3]` | 3D position, RGB colour |
| `[f32; 4]` | RGBA colour, quaternion components |
| `i32` | Discrete values (rounds to nearest) |

You can also animate your own types by implementing `Interpolate`:

```rust
use spanda::traits::Interpolate;

#[derive(Clone)]
struct Point { x: f32, y: f32 }

impl Interpolate for Point {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        Point {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }
}
```

---

## Running a Tween

Spanda does **not** run a loop for you. You call `update(dt)` every frame, passing the delta time (in seconds):

```rust
use spanda::traits::Update;

let mut tween = Tween::new(0.0_f32, 100.0)
    .duration(1.0)
    .build();

// In your game/render loop:
let dt = 0.016; // ~60 fps
let is_running = tween.update(dt); // returns false when complete

let current = tween.value();    // current interpolated value
let raw = tween.progress();     // 0.0 → 1.0 raw progress (before easing)
```

### How `value()` Works

Internally, `value()` does the following:

1. Computes `raw_t = elapsed / duration` (clamped to `[0.0, 1.0]`)
2. Applies the easing curve: `curved_t = easing.apply(raw_t)`
3. Interpolates: `start.lerp(&end, curved_t)`

---

## Tween Lifecycle (TweenState)

A tween goes through distinct phases:

| State | Description |
|-------|-------------|
| `Waiting` | Inside the delay period — animation hasn't started yet |
| `Running` | Actively interpolating between `start` and `end` |
| `Completed` | Reached the end — `value()` returns `end` |
| `Paused` | Manually paused via `pause()` |

```rust
use spanda::tween::TweenState;

let mut tween = Tween::new(0.0_f32, 100.0)
    .delay(0.5)
    .duration(1.0)
    .build();

assert_eq!(*tween.state(), TweenState::Waiting);

tween.update(0.5); // delay consumed
assert_eq!(*tween.state(), TweenState::Running);

tween.update(1.0); // animation complete
assert_eq!(*tween.state(), TweenState::Completed);
assert!(tween.is_complete());
```

---

## Easing

By default, tweens interpolate linearly. Apply an [Easing](easing.md) curve to give motion character:

```rust
use spanda::easing::Easing;

// Smooth deceleration — great for UI element exits
let tween = Tween::new(0.0_f32, 100.0)
    .easing(Easing::EaseOutCubic)
    .build();

// Bouncy landing — great for playful UI
let tween = Tween::new(0.0_f32, 100.0)
    .easing(Easing::EaseOutBounce)
    .build();

// Elastic snap — great for attention-grabbing entrances
let tween = Tween::new(0.0_f32, 100.0)
    .easing(Easing::EaseOutElastic)
    .build();
```

---

## Delays

Instruct a tween to wait before starting:

```rust
let tween = Tween::new(0.0_f32, 100.0)
    .delay(0.5) // waits 0.5 seconds before moving
    .duration(1.0)
    .build();
```

When the delay expires, any leftover time automatically carries into the animation — no frames are wasted.

---

## Advanced Controls

Control a tween mid-flight:

```rust
// Pause and resume
tween.pause();   // freezes at current position
tween.resume();  // continues from where it paused

// Seek to a specific progress point
tween.seek(0.5); // jump to 50% progress instantly

// Reverse the animation
tween.reverse(); // swaps start and end, resets to beginning

// Reset to the start
tween.reset();   // elapsed = 0, state = Waiting (if delay > 0) or Running
```

---

## Edge Cases & Safety

Spanda handles edge cases gracefully:

| Scenario | Behaviour |
|----------|-----------|
| `duration(0.0)` | Completes immediately on first `update()`, returns `end` |
| Very large `dt` (e.g., 999.0) | Clamps to completion, never overshoots |
| Negative `dt` | Treated as 0.0 — no backward time |
| `start == end` | Returns the value immediately, no interpolation |

---

## Using with AnimationDriver

For managing multiple tweens, use [`AnimationDriver`](integrations.md):

```rust
use spanda::driver::AnimationDriver;
use spanda::tween::Tween;

let mut driver = AnimationDriver::new();

// Add multiple tweens
driver.add(Tween::new(0.0_f32, 100.0).duration(1.0).build());
driver.add(Tween::new(0.0_f32, 255.0).duration(0.5).build());

// One tick advances them all; completed ones are auto-removed
driver.tick(0.5);
assert_eq!(driver.active_count(), 1); // second tween finished
```

---

## Multi-Dimensional Tweens

Animate 2D positions, 3D coordinates, or RGBA colours directly:

```rust
// 2D position
let mut pos_tween = Tween::new([0.0_f32, 0.0], [100.0, 200.0])
    .duration(1.0)
    .build();

pos_tween.update(0.5);
let pos = pos_tween.value(); // [50.0, 100.0]

// RGBA colour
let mut color_tween = Tween::new(
    [1.0_f32, 0.0, 0.0, 1.0], // red
    [0.0, 0.0, 1.0, 1.0],     // blue
).duration(2.0).build();
```

---

## Alternative Constructors

For readability, use the `from()` or `from_to()` aliases — they're identical to `new()`:

```rust
use spanda::tween::Tween;

// All three are equivalent:
let t1 = Tween::new(0.0_f32, 100.0).build();
let t2 = Tween::from(0.0_f32, 100.0).build();
let t3 = Tween::from_to(0.0_f32, 100.0).build();
```

---

## Looping

Tweens support the same `Loop` modes as keyframe tracks:

```rust
use spanda::{Tween, Loop};
use spanda::traits::Update;

// Bounce forever between start and end
let mut tween = Tween::new(0.0_f32, 100.0)
    .duration(1.0)
    .looping(Loop::PingPong)
    .build();

// Play exactly 3 times, then stop
let mut tween = Tween::new(0.0_f32, 100.0)
    .duration(1.0)
    .looping(Loop::Times(3))
    .build();

// Loop forever
let mut tween = Tween::new(0.0_f32, 100.0)
    .duration(1.0)
    .looping(Loop::Forever)
    .build();
```

| Loop Mode | Behaviour |
|-----------|-----------|
| `Loop::Once` | Play once and stop (default) |
| `Loop::Times(n)` | Play exactly `n` times, then complete |
| `Loop::Forever` | Reset and replay endlessly — never completes |
| `Loop::PingPong` | Swap start/end each cycle, replay forever |

### PingPong Details

`PingPong` swaps the `start` and `end` values after each cycle. The tween first interpolates from `start` to `end`, then from `end` back to `start`, and so on. Leftover time carries across cycle boundaries to prevent drift.

---

## Time Scale

Speed up or slow down a tween at build time or at runtime:

```rust
use spanda::tween::Tween;

// Build with 2x speed
let mut tween = Tween::new(0.0_f32, 100.0)
    .duration(1.0)
    .time_scale(2.0)
    .build();

// Change at runtime
tween.set_time_scale(0.5); // half speed

// Read current scale
let scale = tween.time_scale();
```

| Scale Value | Effect |
|-------------|--------|
| `2.0` | Twice as fast — 1.0s animation completes in 0.5s |
| `0.5` | Half speed — 1.0s animation takes 2.0s |
| `0.0` | Effectively paused — no progress |
| `1.0` | Normal speed (default) |

---

## Callbacks

> Requires `feature = "std"` and cannot be used with `feature = "bevy"`.

Register callbacks to fire at specific lifecycle points:

```rust,ignore
use spanda::tween::Tween;
use spanda::traits::Update;

let mut tween = Tween::new(0.0_f32, 100.0)
    .duration(1.0)
    .build();

// Fires once when the tween starts running
tween.on_start(|| println!("started!"));

// Fires every frame with the current interpolated value
tween.on_update(|val: f32| {
    println!("current value: {val}");
    // Perfect for reactive frameworks:
    // set_signal.set(val);
});

// Fires once when the tween completes
tween.on_complete(|| println!("done!"));
```

### Leptos Integration Pattern

The `on_update` callback receives the interpolated `T` value, enabling a clean bridge to reactive signals:

```rust,ignore
let (opacity, set_opacity) = create_signal(0.0_f32);

let mut tween = Tween::new(0.0_f32, 1.0)
    .duration(1.0)
    .easing(Easing::EaseOutCubic)
    .build();

tween.on_update(move |val: f32| set_opacity.set(val));
tween.on_complete(move || log::info!("fade complete"));
```

---

## Value Modifiers

Post-process the interpolated value before it's returned by `.value()`:

```rust
use spanda::tween::{Tween, snap_to, round_to};

let mut tween = Tween::new(0.0_f32, 100.0)
    .duration(1.0)
    .build();

// Snap to a 10-unit grid: values become 0, 10, 20, 30...
tween.set_modifier(snap_to(10.0));

// Or round to 2 decimal places:
tween.set_modifier(round_to(2));
```

The `on_update` callback receives the post-modifier value.

### Built-in Modifiers

| Function | What it does |
|----------|-------------|
| `snap_to(grid)` | Rounds to nearest multiple of `grid` |
| `round_to(decimals)` | Rounds to N decimal places |

You can also pass any `Fn(T) -> T`:

```rust,ignore
tween.set_modifier(|val: f32| val.max(0.0)); // clamp to non-negative
```

> Modifiers require `feature = "std"` (they use `Box<dyn Fn>`).

