# Timelines & Sequences

When you have multiple animations, managing them individually becomes difficult. `spanda` provides two orchestrators:

- **Timeline**: Runs multiple animations *concurrently* (at the same time), with per-entry time offsets.
- **Sequence**: Runs multiple animations *sequentially* (one after another), with optional gaps.

---

## Sequences

A `Sequence` is a builder for a `Timeline` that automatically calculates the correct start times to chain animations end-to-end. You can also inject specific time gaps between them.

```rust
use spanda::timeline::Sequence;
use spanda::tween::Tween;
use spanda::traits::Update;

let mut seq = Sequence::new()
    // Start immediately
    .then(Tween::new(0.0_f32, 100.0).duration(1.0).build(), 1.0)
    
    // Wait 0.5s after the first tween finishes
    .gap(0.5) 
    
    // Then run this tween 
    .then(Tween::new(100.0_f32, 0.0).duration(1.0).build(), 1.0)
    .build();

seq.play(); // Timelines must be played explicitly

// update(dt) returns true as long as ANY animation in the sequence is running
while seq.update(0.016) {
    // Both timeline entries are ticked over the total duration of 2.5s
}
```

### How `.then()` Works

Each `.then()` call takes two arguments:

1. **The animation** — any type implementing `Update` (Tween, KeyframeTrack, Spring, etc.)
2. **The duration** — how long this animation lasts in seconds

The duration is required because the trait object erases the animation's type, so the Sequence can't query it directly. The Sequence uses it internally to calculate when the next animation should start.

### Gaps

`.gap(seconds)` inserts a pause in the sequence. The next `.then()` will start after the gap:

```rust
let seq = Sequence::new()
    .then(fade_in, 0.5)       // 0.0 – 0.5
    .gap(0.2)                 // 0.5 – 0.7  (pause)
    .then(slide_up, 0.8)      // 0.7 – 1.5
    .build();
```

### Looping a Sequence

Apply a loop mode to repeat the entire sequence:

```rust
use spanda::keyframe::Loop;

let mut seq = Sequence::new()
    .then(pulse_tween, 0.5)
    .then(fade_tween, 0.3)
    .looping(Loop::Forever)
    .build();
```

---

## Timelines

If you need absolute control over exactly when an animation starts relative to the timeline's beginning (timestamp `0.0`), use a `Timeline` directly.

```rust
use spanda::timeline::Timeline;
use spanda::tween::Tween;
use spanda::easing::Easing;

let mut timeline = Timeline::new()
    // Starts immediately
    .add("fade_in", Tween::new(0.0_f32, 1.0).duration(0.5).build(), 0.0)
    
    // Starts 0.4 seconds into the timeline, overlapping the fade
    .add("slide_up", Tween::new(50.0_f32, 0.0).duration(0.8).build(), 0.4)
    
    // Starts 1.2 seconds in
    .add("scale", Tween::new(1.0_f32, 1.5).duration(0.5).build(), 1.2);

timeline.play();
```

### How Time Offsets Work

The third argument to `.add()` is the **absolute start time** (in seconds) from the beginning of the timeline. This allows overlapping animations:

```
fade_in:  |█████|                    (0.0 – 0.5)
slide_up:     |████████|             (0.4 – 1.2)  ← overlaps the fade!
scale:                   |█████|     (1.2 – 1.7)
```

### Labels

Every entry has a string label. Labels are useful for debugging and identifying entries. They're provided as the first argument to `.add()`:

```rust
timeline.add("hero_entrance", animation, 0.0);
timeline.add("subtitle_fade", animation, 0.3);
```

---

## Timeline Lifecycle (TimelineState)

A timeline goes through distinct phases:

| State | Description |
|-------|-------------|
| `Idle` | Created but `.play()` hasn't been called yet |
| `Playing` | Actively ticking all entries |
| `Paused` | Manually paused via `.pause()` |
| `Completed` | All entries have finished |

