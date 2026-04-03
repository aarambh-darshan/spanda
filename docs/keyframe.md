# Keyframe Tracks

While [Tweens](tween.md) animate from A to B, a `KeyframeTrack<T>` animates through **any number of points** — A → B → C → D — with specific timings and distinct easing between each segment.

---

## Creating a Keyframe Track

Build a track by pushing pairs of `(time, value)`. The `time` is an absolute timestamp in seconds from the start of the track:

```rust
use spanda::keyframe::{KeyframeTrack, Loop};

let track = KeyframeTrack::new()
    .push(0.0, 0.0_f32)   // Start at 0
    .push(1.0, 100.0)     // Move to 100 over 1 second
    .push(3.0, 50.0)      // Slide back to 50 over 2 seconds
    .push(4.0, 200.0)     // Shoot up to 200 over 1 second
    .looping(Loop::Once);  // Play once and stop
```

Keyframes are **automatically sorted** by time internally — you can push them in any order.

---

## Running a Keyframe Track

Like tweens, keyframe tracks are driven by `update(dt)`:

```rust
use spanda::traits::Update;

let mut track = KeyframeTrack::new()
    .push(0.0, 0.0_f32)
    .push(1.0, 100.0)
    .push(2.0, 0.0);

// In your render loop:
let is_running = track.update(0.016);
// Output current position (since we pushed frames, it will be Some)
let current = track.value().unwrap();  // current interpolated value
```

### Direct Time Lookup

You can also evaluate the track at any arbitrary time without advancing the internal clock:

```rust
let value = track.value_at(0.5);  // evaluate at t=0.5s, pure function
let value = track.value_at(1.5);  // evaluate at t=1.5s
```

This is useful for scrubbing, previewing, or building tools.

---

## Looping Behaviour

Keyframe tracks support 4 loop modes via the `Loop` enum:

```rust
use spanda::keyframe::Loop;

// 1. Play once and stop at the final keyframe (default)
track.looping(Loop::Once);

// 2. Loop forever — great for ambient animations
track.looping(Loop::Forever);

// 3. Play forward, then perfectly in reverse, repeating
track.looping(Loop::PingPong);

// 4. Run exactly N times, then stop
track.looping(Loop::Times(3));
```

### How PingPong Works

PingPong computes the effective time using this cycle:

```
total_duration = time of last keyframe
cycle_length = 2 × total_duration
cycle_t = elapsed % cycle_length

if cycle_t ≤ total_duration:
    t = cycle_t           (forward)
else:
    t = 2 × duration - cycle_t  (backward)
```

This produces a smooth, seamless forward-backward loop with no discontinuities.

### Loop::Times(n) Completion

`Loop::Times(n)` plays the track exactly `n` times, then sets `is_complete()` to `true`. Useful for controlled repetition (e.g., "flash 3 times"):

```rust
let mut track = KeyframeTrack::new()
    .push(0.0, 0.0_f32)
    .push(0.5, 1.0)
    .push(1.0, 0.0)
    .looping(Loop::Times(3));

// After 3.0 seconds of updates, is_complete() returns true
```

---

## Easing Between Keyframes

By default, interpolation between keyframes is **linear**. You can apply a specific easing to the segment **following** a keyframe:

```rust
use spanda::easing::Easing;

let track = KeyframeTrack::new()
    // Move 0 → 100 with a bouncy exit
    .push_with_easing(0.0, 0.0_f32, Easing::EaseOutBounce)

    // Move 100 → 0 with a smooth S-curve
    .push_with_easing(1.0, 100.0, Easing::EaseInOutCubic)

    // Final keyframe (easing on this frame is ignored — no next frame)
    .push(3.0, 0.0);
```

---

## Key Methods

| Method | Description |
|--------|-------------|
| `push(time, value)` | Add a keyframe with linear easing to next |
| `push_with_easing(time, value, easing)` | Add a keyframe with specific easing |
| `looping(mode)` | Set loop mode: `Once`, `Forever`, `PingPong`, `Times(n)` |
| `value()` | Current value based on internal elapsed time (`Option<T>`) |
| `value_at(t)` | Value at specific absolute time (`Option<T>`) |
| `duration()` | Total duration (time of last keyframe) |
| `is_complete()` | Whether the track has finished playing |
| `reset()` | Reset elapsed time, loop count, and completion state |

---

## Performance

- **Binary search** (`O(log N)`) finds the correct segment — no linear scan even with thousands of keyframes
- Keyframes are kept **pre-sorted** internally after each `push()` call
- The `value_at()` method is **pure** — it doesn't modify internal state and can be called freely

---

## Edge Cases

| Scenario | Behaviour |
|----------|-----------|
| Empty track | `value_at()` panics (undefined) |
| Single keyframe | Returns that value for all `t` |
| `t` before first keyframe | Clamps to first value |
| `t` after last keyframe | Clamps to last value |
| Two keyframes at same time | Returns the later value (zero-length segment) |
| Negative `dt` in `update()` | Treated as 0.0 |
