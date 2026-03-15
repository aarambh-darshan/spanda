# spanda — Full Project Architecture & Build Guide

> *Sanskrit: स्पन्द — vibration, pulse, the throb of motion.*
>
> A general-purpose animation library for Rust covering tweening, keyframe
> animations, timelines, and physics-based motion.  Zero mandatory
> dependencies.  Works on TUI, Web (WASM), Bevy, and native targets.

---

## Table of Contents

1. [Project Vision](#1-project-vision)
2. [Crate Structure](#2-crate-structure)
3. [Module-by-Module Specification](#3-module-by-module-specification)
   - 3.1 [traits.rs](#31-traitsrs)
   - 3.2 [easing.rs](#32-easingrs)
   - 3.3 [tween.rs](#33-tweenrs)
   - 3.4 [keyframe.rs](#34-keyframers)
   - 3.5 [timeline.rs](#35-timeliners)
   - 3.6 [spring.rs](#36-springrs)
   - 3.7 [driver.rs](#37-driverrs)
   - 3.8 [clock.rs](#38-clockrs)
   - 3.9 [lib.rs](#39-librs)
4. [Cargo.toml & Feature Flags](#4-cargotoml--feature-flags)
5. [Data Flow & Runtime Loop](#5-data-flow--runtime-loop)
6. [Type System Design](#6-type-system-design)
7. [Integration Targets](#7-integration-targets)
   - 7.1 [TUI / CLI (ratatui)](#71-tui--cli-ratatui)
   - 7.2 [Web / WASM](#72-web--wasm)
   - 7.3 [Bevy Plugin](#73-bevy-plugin)
   - 7.4 [no\_std / Embedded](#74-no_std--embedded)
8. [API Design Reference](#8-api-design-reference)
9. [Error Handling Strategy](#9-error-handling-strategy)
10. [Testing Strategy](#10-testing-strategy)
11. [Performance Considerations](#11-performance-considerations)
12. [Publishing Checklist (crates.io)](#12-publishing-checklist-cratesio)
13. [Suggested Build Order](#13-suggested-build-order)
14. [Naming Conventions](#14-naming-conventions)

---

## 1. Project Vision

`spanda` is built around a single idea: **any value that can be linearly
interpolated can be animated**.  Everything else — easing curves, keyframe
tracks, timelines, physics springs — is layered on top of that one primitive.

### Design Goals

| Goal | Decision |
|------|----------|
| Zero mandatory dependencies | Pure Rust math only in the core |
| Works in `no_std` environments | `#![cfg_attr(not(feature = "std"), no_std)]` |
| Composable, not monolithic | Each module is useful standalone |
| Ergonomic public API | Builder pattern everywhere, sensible defaults |
| Type-safe animation targets | Generic over `T: Animatable` |
| Testable without a real clock | `Clock` trait with a `MockClock` |
| Serialisable state | Optional `serde` feature, no forced dep |

### Non-Goals

- `spanda` does NOT render anything.  It computes values; the caller renders.
- `spanda` does NOT own a game loop.  It accepts a `dt` tick; the caller drives it.
- `spanda` does NOT manage scene graphs or entity hierarchies (use Bevy for that).

---

## 2. Crate Structure

```
spanda/
├── Cargo.toml
├── README.md
├── ARCHITECTURE.md          ← this file
├── CHANGELOG.md
├── LICENSE-MIT
├── LICENSE-APACHE
├── examples/
│   ├── tui_spinner.rs       ← ratatui demo
│   ├── tui_progress.rs
│   ├── spring_demo.rs       ← terminal spring simulation
│   ├── wasm_tween/          ← wasm-pack project
│   │   ├── src/lib.rs
│   │   └── www/index.html
│   └── bevy_bounce.rs       ← bevy example
├── benches/
│   └── easing_bench.rs      ← criterion benchmarks
└── src/
    ├── lib.rs               ← crate root, re-exports
    ├── traits.rs            ← Interpolate, Animatable, Update
    ├── easing.rs            ← Easing enum + 31 pure functions
    ├── tween.rs             ← Tween<T> struct, looping, time scale, callbacks
    ├── keyframe.rs          ← KeyframeTrack<T>
    ├── timeline.rs          ← Timeline, Sequence, At, stagger
    ├── spring.rs        — Spring, SpringConfig, SpringN<T>, SpringAnimatable
    ├── driver.rs            ← AnimationDriver (manages active animations)
    ├── clock.rs             ← Clock trait, WallClock, MockClock
    ├── driver.rs            ← AnimationDriver (manages active animations)
    ├── scroll.rs            ← ScrollClock, ScrollDriver (scroll-linked animation)
    ├── path.rs              ← BezierPath, MotionPath, MotionPathTween
    ├── bezier.rs            ← CatmullRomSpline, PathEvaluate2D (tangent, auto-rotate)
    ├── motion_path.rs       ← PolyPath, CompoundPath, PathCommand (arc-length param)
    ├── svg_path.rs          ← SvgPathParser (SVG d-attribute parser)
    ├── colour.rs            ← colour interpolation (feature = "palette")
    └── integrations/
        ├── mod.rs
        ├── bevy.rs          ← SpandaPlugin (feature = "bevy")
        └── wasm.rs          ← RafDriver (feature = "wasm")
```

---

## 3. Module-by-Module Specification

---

### 3.1 `traits.rs`

**Status: complete** (already written)

This is the foundation.  Nothing in the crate can be built without these traits.

#### Traits defined

```rust
// The only thing a type needs to implement manually:
pub trait Interpolate: Sized {
    fn lerp(&self, other: &Self, t: f32) -> Self;
}

// Auto-derived via blanket impl — never impl manually:
pub trait Animatable: Interpolate + Clone + 'static {}
impl<T: Interpolate + Clone + 'static> Animatable for T {}

// Implemented by Tween, Timeline, Spring — the driver calls this:
pub trait Update {
    fn update(&mut self, dt: f32) -> bool;  // returns false when done
}
```

#### Blanket `Interpolate` implementations to ship

| Type | Notes |
|------|-------|
| `f32` | Core scalar |
| `f64` | High-precision, `t` cast to `f64` internally |
| `[f32; 2]` | 2-D position / size |
| `[f32; 3]` | 3-D position / RGB colour |
| `[f32; 4]` | RGBA colour / quaternion components |
| `i32` | Rounds to nearest after lerp |
| `u8` | Useful for byte-level colour channels |

#### Future blanket impls (when features are enabled)

| Feature | Type | Notes |
|---------|------|-------|
| `palette` | `palette::Srgba` | Proper gamma-correct colour lerp |
| `bevy` | `bevy::math::Vec2`, `Vec3`, `Vec4`, `Quat` | Delegate to bevy's lerp |

---

### 3.2 `easing.rs`

**Status: complete** (already written)

31 easing functions exposed both as:
- `Easing` enum with `.apply(t: f32) -> f32` — pass-around, storable, optionally serialisable
- Free `pub fn ease_out_cubic(t: f32) -> f32` — zero-overhead direct calls

#### Full easing reference

| Group | Variants | Character |
|-------|----------|-----------|
| Linear | `Linear` | Constant velocity |
| Polynomial | `EaseIn/Out/InOut` × Quad, Cubic, Quart, Quint | Smooth curves, increasing sharpness |
| Sinusoidal | `EaseIn/Out/InOutSine` | Gentle, natural |
| Exponential | `EaseIn/Out/InOutExpo` | Very sharp acceleration/deceleration |
| Circular | `EaseIn/Out/InOutCirc` | Arc-shaped curves |
| Back | `EaseIn/Out/InOutBack` | Overshoot — playful |
| Elastic | `EaseIn/Out/InOutElastic` | Spring-like oscillation |
| Bounce | `EaseIn/Out/InOutBounce` | Ball bouncing to rest |
| Custom | `Custom(fn(f32) -> f32)` | Arbitrary user curve |

#### Key implementation notes

- All functions clamp `t` to `[0.0, 1.0]` before evaluation.
- `apply(0.0)` must always return `0.0`, `apply(1.0)` must always return `1.0`.
  This is verified in the test suite for every named variant.
- `Easing::all_named()` returns a `&'static [Easing]` — useful for building
  picker UIs or running test sweeps.
- `Custom` variant is `serde(skip)` — function pointers are not serialisable.
  Store the name as a string separately if you need to round-trip it.

---

### 3.3 `tween.rs`

**Status: not yet written — build this second**

A `Tween<T>` animates a single value from `start` to `end` over a `duration`
in seconds, applying an easing curve.

#### Struct definition

```rust
pub struct Tween<T: Animatable> {
    pub start:    T,
    pub end:      T,
    pub duration: f32,        // seconds
    pub easing:   Easing,
    pub delay:    f32,        // seconds before animation starts
    elapsed:      f32,        // private, managed by Update::update()
    state:        TweenState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TweenState {
    Waiting,     // inside delay period
    Running,
    Completed,
    Paused,
}
```

#### Builder pattern

```rust
// Desired usage:
let tween = Tween::new(0.0_f32, 100.0_f32)
    .duration(1.5)
    .easing(Easing::EaseOutCubic)
    .delay(0.2)
    .build();
```

#### Key methods

```rust
impl<T: Animatable> Tween<T> {
    pub fn new(start: T, end: T) -> TweenBuilder<T>;
    pub fn value(&self) -> T;          // current interpolated value
    pub fn progress(&self) -> f32;     // 0.0..=1.0 raw (before easing)
    pub fn is_complete(&self) -> bool;
    pub fn reset(&mut self);
    pub fn seek(&mut self, t: f32);    // jump to progress t
    pub fn reverse(&mut self);         // swap start/end, reset
}

impl<T: Animatable> Update for Tween<T> {
    fn update(&mut self, dt: f32) -> bool {
        // 1. If in delay: drain delay, return true
        // 2. Advance elapsed by dt
        // 3. Clamp elapsed to duration
        // 4. Set state = Completed if elapsed >= duration
        // 5. Return !is_complete()
    }
}
```

#### Value computation

```rust
pub fn value(&self) -> T {
    let raw_t    = (self.elapsed / self.duration).clamp(0.0, 1.0);
    let curved_t = self.easing.apply(raw_t);
    self.start.lerp(&self.end, curved_t)
}
```

#### Tests to write for `tween.rs`

- `tween_starts_at_start_value()`
- `tween_ends_at_end_value()`
- `tween_is_complete_after_full_duration()`
- `tween_delay_is_respected()`
- `tween_reverse_swaps_values()`
- `tween_seek_jumps_to_correct_value()`
- `tween_does_not_overshoot_on_large_dt()`

---

### 3.4 `keyframe.rs`

**Status: not yet written — build this third**

A `KeyframeTrack<T>` holds a series of `(time, value)` pairs and a per-segment
easing.  At any time `t`, it interpolates between the two surrounding keyframes.

#### Core types

```rust
#[derive(Clone)]
pub struct Keyframe<T: Animatable> {
    pub time:   f32,      // seconds from track start
    pub value:  T,
    pub easing: Easing,   // easing used from THIS frame to the NEXT
}

pub struct KeyframeTrack<T: Animatable> {
    frames:   Vec<Keyframe<T>>,   // must be kept sorted by time
    elapsed:  f32,
    looping:  Loop,
}

pub enum Loop {
    Once,
    Times(u32),
    Forever,
    PingPong,             // plays forward then backward repeatedly
}
```

#### Key methods

```rust
impl<T: Animatable> KeyframeTrack<T> {
    pub fn new() -> Self;
    pub fn push(mut self, time: f32, value: T) -> Self;
    pub fn push_with_easing(mut self, time: f32, value: T, easing: Easing) -> Self;
    pub fn looping(mut self, mode: Loop) -> Self;

    pub fn value_at(&self, t: f32) -> T;    // evaluate at any time, pure
    pub fn value(&self) -> T;               // current value based on elapsed
    pub fn duration(&self) -> f32;          // time of last keyframe
}
```

#### Interpolation algorithm

```
Given time t:
1. Binary search frames for the last frame where frame.time <= t.
2. If t >= last frame time → return last frame value (clamped).
3. Compute local_t = (t - frame[i].time) / (frame[i+1].time - frame[i].time)
4. Apply frame[i].easing to local_t → curved_t
5. Return frame[i].value.lerp(&frame[i+1].value, curved_t)
```

#### PingPong loop logic

```
total_duration = duration()
cycle_t = elapsed % (2.0 * total_duration)
if cycle_t <= total_duration:
    t = cycle_t                            // forward
else:
    t = 2.0 * total_duration - cycle_t    // backward
```

---

### 3.5 `timeline.rs`

**Status: not yet written — build this fourth**

A `Timeline` is a collection of labelled animations that play concurrently or
in sequence, with per-entry delays.  Think of it as a mini animation mixer.

#### Core types

```rust
pub struct Timeline {
    entries:  Vec<TimelineEntry>,
    elapsed:  f32,
    state:    TimelineState,
    looping:  Loop,
}

struct TimelineEntry {
    label:     String,
    animation: Box<dyn Update>,
    start_at:  f32,              // seconds offset from timeline start
    duration:  f32,              // how long this entry runs
}

pub enum TimelineState {
    Idle,
    Playing,
    Paused,
    Completed,
}
```

#### Playback control

```rust
impl Timeline {
    pub fn play(&mut self);
    pub fn pause(&mut self);
    pub fn resume(&mut self);
    pub fn seek(&mut self, t: f32);
    pub fn reset(&mut self);
    pub fn reverse(&mut self);

    pub fn duration(&self) -> f32;   // end time of the last entry
    pub fn progress(&self) -> f32;   // 0.0..=1.0
}
```

#### Builder pattern

```rust
// Desired usage — concurrent animations:
let timeline = Timeline::new()
    .add("fade_in",  fade_tween,   at: 0.0)
    .add("slide_up", slide_tween,  at: 0.0)
    .add("pop",      scale_tween,  at: 0.4)
    .looping(Loop::Once);

// Sequence helper — each animation starts when the previous ends:
let sequence = Sequence::new()
    .then(move_tween)
    .then(fade_tween)
    .gap(0.1)   // 100ms pause between each step
    .then(scale_tween);
```

#### `Sequence` is sugar over `Timeline`

`Sequence` auto-calculates `start_at` values by accumulating durations.
Internally it builds and returns a `Timeline`.

#### Callback system (with `std` feature)

```rust
// Callbacks fire when a labelled entry completes:
timeline.on_complete("fade_in", || println!("fade done"));

// Or on the whole timeline:
timeline.on_finish(|| println!("all done"));
```

Callbacks are stored as `Box<dyn FnMut()>` and only available with `std`
(because closures with heap allocation need the allocator).

#### `tokio` async feature

```rust
// With feature = "tokio":
timeline.play();
timeline.wait().await;  // resolves when timeline completes
```

Internally uses a `tokio::sync::watch::Sender<TimelineState>`.

---

### 3.6 `spring.rs`

**Status: not yet written — build this fifth**

Physics-based animation using a damped harmonic oscillator.  Unlike easing
functions, a spring has no fixed duration — it settles when velocity and
displacement are below a threshold.

#### Core types

```rust
pub struct Spring {
    pub config:   SpringConfig,
    position:     f32,
    velocity:     f32,
    target:       f32,
}

#[derive(Clone, Debug)]
pub struct SpringConfig {
    pub stiffness: f32,   // "tightness" — higher = faster (default: 100.0)
    pub damping:   f32,   // resistance — higher = less bounce (default: 10.0)
    pub mass:      f32,   // inertia — higher = slower start (default: 1.0)
    pub epsilon:   f32,   // settle threshold (default: 0.001)
}
```

#### Presets

```rust
impl SpringConfig {
    pub fn gentle()   -> Self; // stiffness: 60,  damping: 14
    pub fn wobbly()   -> Self; // stiffness: 180, damping: 12
    pub fn stiff()    -> Self; // stiffness: 210, damping: 20
    pub fn slow()     -> Self; // stiffness: 37,  damping: 14
}
```

#### Integration algorithm: Semi-implicit Euler (RK4 optional)

For a spring system, the equation of motion is:

```
a = (-stiffness * displacement - damping * velocity) / mass
```

Semi-implicit Euler is sufficient for animation (stable, cheap):

```rust
fn step(&mut self, dt: f32) {
    let displacement = self.position - self.target;
    let acceleration  = (-self.config.stiffness * displacement
                         - self.config.damping * self.velocity)
                        / self.config.mass;
    self.velocity += acceleration * dt;
    self.position += self.velocity * dt;
}
```

RK4 is more accurate for high-stiffness springs but 4× more expensive per
step.  Add it behind a method flag: `spring.use_rk4(true)`.

#### Settle detection

```rust
fn is_settled(&self) -> bool {
    let displacement = (self.position - self.target).abs();
    let velocity     = self.velocity.abs();
    displacement < self.config.epsilon && velocity < self.config.epsilon
}
```

#### Generic `Spring<T: Animatable>`

Extend to generic types by maintaining a `Vec<f32>` of components internally
and applying the spring equation per-component, then reassembling into `T`.

```rust
pub struct SpringN<T: Animatable> {
    components: Vec<Spring>,   // one per interpolatable dimension
    _marker:    PhantomData<T>,
}
```

---

### 3.7 `driver.rs`

**Status: not yet written — build this sixth**

The `AnimationDriver` owns a collection of active animations and ticks them
all on each frame.  It handles cleanup of completed animations automatically.

#### Core type

```rust
pub struct AnimationDriver {
    animations: Vec<(AnimationId, Box<dyn Update>)>,
    next_id:    u64,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct AnimationId(u64);
```

#### API

```rust
impl AnimationDriver {
    pub fn new() -> Self;

    // Add an animation, get back an ID to cancel it later:
    pub fn add<A: Update + 'static>(&mut self, animation: A) -> AnimationId;

    // Tick all active animations forward by dt seconds.
    // Automatically removes completed animations.
    pub fn tick(&mut self, dt: f32);

    pub fn cancel(&mut self, id: AnimationId);
    pub fn cancel_all(&mut self);
    pub fn active_count(&self) -> usize;
}
```

#### Thread-safe variant (with `std` feature)

```rust
// AnimationDriverArc wraps the driver in Arc<Mutex<>> for sharing
// across threads (e.g. audio thread + render thread).
pub struct AnimationDriverArc(Arc<Mutex<AnimationDriver>>);
```

---

### 3.8 `clock.rs`

**Status: not yet written — build with driver**

The `Clock` trait decouples the animation system from wall time, making the
entire crate deterministically testable.

#### Trait

```rust
pub trait Clock {
    /// Returns seconds elapsed since the last call to `delta()`.
    fn delta(&mut self) -> f32;
}
```

#### Implementations

```rust
// Wall clock — uses std::time::Instant (requires "std" feature):
pub struct WallClock {
    last: std::time::Instant,
}

// Manual clock — caller provides dt, useful for game engines:
pub struct ManualClock {
    pending_dt: f32,
}
impl ManualClock {
    pub fn advance(&mut self, dt: f32) { self.pending_dt += dt; }
}

// Mock clock — steps in fixed increments, perfect for tests:
pub struct MockClock {
    step: f32,
}
impl MockClock {
    pub fn new(step_seconds: f32) -> Self;
}
```

---

### 3.9 `lib.rs`

**Status: complete** (already written)

The crate root re-exports everything the user needs:

```rust
pub use easing::Easing;
pub use traits::{Animatable, Interpolate, Update};
pub use tween::{Tween, TweenState, snap_to, round_to};
pub use keyframe::{KeyframeTrack, Keyframe, Loop};
pub use timeline::{Timeline, Sequence, At, stagger};
pub use spring::{Spring, SpringConfig, SpringN, SpringAnimatable};
pub use driver::{AnimationDriver, AnimationId};
pub use clock::{Clock, WallClock, ManualClock, MockClock};
pub use scroll::{ScrollClock, ScrollDriver};
pub use path::{BezierPath, MotionPath, MotionPathTween, PathEvaluate};
pub use bezier::{CatmullRomSpline, PathEvaluate2D, tangent_angle, tangent_angle_deg};
pub use motion_path::{PolyPath, CompoundPath, PathCommand};
pub use svg_path::SvgPathParser;
```

---

## 4. Cargo.toml & Feature Flags

```toml
[package]
name        = "spanda"
version     = "0.1.0"
edition     = "2021"
description = "A general-purpose animation library for Rust — tweening, keyframes, timelines, and physics."
license     = "MIT OR Apache-2.0"
repository  = "https://github.com/aarambh-darshan/spanda"
keywords    = ["animation", "tween", "easing", "keyframe", "gamedev"]
categories  = ["game-development", "graphics", "mathematics", "multimedia"]
readme      = "README.md"
rust-version = "1.70"

[features]
default  = ["std"]
std      = []
serde    = ["dep:serde"]
bevy     = ["dep:bevy_app", "dep:bevy_ecs", "dep:bevy_time", "std"]
wasm     = ["dep:wasm-bindgen", "dep:js-sys", "std"]
palette  = ["dep:palette"]
tokio    = ["dep:tokio", "std"]

[dependencies]
serde        = { version = "1",    features = ["derive"], optional = true }
bevy_app     = { version = "0.13", optional = true }
bevy_ecs     = { version = "0.13", optional = true }
bevy_time    = { version = "0.13", optional = true }
wasm-bindgen = { version = "0.2",  optional = true }
js-sys       = { version = "0.3",  optional = true }
palette      = { version = "0.7",  optional = true }
tokio        = { version = "1",    features = ["sync"], optional = true }

[dev-dependencies]
approx    = "0.5"
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name    = "easing_bench"
harness = false
```

### Feature flag decision guide

| You are building... | Recommended features |
|---------------------|----------------------|
| A TUI app | `default` (just `std`) |
| A Bevy game | `bevy` |
| A WASM web app | `wasm`, no `std` |
| A CLI tool | `default` |
| Embedded / `no_std` | disable default: `default-features = false` |
| Full everything | `std,serde,bevy,wasm,palette,tokio` |

---

## 5. Data Flow & Runtime Loop

### Standard (non-Bevy, non-WASM) loop

```
Application loop
      │
      ▼
 WallClock::delta()   ──► dt: f32 (seconds since last frame)
      │
      ▼
 AnimationDriver::tick(dt)
      │
      ├── Tween::update(dt)         ─► advance elapsed, compute value()
      ├── KeyframeTrack::update(dt) ─► advance elapsed, find segment, lerp
      ├── Timeline::update(dt)      ─► tick all entries, fire callbacks
      └── Spring::update(dt)        ─► integrate velocity + position
      │
      ▼
 Application reads value()
 from each animation and
 renders / applies it.
```

### Bevy loop (with `bevy` feature)

```
Bevy scheduler
      │
      ▼
 SpandaPlugin registers system: spanda_tick_system
      │
      ▼
 Query<&mut Tween<T>, &mut Spring, …>
      │
      ▼
 Calls .update(time.delta_seconds()) on each component
      │
      ▼
 Bevy renders updated component values on the next frame
```

### WASM loop (with `wasm` feature)

```
Browser
      │
      ▼
 requestAnimationFrame callback
      │
      ▼
 RafDriver::tick(timestamp_ms)
      │
      ▼
 Calls AnimationDriver::tick(dt)
      │
      ▼
 Writes output values to JS via wasm-bindgen closures
```

---

## 6. Type System Design

### The `Animatable` hierarchy

```
Interpolate
    │  lerp(&self, other: &Self, t: f32) -> Self
    │
    └── Animatable  (auto blanket impl: Interpolate + Clone + 'static)
            │
            └── Used as the generic bound on:
                  Tween<T: Animatable>
                  KeyframeTrack<T: Animatable>
                  SpringN<T: Animatable>
```

### Why `t: f32` (not generic)?

The entire crate uses `f32` for the progress parameter `t`.  This is an
intentional design decision:

- Animation timing is inherently a display-frequency concern.  Sub-millisecond
  precision beyond `f32` is not perceptible.
- Using a single concrete type avoids a second generic parameter `<T, P>`
  exploding the API surface.
- `f64` values (like world coordinates in a simulation) still get full `f64`
  math internally — only the `t` input is cast to `f64` in the blanket impl.

### Builder pattern everywhere

Every complex type uses the builder pattern:

```rust
// Bad (many constructor arguments, hard to read):
let t = Tween::new(start, end, 1.0, Easing::EaseOutCubic, 0.2, Loop::Once);

// Good (self-documenting, optional params have defaults):
let t = Tween::new(start, end)
    .duration(1.0)
    .easing(Easing::EaseOutCubic)
    .delay(0.2)
    .build();
```

Builders should take `self` (consuming) so they chain cleanly.

### `no_std` strategy

All heap allocation is gated behind the `std` feature or an explicit `alloc`
import.  When `no_std` is active:

- `Vec` requires `extern crate alloc; use alloc::vec::Vec;`
- Closures / `Box<dyn Fn>` are unavailable — callbacks are disabled
- `String` becomes `&'static str` for labels
- `AnimationDriver` is unavailable (requires `Vec`)
- `Tween<T>`, `Spring`, `Easing` all work — they are stack-allocated

---

## 7. Integration Targets

---

### 7.1 TUI / CLI (ratatui)

`ratatui` renders at ~60 fps in the terminal.  The pattern is:

```rust
use spanda::{Tween, Easing};

struct App {
    progress_tween: Tween<f32>,
}

fn main() {
    let mut app = App {
        progress_tween: Tween::new(0.0_f32, 1.0)
            .duration(2.0)
            .easing(Easing::EaseInOutCubic)
            .build(),
    };

    let mut clock = spanda::WallClock::new();

    loop {
        let dt = clock.delta();
        app.progress_tween.update(dt);

        // Pass app.progress_tween.value() to ratatui's Gauge widget
        terminal.draw(|f| ui(f, &app))?;

        if app.progress_tween.is_complete() { break; }
    }
}
```

**Key examples to ship:**

- `examples/tui_progress.rs` — animated progress bar using `Gauge`
- `examples/tui_spinner.rs` — rotating braille spinner using `KeyframeTrack<char>`
- `examples/tui_bounce.rs` — element bouncing around the terminal using `Spring`

---

### 7.2 Web / WASM

Build with `wasm-pack`.  The `wasm` feature wires up a `requestAnimationFrame`
loop automatically.

```rust
// src/lib.rs (wasm example)
use wasm_bindgen::prelude::*;
use spanda::{Tween, Easing};

#[wasm_bindgen]
pub struct App {
    tween: Tween<f32>,
    driver: spanda::wasm::RafDriver,
}

#[wasm_bindgen]
impl App {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let tween = Tween::new(0.0_f32, 500.0)
            .duration(1.5)
            .easing(Easing::EaseOutBounce)
            .build();
        Self { tween, driver: spanda::wasm::RafDriver::new() }
    }

    pub fn value(&self) -> f32 {
        self.tween.value()
    }
}
```

**Build command:**

```bash
wasm-pack build --target web --features wasm
```

---

### 7.3 Bevy Plugin

```rust
// In user's Bevy app:
use spanda::integrations::bevy::SpandaPlugin;

app.add_plugins(SpandaPlugin);
```

The plugin registers:

- `Tween<T>` as a `Component` for all `T: Animatable + Reflect`
- `SpringN<T>` as a `Component`
- `spanda_tick_system` which runs in `Update` and ticks all components

```rust
// Usage in Bevy:
commands.spawn((
    SpriteBundle { .. },
    Tween::new([0.0_f32, 0.0], [200.0, 0.0])
        .duration(0.8)
        .easing(Easing::EaseOutBack)
        .build(),
));
```

A `TweenCompleted` event is fired when any `Tween` component finishes.

---

### 7.4 `no_std` / Embedded

```toml
# In downstream Cargo.toml:
[dependencies]
spanda = { version = "0.1", default-features = false }
```

Available in `no_std`: `Easing`, `Tween<T>`, `Spring`, `KeyframeTrack<T>` (if
`alloc` is available), all `Interpolate` blanket impls.

Not available in `no_std`: `AnimationDriver`, `WallClock`, callbacks,
`SpandaPlugin`, `RafDriver`.

---

## 8. API Design Reference

### The five most common patterns users will write

**1. One-shot tween (most common)**

```rust
let mut t = Tween::new(0.0_f32, 1.0)
    .duration(0.4)
    .easing(Easing::EaseOutCubic)
    .build();

// In loop:
t.update(dt);
let opacity = t.value();
```

**2. Looping keyframe animation**

```rust
let mut track = KeyframeTrack::new()
    .push(0.0, 0.0_f32)
    .push(0.5, 1.0)
    .push(1.0, 0.0)
    .looping(Loop::Forever);

// In loop:
track.update(dt);
let alpha = track.value();
```

**3. Sequenced timeline**

```rust
let mut seq = Sequence::new()
    .then(Tween::new(0.0_f32, 100.0).duration(0.3).easing(Easing::EaseOutQuad).build())
    .gap(0.1)
    .then(Tween::new(1.0_f32, 0.0).duration(0.2).easing(Easing::EaseInQuad).build());

seq.play();
// In loop:
seq.update(dt);
```

**4. Spring to a target**

```rust
let mut spring = Spring::new(SpringConfig::wobbly());
spring.set_target(200.0);

// In loop:
spring.update(dt);
let x = spring.position();
```

**5. Driver managing many animations**

```rust
let mut driver = AnimationDriver::new();

let id = driver.add(
    Tween::new(0.0_f32, 1.0).duration(1.0).build()
);

// In loop:
driver.tick(dt);

// Cancel early if needed:
driver.cancel(id);
```

---

## 9. Error Handling Strategy

`spanda` uses **no `Result` in hot paths**.  Animation update functions never
fail — they clamp, saturate, or silently correct invalid input rather than
returning errors.

| Situation | Behaviour |
|-----------|-----------|
| `t` outside `[0, 1]` passed to easing | Clamped silently |
| `duration = 0.0` in tween | Immediately complete (returns `end` value) |
| `KeyframeTrack` with 0 or 1 frames | Returns the single value or `T::default()` |
| `dt < 0.0` | Treated as `0.0` (no backward time) |
| Spring with `stiffness = 0.0` | Returns target immediately |
| `seek()` with `t > duration` | Clamped to end |

The only `Result`-returning APIs are constructors that validate user-provided
data at build time (e.g. `TweenBuilder::build()` could return
`Result<Tween<T>, SpandaError>` if you want to catch `duration < 0`).

---

## 10. Testing Strategy

### Unit tests (in each `rs` file)

Every module has a `#[cfg(test)]` block at the bottom.  Required tests per
module:

| Module | Key tests |
|--------|-----------|
| `traits.rs` | `f32` lerp endpoints and midpoint, `[f32;4]` channel independence, `Animatable` auto-impl |
| `easing.rs` | All 31 variants: `apply(0)=0`, `apply(1)=1`, no panic on out-of-range |
| `tween.rs` | Start/end values, complete flag, delay, seek, reverse, large-dt safety |
| `keyframe.rs` | Single frame, two frames, looping, ping-pong, out-of-bounds query |
| `timeline.rs` | Sequential play, concurrent play, seek, pause/resume, loop |
| `spring.rs` | Settles to target, stiff spring settles fast, damping=0 oscillates |
| `driver.rs` | Completed animations are removed, cancel works, `active_count` |
| `clock.rs` | `MockClock` returns correct fixed dt, `WallClock` is positive |

### Integration tests (`tests/` directory)

```
tests/
├── tween_with_easing.rs      — full tween lifecycle using MockClock
├── keyframe_looping.rs       — long-running looping track
├── spring_settles.rs         — spring reaches target within N steps
└── timeline_sequence.rs      — multi-step sequence completes in order
```

### Benchmark (`benches/easing_bench.rs`)

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use spanda::easing::*;

fn bench_easings(c: &mut Criterion) {
    c.bench_function("ease_out_elastic", |b| {
        b.iter(|| ease_out_elastic(black_box(0.5)))
    });
}

criterion_group!(benches, bench_easings);
criterion_main!(benches);
```

Run with: `cargo bench`

---

## 11. Performance Considerations

### Zero-cost in the common case

- All easing functions are `#[inline]` — the compiler inlines them at call site.
- `Tween<T>` is a stack-allocated struct, no heap allocation.
- `Interpolate` blanket impls on primitives compile to 2–3 float multiplications.
- `Easing::apply()` is a match on a local enum — branch predictor handles it well.

### When allocation is needed

- `KeyframeTrack<T>` holds a `Vec<Keyframe<T>>` — one allocation at build time, zero during update.
- `Timeline` holds a `Vec<TimelineEntry>` — same pattern.
- `AnimationDriver` holds a `Vec<Box<dyn Update>>` — dynamic dispatch, one allocation per `add()`.

### Avoiding dynamic dispatch in hot loops

If you are animating thousands of values per frame (particle systems), avoid
`Box<dyn Update>` in `AnimationDriver`.  Instead, keep a `Vec<Tween<f32>>`
directly and call `.update()` on each element — the compiler monomorphises the
call and can auto-vectorise.

### `no_std` binary size

With `default-features = false`, the entire `easing.rs` + `traits.rs` +
`tween.rs` stack compiles to approximately 3–8 KB of `.text` depending on which
easing variants are used (link-time dead code elimination removes unused variants).

---

## 12. Publishing Checklist (crates.io)

Before running `cargo publish`:

- [ ] All public items have `///` doc comments
- [ ] `README.md` has a quick-start example that compiles
- [ ] `CHANGELOG.md` has a `## [0.1.0]` entry
- [ ] `LICENSE-MIT` and `LICENSE-APACHE` are present
- [ ] `cargo test` passes with no warnings
- [ ] `cargo test --no-default-features` passes (no_std check)
- [ ] `cargo test --all-features` passes
- [ ] `cargo clippy --all-features -- -D warnings` is clean
- [ ] `cargo doc --all-features --open` renders correctly
- [ ] `cargo bench` runs without error
- [ ] Version in `Cargo.toml` matches git tag
- [ ] `cargo publish --dry-run` succeeds

### Semantic versioning plan

| Version | Milestone |
|---------|-----------|
| `0.1.0` | Core complete — tweening, keyframes, timelines, springs, driver, clock |
| `0.2.0` | Ergonomics — stagger, looping, time scale, callbacks, value modifiers |
| `0.3.0` | Scroll & motion paths — ScrollDriver, At positioning, Bezier paths, MotionPath |
| `0.4.0` | Full motion path system — CatmullRom, PolyPath, CompoundPath, SvgPathParser, CSS easing |
| `0.5.0` | `spring` generics & Bevy polish — SpringN<T>, SpringSettled event, AnimationLabel |
| `0.6.0` | `wasm` & web polish — RafDriver enhancements, start_raf_loop, Leptos/Dioxus guides |
| `0.7.0` | Colour & advanced interpolation — 9 palette types, InLab/InOklch/InLinear, SpringAnimatable |
| `1.0.0` | Stable API, full docs, all examples |

---

## 13. Suggested Build Order

Build in this exact order — each step depends only on what came before.

```
Step 1  ──  traits.rs          DONE ✓
            Easing enum + pure fns

Step 2  ──  easing.rs          DONE ✓
            Blanket Interpolate impls

Step 3  ──  tween.rs
            Tween<T> struct, TweenBuilder, Update impl
            → First usable animation!

Step 4  ──  clock.rs
            Clock trait, WallClock, MockClock
            → Makes tween testable against real time

Step 5  ──  driver.rs
            AnimationDriver, AnimationId
            → Manage multiple tweens at once

Step 6  ──  keyframe.rs
            Keyframe<T>, KeyframeTrack<T>, Loop enum
            → Multi-step animations

Step 7  ──  timeline.rs
            Timeline, Sequence
            → Compose everything together

Step 8  ──  spring.rs
            Spring, SpringConfig, presets
            → Physics-based motion

Step 9  ──  integrations/bevy.rs
            SpandaPlugin, Component impls
            → Bevy users

Step 10 ──  integrations/wasm.rs
            RafDriver, wasm-bindgen wiring
            → Web users

Step 11 ──  examples/
            TUI demos, Bevy demo, WASM demo
            → Proof it works + YouTube content

Step 12 ──  benches/
            Criterion benchmarks
            → Performance story for blog post

Step 13 ──  Publish 0.1.0
```

---

## 14. Naming Conventions

### Crate name

`spanda` — Sanskrit *स्पन्द* (vibration, pulse).  Unique on crates.io at time
of writing.  Short, memorable, pronounceable.

### Module naming rationale

| Module | Why this name |
|--------|---------------|
| `traits.rs` | Rust convention for trait-only modules |
| `easing.rs` | Standard animation industry term |
| `tween.rs` | Standard animation industry term (from "in-between") |
| `keyframe.rs` | Standard animation industry term |
| `timeline.rs` | Standard animation industry term |
| `spring.rs` | Descriptive — physical model |
| `driver.rs` | Drives the animation system forward |
| `clock.rs` | Provides time to the driver |

### Type naming

| Type | Convention |
|------|-----------|
| `Tween<T>` | `PascalCase`, generic over animated value |
| `KeyframeTrack<T>` | Verbose but unambiguous |
| `SpringConfig` | Config struct = `{Type}Config` |
| `AnimationId` | Newtype over `u64` for type safety |
| `TweenState` | State enum = `{Type}State` |
| `Loop` | Short, used everywhere, not `LoopMode` |

### Public vs private fields

All timing internals (`elapsed`, `velocity`, `state`) are private.  All
configuration fields (`duration`, `stiffness`, `easing`) are `pub` so users
can inspect and mutate them directly without getters.

---

*Document version: 0.7 — covers planned scope through spanda 1.0.0*
*Project: Aarambh Dev Hub — github.com/aarambh-darshan/spanda*
