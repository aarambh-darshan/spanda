# Keyframe Tracks

While [Tweens](tween.md) are great for moving from A to B, sometimes you need to move from A to B to C to D, with specific timings and distinct easings between each segment. That's a `KeyframeTrack<T>`.

## Creating a Keyframe Track

You build a track by pushing pairs of `(time, value)`. `time` is a float representing the absolute timestamp (in seconds) of that keyframe, starting from `0.0`.

```rust
use spanda::keyframe::{KeyframeTrack, Loop};

let track = KeyframeTrack::new()
    .push(0.0, 0.0_f32)   // Start at 0
    .push(1.0, 100.0)     // Move to 100 immediately
    .push(3.0, 50.0)      // Slowly slide back to 50 over 2 seconds
    .push(4.0, 200.0)     // Shoot up to 200
    .looping(Loop::Once); // Default behavior
```

## Looping Behavior

Unlike Tweens, KeyframeTracks are highly likely to be used for continuous, ambient looping animations (e.g., a pulsing glow or a spinning loading icon). Spanda natively supports 4 looping modes via the `Loop` enum:

```rust
// 1. Run exactly once and stop at the final keyframe
track.looping(Loop::Once);

// 2. Loop continuously forever
track.looping(Loop::Forever);

// 3. Play forward, hit the end, then play perfectly in reverse back to the start
track.looping(Loop::PingPong);

// 4. Run exactly N times, then stop
track.looping(Loop::Times(3));
```

## Easing Between Keyframes

By default, interpolation between two keyframes is linear. However, you can provide an explicit `Easing` curve to apply *only to the segment following that keyframe*.

The easing is applied to the segment between the keyframe you attach it to, and the next keyframe.

```rust
use spanda::easing::Easing;

let track = KeyframeTrack::new()
    // Move 0 -> 100 with a bouncy exit
    .push_with_easing(0.0, 0.0_f32, Easing::EaseOutBounce)
    
    // Smoothly slide 100 -> 0
    .push_with_easing(1.0, 100.0, Easing::EaseInOutCubic)
    
    // Final keyframe (easing argument ignored because there's no next frame)
    .push(3.0, 0.0);
```

### Note on Performance
To evaluate the value at any timestamp `t`, `KeyframeTrack` uses a fast binary search (`O(log N)`) to find the correct surrounding keyframes, avoiding a linear scan even if you have thousands of keyframes.