```rust
use spanda::timeline::TimelineState;

let mut tl = Timeline::new()
    .add("a", tween, 0.0);

assert_eq!(*tl.state(), TimelineState::Idle);

tl.play();
assert_eq!(*tl.state(), TimelineState::Playing);

tl.update(10.0);
assert_eq!(*tl.state(), TimelineState::Completed);
```

---

## Playback Controls

Control a timeline mid-flight:

```rust
// Pause and resume
timeline.pause();   // freezes all entries at their current position
timeline.resume();  // continues from where they paused

// Seek to a specific time
timeline.seek(0.5); // jump to 0.5 seconds into the timeline

// Reset to the beginning
timeline.reset();   // elapsed = 0, state = Idle, all entries reset
```

### Progress & Duration

```rust
let total = timeline.duration();   // total length in seconds
let progress = timeline.progress(); // 0.0 → 1.0
```

---

## Looping

Timelines support the same `Loop` enum as [KeyframeTracks](keyframe.md):

```rust
use spanda::keyframe::Loop;

let mut timeline = Timeline::new()
    .add("pulse", pulse_tween, 0.0)
    .looping(Loop::Forever);

timeline.play();
// This timeline will never complete — it loops endlessly
```

| Loop Mode | Behaviour |
|-----------|-----------|
| `Loop::Once` | Play once and stop (default) |
| `Loop::Forever` | Loop endlessly |
| `Loop::PingPong` | Play forward, then reverse, repeating |
| `Loop::Times(n)` | Play exactly `n` times, then stop |

---

## Callbacks

With the `std` feature enabled, you can register callbacks that fire when the timeline completes:

```rust
timeline.on_finish(|| {
    println!("Timeline finished!");
});
```

**Callbacks and `Arc`**: For shared state (e.g., setting a flag), use `Arc` + `AtomicBool`:

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

let done = Arc::new(AtomicBool::new(false));
let done_clone = done.clone();

timeline.on_finish(move || {
    done_clone.store(true, Ordering::SeqCst);
});
```

> **Note**: Callbacks require `feature = "std"` because they use `Box<dyn FnMut()>`.

---

## Key Methods

### Timeline

| Method | Description |
|--------|-------------|
| `Timeline::new()` | Create an empty timeline |
| `.add(label, animation, start_at)` | Add a labelled animation at a specific time |
| `.add_at(label, animation, duration, at)` | Add using relative positioning (see below) |
| `.looping(mode)` | Set loop mode |
| `.play()` | Start playback |
| `.pause()` | Pause playback |
| `.resume()` | Resume from pause |
| `.seek(t)` | Jump to a specific time |
| `.reset()` | Reset to beginning |
| `.duration()` | Total timeline length in seconds |
| `.progress()` | Playback progress (0.0 → 1.0) |
| `.state()` | Current `TimelineState` |
| `.set_time_scale(scale)` | Set playback speed multiplier |
| `.time_scale()` | Get current speed multiplier |
| `.on_finish(callback)` | Register a completion callback (`std` only) |

### Sequence

| Method | Description |
|--------|-------------|
| `Sequence::new()` | Create an empty sequence |
| `.then(animation, duration)` | Append an animation |
| `.gap(seconds)` | Insert a pause |
| `.looping(mode)` | Set loop mode for the resulting timeline |
| `.build()` | Build the final `Timeline` |

---

## Relative Positioning (`At` enum)

Instead of calculating absolute start times manually, use `At` to position animations relative to existing entries — GSAP-style.

```rust
use spanda::timeline::{Timeline, At};
use spanda::tween::Tween;
use spanda::easing::Easing;

let mut tl = Timeline::new()
    .add("fade", Tween::new(0.0_f32, 1.0).duration(0.5).build(), 0.0);

// At::Start — always at t=0
tl.add_at("intro", Tween::new(0.0_f32, 1.0).duration(0.3).build(), 0.3, At::Start);

