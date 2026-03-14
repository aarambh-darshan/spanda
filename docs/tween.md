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
