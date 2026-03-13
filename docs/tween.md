# Tweens

A `Tween<T>` is the fundamental building block of `spanda`. It smoothly interpolates a value of type `T` from a `start` point to an `end` point over a given `duration`.

## Creating a Tween

Tweens use a builder pattern. You must provide the start and end values upfront:

```rust
use spanda::tween::Tween;

// Animates an f32 from 0.0 to 100.0
let tween = Tween::new(0.0_f32, 100.0)
    .duration(2.5) // 2.5 seconds
    .build();
```

### Animatable Types

The generic type `T` can be anything that implements the `Animatable` (and therefore `Interpolate`) trait. Spanda provides blanket implementations for:
- `f32`, `f64`
- `[f32; 2]`, `[f32; 3]`, `[f32; 4]`
- `i32`

## Customising Behavior

### Easing

By default, tweens are linear. To make them feel natural, you apply an [Easing](easing.md) curve:

```rust
use spanda::easing::Easing;

let tween = Tween::new(0.0_f32, 100.0)
    .duration(1.0)
    .easing(Easing::EaseOutElastic) // Adds a nice bouncy stop
    .build();
```

### Delays

You can instruct a tween to wait before starting:

```rust
let tween = Tween::new(0.0_f32, 100.0)
    .delay(0.5) // Waits 0.5 seconds before moving
    .build();
```

## Running a Tween

Spanda does not run the loop for you. You must call `update(dt)` every frame, passing the delta time (in seconds) since the last frame:

```rust
use spanda::traits::Update;

let mut tween = Tween::new(0.0, 100.0).build();

// In your game/render loop:
let dt = 0.016; // e.g. 60fps
let is_running = tween.update(dt); // returns false when complete

let current = tween.value(); // Get the interpolated value
let raw = tween.progress();  // Get 0.0 -> 1.0 progress
```

## Advanced Controls

You can control a tween mid-flight:

```rust
tween.pause();
tween.resume();
tween.seek(0.5); // Jump halfway through
tween.reverse(); // Swap start/end and restart
tween.reset();   // Start over from 0
```