// At::End — after the latest-ending entry
tl.add_at("slide", Tween::new(0.0_f32, 100.0).duration(0.8).build(), 0.8, At::End);

// At::Label — same start time as a named entry
tl.add_at("glow", Tween::new(0.0_f32, 1.0).duration(0.3).build(), 0.3, At::Label("fade"));

// At::Offset — relative to previous entry's end (positive = gap, negative = overlap)
tl.add_at("pop", Tween::new(0.0_f32, 1.0).duration(0.2).build(), 0.2, At::Offset(0.1));

tl.play();
```

### `At` Variants

| Variant | Places at |
|---------|-----------|
| `At::Start` | Absolute t=0 |
| `At::End` | After the latest-ending entry (`start_at + duration`) |
| `At::Label("name")` | Same start time as the entry with that label |
| `At::Offset(f32)` | Last entry's end + offset (positive = gap, negative = overlap) |

### `.add()` vs `.add_at()`

| | `.add()` | `.add_at()` |
|---|---------|------------|
| **Signature** | `add(label, animation, start_at)` | `add_at(label, animation, duration, at)` |
| **Takes self** | `self` (consuming, for builder chains) | `&mut self` (for post-construction) |
| **Positioning** | Absolute seconds | Relative via `At` enum |
| **Duration** | Not stored (0.0) | Stored (needed for `At::End` / `At::Offset`) |

---

## Time Scale

Speed up or slow down an entire timeline at runtime:

```rust
use spanda::timeline::Timeline;
use spanda::tween::Tween;

let mut tl = Timeline::new()
    .add("t1", Tween::new(0.0_f32, 1.0).duration(1.0).build(), 0.0);

tl.set_time_scale(2.0); // 2x speed — completes in half the time
tl.play();

// Or slow down:
tl.set_time_scale(0.5); // half speed
```

The time scale multiplies the `dt` passed to each entry. A scale of 0.0 effectively pauses the timeline (though `.pause()` is more idiomatic).

---

## Stagger Utility

Create a `Timeline` where multiple animations start with evenly-spaced delays:

```rust
use spanda::timeline::stagger;
use spanda::tween::Tween;
use spanda::traits::Update;

let tweens: Vec<_> = (0..5).map(|i| {
    let end = (i + 1) as f32 * 20.0;
    (Tween::new(0.0_f32, end).duration(0.5).build(), 0.5)
}).collect();

let mut timeline = stagger(tweens, 0.1);
timeline.play();
```

Each tuple is `(animation, duration)`. Animation `i` starts at `i * stagger_delay`:

```
Animation 0: ████████               (0.0 – 0.5)
Animation 1:   ████████             (0.1 – 0.6)
Animation 2:     ████████           (0.2 – 0.7)
Animation 3:       ████████         (0.3 – 0.8)
Animation 4:         ████████       (0.4 – 0.9)
```

The `stagger()` function works with any type implementing `Update + 'static`.

---

## Nesting

Since `Timeline` implements `Update`, you can nest timelines inside other timelines or sequences for complex, multi-layered compositions:

```rust
let intro = Sequence::new()
    .then(fade_in, 0.5)
    .then(slide_up, 0.8)
    .build();

let outro = Sequence::new()
    .then(slide_down, 0.8)
    .then(fade_out, 0.5)
    .build();

let mut master = Sequence::new()
    .then(intro, 1.3)      // intro plays for 1.3s total
    .gap(2.0)              // 2 second pause
    .then(outro, 1.3)      // outro plays for 1.3s total
    .build();

master.play();
```

---

## Edge Cases

| Scenario | Behaviour |
|----------|-----------|
| Empty timeline | `update()` returns `true` (no entries to complete) |
| `.play()` never called | `update()` is a no-op, state stays `Idle` |
| Very large `dt` | Entries complete without issues, no overflow |
| Entry `start_at` in the past | Entry starts immediately on next `update()` |

> **Note**: `Timeline` requires heap allocation (`Box<dyn Update>`) and thus needs either `std` or `alloc`.
