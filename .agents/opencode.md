# spanda — OpenCode Agent Skill

> Use this reference when helping a user integrate the **spanda** animation library into their Rust project. This file contains the complete public API, usage patterns, and examples.

---

## Quick Start

Add to `Cargo.toml`:
```toml
[dependencies]
spanda = "0.8"
```

Feature flags (add as needed):
```toml
spanda = { version = "0.8", features = ["serde", "palette"] }
```

---

## What is spanda?

**spanda** (v0.8.0) is a general-purpose animation library for Rust. It is a **pure value interpolation engine** — it computes animation values mathematically and never touches rendering, DOM, or any display target. You provide time (`dt`), spanda returns values. The caller is responsible for rendering.

- **Pure math, no side effects** — computes values; caller renders
- **`no_std` compatible** — core modules work without `std`; `alloc` needed for `Vec`-based types
- **Zero mandatory dependencies** — optional features add integrations
- **`#![forbid(unsafe_code)]`** — no unsafe code anywhere

---

## Feature Flags

| Flag | Implies | What it enables |
|------|---------|----------------|
| `std` *(default)* | — | `WallClock`, `AnimationDriverArc`, tween callbacks |
| `serde` | — | `Serialize`/`Deserialize` on public types |
| `bevy` | `std` | `SpandaPlugin` for Bevy 0.13 |
| `wasm` | `std` | `RafDriver` for `requestAnimationFrame` |
| `wasm-dom` | `wasm` | DOM plugins: FLIP, SplitText DOM, ScrollSmoother, Draggable, Observer |
| `palette` | — | Colour interpolation for 9 palette types |
| `tokio` | `std` | Async `.await` on timeline completion |

### Recommended Combinations

| Building... | Features |
|-------------|----------|
| TUI / CLI app | `default` (just `std`) |
| Bevy game | `bevy` |
| Leptos/Yew WASM app | `wasm` |
| WASM app with DOM interaction | `wasm-dom` |
| Embedded / `no_std` | `default-features = false` |
| Colour animations | `palette` |
| State persistence | `serde` |
| Async workflows | `tokio` |

---

## Conventions

- All durations are **`f32` seconds** (never milliseconds)
- Builder pattern: `Type::new(...).option().option().build()`
- `update(dt) -> bool` — returns `true` while running, `false` when complete
- `value()` / `position()` — read current state after `update()`
- Frame loop: call `update(dt)` each frame, use `value()` to render

---

## Core Traits

```rust
use spanda::{Interpolate, Animatable, Update};

/// Linear interpolation. Implement this to make any type animatable.
pub trait Interpolate: Sized {
    fn lerp(&self, other: &Self, t: f32) -> Self;
}

/// Blanket: Interpolate + Clone + 'static. No manual impl needed.
pub trait Animatable: Interpolate + Clone + 'static {}

/// Tick an animation forward by dt seconds. Returns false when done.
pub trait Update {
    fn update(&mut self, dt: f32) -> bool;
}
```

**Built-in `Interpolate` impls:** `f32`, `f64`, `i32`, `[f32; 2]`, `[f32; 3]`, `[f32; 4]`

### Making a Custom Type Animatable

```rust
use spanda::Interpolate;

#[derive(Clone)]
struct Color { r: f32, g: f32, b: f32 }

impl Interpolate for Color {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        Color {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
        }
    }
}
// Color is now Animatable (blanket impl) and can be used with Tween<Color>
```

---

## Easing

```rust
use spanda::Easing;
```

### Enum `Easing` — 38+ variants

**31 standard curves:**
`Linear`, `EaseInQuad`, `EaseOutQuad`, `EaseInOutQuad`, `EaseInCubic`, `EaseOutCubic`, `EaseInOutCubic`, `EaseInQuart`, `EaseOutQuart`, `EaseInOutQuart`, `EaseInQuint`, `EaseOutQuint`, `EaseInOutQuint`, `EaseInSine`, `EaseOutSine`, `EaseInOutSine`, `EaseInExpo`, `EaseOutExpo`, `EaseInOutExpo`, `EaseInCirc`, `EaseOutCirc`, `EaseInOutCirc`, `EaseInBack`, `EaseOutBack`, `EaseInOutBack`, `EaseInElastic`, `EaseOutElastic`, `EaseInOutElastic`, `EaseInBounce`, `EaseOutBounce`, `EaseInOutBounce`

**Escape hatch:**
`Custom(fn(f32) -> f32)` — any pure function

**CSS-compatible:**
`CubicBezier(x1, y1, x2, y2)` — CSS `cubic-bezier()` equivalent
`Steps(n)` — CSS `steps()` equivalent

**Advanced parametric (v0.8):**
```rust
Easing::RoughEase { strength: f32, points: u32, seed: u32 }  // hand-drawn noise
Easing::SlowMo { ratio: f32, power: f32, yoyo_mode: bool }   // slow-fast-slow
Easing::ExpoScale { start_scale: f32, end_scale: f32 }       // perceptual zoom
Easing::Wiggle { frequency: f32, amplitude: f32 }             // shake/vibration
Easing::CustomBounce { strength: f32, squash: f32 }           // parametric bounce
```

### Methods
```rust
impl Easing {
    pub fn apply(&self, t: f32) -> f32;       // evaluate at t ∈ [0, 1]
    pub fn name(&self) -> &'static str;       // human-readable name
    pub fn all_named() -> &'static [Easing];  // 31 standard variants only
}
```

### CSS Preset Equivalents
```rust
let ease     = Easing::CubicBezier(0.25, 0.1, 0.25, 1.0);
let ease_in  = Easing::CubicBezier(0.42, 0.0, 1.0, 1.0);
let ease_out = Easing::CubicBezier(0.0, 0.0, 0.58, 1.0);
let ease_io  = Easing::CubicBezier(0.42, 0.0, 0.58, 1.0);
```

---

## Tween

```rust
use spanda::{Tween, Update, Easing};
```

### Creating a Tween

```rust
let mut tween = Tween::new(0.0_f32, 100.0)
    .duration(1.0)              // seconds
    .easing(Easing::EaseOutCubic)
    .delay(0.5)                 // wait before starting
    .time_scale(1.0)            // speed multiplier
    .looping(Loop::Once)        // Once | Times(n) | Forever | PingPong
    .build();
```

Aliases: `Tween::from_to(a, b)`, `Tween::from(a, b)` — all return `TweenBuilder<T>`.

### Frame Loop

```rust
loop {
    let dt = 1.0 / 60.0; // from your clock
    let running = tween.update(dt);
    let current = tween.value();   // interpolated T
    // render with `current`
    if !running { break; }
}
```

### TweenBuilder Methods
```rust
.duration(f32)          // animation length in seconds (default: 1.0)
.easing(Easing)         // easing curve (default: Linear)
.delay(f32)             // pre-delay in seconds (default: 0.0)
.time_scale(f32)        // speed multiplier (default: 1.0)
.looping(Loop)          // loop mode (default: Once)
.build() -> Tween<T>    // consume builder
```

### Tween Control
```rust
tween.value() -> T              // current interpolated value
tween.progress() -> f32         // 0.0..=1.0
tween.is_complete() -> bool
tween.state() -> &TweenState    // Waiting | Running | Completed | Paused
tween.seek(0.5)                 // jump to progress
tween.reset()                   // start over
tween.reverse()                 // swap start/end
tween.pause()
tween.resume()
tween.set_time_scale(2.0)       // speed up
tween.time_scale() -> f32
tween.loop_mode() -> &Loop
```

### Callbacks (requires `feature = "std"`, excluded when `"bevy"` active)
```rust
tween.on_start(|| println!("started"));
tween.on_update(|val: f32| println!("val: {val}"));
tween.on_complete(|| println!("done"));
tween.set_modifier(|v| (v * 100.0).round() / 100.0); // post-process
```

### Utility Functions
```rust
use spanda::tween::{snap_to, round_to};
let snap = snap_to(10.0);   // returns impl Fn(f32) -> f32, snaps to grid
let round = round_to(2);    // returns impl Fn(f32) -> f32, rounds decimals
```

---

## Loop

```rust
use spanda::Loop;
```

```rust
pub enum Loop {
    Once,           // play once
    Times(u32),     // repeat n times
    Forever,        // infinite loop
    PingPong,       // forward then backward, forever
}
```

---

## Keyframes

```rust
use spanda::{KeyframeTrack, Keyframe, Easing, Update, Loop};
```

### Creating a Keyframe Track

```rust
let mut track = KeyframeTrack::new()
    .push(0.0, 0.0_f32)                                     // time, value
    .push(0.5, 100.0)                                       // ease = Linear (default)
    .push_with_easing(1.0, 50.0, Easing::EaseOutBounce)     // custom ease for this segment
    .looping(Loop::Once);
```

### Methods
```rust
track.update(dt) -> bool
track.value() -> T               // current interpolated value
track.value_at(0.75) -> T        // sample at any time
track.duration() -> f32           // total track duration
track.is_complete() -> bool
track.reset()
```

---

## Timeline

```rust
use spanda::{Timeline, Sequence, At, stagger, Update};
```

### Parallel Animations (Timeline)

```rust
let mut tl = Timeline::new()
    .add("fade", opacity_tween, 0.0)     // label, animation, start_time
    .add("slide", position_tween, 0.3)   // starts 0.3s later
    .looping(Loop::Once);

// Or add relatively:
tl.add_at("scale", scale_tween, 0.5, At::End);       // at end of timeline
tl.add_at("color", color_tween, 0.8, At::Label("fade")); // at label's start
tl.add_at("glow", glow_tween, 0.2, At::Offset(1.5));    // at absolute offset
```

### Sequential Animations (Sequence)

```rust
let mut tl = Sequence::new()
    .then(tween_a, 0.5)    // animation, duration
    .gap(0.2)              // pause
    .then(tween_b, 0.3)
    .looping(Loop::Once)
    .build();              // returns Timeline
```

### Stagger

```rust
let animations: Vec<(Tween<f32>, f32)> = items.iter()
    .map(|_| (Tween::new(0.0, 1.0).duration(0.3).build(), 0.3))
    .collect();

let mut tl = stagger(animations, 0.05); // 50ms delay between each
```

### Timeline Control
```rust
tl.update(dt) -> bool
tl.play()
tl.pause()
tl.resume()
tl.seek(0.5)               // jump to time
tl.reset()
tl.duration() -> f32
tl.progress() -> f32        // 0.0..=1.0
tl.state() -> &TimelineState // Idle | Playing | Paused | Completed
tl.set_time_scale(2.0)
tl.time_scale() -> f32

#[cfg(feature = "std")]
tl.on_finish(|| println!("timeline done"));
```

### At Enum
```rust
pub enum At<'a> {
    Start,              // beginning of timeline
    End,                // end of current timeline
    Label(&'a str),     // at the start of a labelled entry
    Offset(f32),        // absolute time offset
}
```

---

## Spring Physics

```rust
use spanda::{Spring, SpringConfig, SpringN, SpringAnimatable, Update};
```

### SpringConfig Presets
```rust
SpringConfig::default()  // stiffness: 100, damping: 10, mass: 1, epsilon: 0.001
SpringConfig::gentle()   // soft, slow
SpringConfig::wobbly()   // low damping, oscillates
SpringConfig::stiff()    // high stiffness, quick
SpringConfig::slow()     // low stiffness, gradual
```

Or custom:
```rust
let config = SpringConfig {
    stiffness: 200.0,
    damping: 15.0,
    mass: 1.0,
    epsilon: 0.001,
};
```

### Spring (1D)

```rust
let mut spring = Spring::new(SpringConfig::wobbly())
    .with_position(0.0);

spring.set_target(100.0);

loop {
    spring.update(1.0 / 60.0);
    let pos = spring.position();     // current value
    let vel = spring.velocity();
    let target = spring.target();
    if spring.is_settled() { break; }
}
spring.reset();
```

### SpringN (Multi-Dimensional)

```rust
let mut spring = SpringN::new(SpringConfig::gentle(), [0.0_f32, 0.0]);
spring.set_target([100.0, 200.0]);

spring.update(dt);
let pos: [f32; 2] = spring.position();
let components: &[f32] = spring.position_components();
let vel_components: &[f32] = spring.velocity_components();
```

### SpringAnimatable Trait
Implement for custom multi-dimensional types:
```rust
pub trait SpringAnimatable: Clone + 'static {
    fn to_components(&self) -> Vec<f32>;
    fn from_components(components: &[f32]) -> Self;
}
// Built-in: f32, [f32; 2], [f32; 3], [f32; 4]
// With palette feature: Srgba, Srgb, LinSrgba, LinSrgb, Lab, Laba, InLab, InOklch, InLinear
```

---

## Inertia (Friction Deceleration)

```rust
use spanda::{Inertia, InertiaN, InertiaConfig, Update};
```

### InertiaConfig Presets
```rust
InertiaConfig::default_flick()  // friction: 0.05, epsilon: 0.1
InertiaConfig::heavy()          // friction: 0.02 (slow stop)
InertiaConfig::snappy()         // friction: 0.1 (fast stop)
```

### Inertia (1D)
```rust
let mut inertia = Inertia::new(InertiaConfig::default_flick())
    .with_velocity(500.0)
    .with_position(100.0);

while !inertia.is_settled() {
    inertia.update(1.0 / 60.0);
    let pos = inertia.position();
    let vel = inertia.velocity();
}

inertia.kick(800.0);  // re-apply velocity
inertia.reset();
```

### InertiaN (Multi-Dimensional)
```rust
let mut inertia = InertiaN::new(InertiaConfig::default_flick(), [100.0_f32, 200.0])
    .with_velocity([300.0, -150.0]);

inertia.update(dt);
let pos: [f32; 2] = inertia.position();

inertia.kick([400.0, 0.0]); // re-apply velocity
inertia.reset([0.0, 0.0]);  // reset to position
```

---

## Drag State

```rust
use spanda::{DragState, DragConstraints, DragAxis, PointerData};
```

### Pure-Math Drag Tracker (No DOM)
```rust
let mut drag = DragState::new()
    .with_position([100.0, 100.0])
    .with_constraints(DragConstraints {
        bounds: Some([0.0, 0.0, 500.0, 500.0]),  // [min_x, min_y, max_x, max_y]
        axis_lock: Some(DragAxis::X),              // or DragAxis::Y or None
        snap_to_grid: Some([20.0, 20.0]),          // snap to grid
        ..Default::default()
    });

// Pointer events:
drag.on_pointer_down(150.0, 150.0);
drag.on_pointer_move(170.0, 160.0, dt);
let pos: [f32; 2] = drag.position();
let vel: [f32; 2] = drag.velocity();
let dragging: bool = drag.is_dragging();

// Release → get momentum:
let mut inertia: InertiaN<[f32; 2]> = drag.on_pointer_up();
inertia.update(dt);
let fling_pos = inertia.position();
```

### PointerData
```rust
pub struct PointerData {
    pub x: f32,
    pub y: f32,
    pub pressure: f32,
    pub pointer_id: i32,
}
```

---

## Clock & Driver

```rust
use spanda::{Clock, ManualClock, MockClock, AnimationDriver, AnimationId};
#[cfg(feature = "std")]
use spanda::WallClock;
```

### Clock Trait
```rust
pub trait Clock {
    fn delta(&mut self) -> f32;  // returns seconds since last call
}
```

### Clock Implementations
```rust
// Real time (feature = "std")
let mut clock = WallClock::new();
let dt = clock.delta();

// Manual control (for testing / custom loops)
let mut clock = ManualClock::new();
clock.advance(1.0 / 60.0);
let dt = clock.delta(); // returns 1/60, then 0 until next advance()

// Fixed step (for deterministic testing)
let mut clock = MockClock::new(1.0 / 60.0);
let dt = clock.delta(); // always returns 1/60
```

### AnimationDriver
```rust
let mut driver = AnimationDriver::new();
let id: AnimationId = driver.add(my_tween);
driver.tick(dt);                     // ticks all active animations
driver.cancel(id);                   // remove one
driver.cancel_all();                 // remove all
let count = driver.active_count();   // how many running
```

### AnimationDriverArc (thread-safe, feature = "std")
```rust
use spanda::driver::AnimationDriverArc;
let driver = AnimationDriverArc::new();
let id = driver.add(my_tween);  // Arc<Mutex<>> inside
driver.tick(dt);
driver.cancel(id);
driver.cancel_all();
```

---

## Scroll-Linked Animation

```rust
use spanda::{ScrollClock, ScrollDriver, AnimationId, Update};
```

### ScrollClock
```rust
let mut clock = ScrollClock::new(0.0, 1000.0); // scroll start, end
clock.set_position(500.0);                       // set current scroll
let progress = clock.progress();                 // 0.0..=1.0
let delta = clock.delta();                       // change since last delta()
clock.set_range(0.0, 2000.0);                   // change range
```

### ScrollDriver
```rust
let mut driver = ScrollDriver::new(0.0, 1000.0);
let id = driver.add(my_tween);
driver.set_position(scroll_y);  // call on scroll
let progress = driver.progress();
driver.cancel(id);
driver.cancel_all();
```

---

## Paths

```rust
use spanda::{BezierPath, MotionPath, MotionPathTween, PathEvaluate, Update};
```

### BezierPath
```rust
let linear    = BezierPath::linear([0.0, 0.0], [100.0, 100.0]);
let quadratic = BezierPath::quadratic([0.0, 0.0], [50.0, 100.0], [100.0, 0.0]);
let cubic     = BezierPath::cubic([0.0, 0.0], [25.0, 100.0], [75.0, 100.0], [100.0, 0.0]);

let point: [f32; 2] = linear.evaluate(0.5); // PathEvaluate trait
```

### MotionPath
```rust
let path = MotionPath::new()
    .line([0.0, 0.0], [100.0, 0.0])
    .quadratic([100.0, 0.0], [150.0, 50.0], [100.0, 100.0])
    .cubic_weighted([100.0, 100.0], [50.0, 150.0], [0.0, 50.0], [0.0, 0.0], 2.0);

let point: [f32; 2] = path.evaluate(0.5);
```

### MotionPathTween (animate along a path)
```rust
let mut tween = MotionPathTween::new(path)
    .duration(2.0)
    .easing(Easing::EaseInOutCubic);

tween.update(dt);
let pos: [f32; 2] = tween.value();
let progress = tween.progress();
```

---

## Catmull-Rom Splines

```rust
use spanda::{CatmullRomSpline, PathEvaluate2D, tangent_angle, tangent_angle_deg};
```

```rust
let spline = CatmullRomSpline::new(vec![
    [0.0, 0.0], [100.0, 50.0], [200.0, 0.0], [300.0, 50.0]
]).tension(0.5);

let pos: [f32; 2] = spline.evaluate([0.0, 0.0], 0.5);
let tangent: [f32; 2] = spline.tangent([0.0, 0.0], 0.5);
let angle_rad = tangent_angle(tangent);
let angle_deg = tangent_angle_deg(tangent);
```

---

## Motion Paths (Arc-Length Parameterised)

```rust
use spanda::{PolyPath, CompoundPath, PathCommand};
```

### PolyPath (Catmull-Rom based)
```rust
let path = PolyPath::from_points(vec![
    [0.0, 0.0], [100.0, 50.0], [200.0, 0.0]
])
.start_offset(0.1)       // skip first 10%
.end_offset(0.9)         // stop at 90%
.rotation_offset(45.0);  // degrees

let pos: [f32; 2] = path.position(0.5);     // uniform-speed position
let tangent: [f32; 2] = path.tangent(0.5);
let rot_rad = path.rotation(0.5);
let rot_deg = path.rotation_deg(0.5);
let length = path.arc_length();
```

### CompoundPath (from PathCommands)
```rust
let path = CompoundPath::new(vec![
    PathCommand::MoveTo([0.0, 0.0]),
    PathCommand::CubicTo {
        control1: [50.0, 100.0],
        control2: [150.0, 100.0],
        end: [200.0, 0.0],
    },
    PathCommand::LineTo([300.0, 0.0]),
]);

let pos = path.position(0.5);
```

---

## SVG Path Parser

```rust
use spanda::{SvgPathParser, PathCommand, CompoundPath};
```

```rust
let commands: Vec<PathCommand> = SvgPathParser::parse("M 0 0 C 50 100 150 100 200 0 L 300 0");
let path = CompoundPath::new(commands);

// Supports: M/m, L/l, H/h, V/v, Q/q, C/c, Z/z
```

---

## DrawSVG (Stroke Animation)

```rust
use spanda::{draw_on, draw_on_reverse, Update, Easing};
```

```rust
// Animate stroke-dashoffset from path_length → 0 (draw on)
let mut tween = draw_on(300.0)
    .duration(1.5)
    .easing(Easing::EaseInOutCubic)
    .build();

tween.update(dt);
let dash_offset: f32 = tween.value();
// Apply: element.style.stroke_dashoffset = dash_offset

// Reverse: erase stroke (0 → path_length)
let mut erase = draw_on_reverse(300.0).duration(0.8).build();
```

---

## Shape Morphing

```rust
use spanda::{MorphPath, resample, Update, Easing};
```

```rust
let triangle = vec![[0.0, 0.0], [50.0, 100.0], [100.0, 0.0]];
let square   = vec![[0.0, 0.0], [0.0, 100.0], [100.0, 100.0], [100.0, 0.0]];

// Auto-resamples shorter shape to match point counts
let mut morph = MorphPath::new(triangle, square)
    .duration(1.0)
    .easing(Easing::EaseInOutCubic)
    .build();

morph.update(dt);
let shape: Vec<[f32; 2]> = morph.value();
morph.seek(0.5);   // jump to 50%
morph.reset();

// Manual resampling utility:
let smooth = resample(&rough_points, 50); // 50 evenly-spaced points
```

---

## Colour Interpolation (feature = "palette")

```rust
use spanda::colour::{InLab, InOklch, InLinear, lerp_in_lab, lerp_in_oklch, lerp_in_linear};
use palette::Srgba;
```

### Direct Interpolation
```rust
// Palette types implement Interpolate directly (component-wise lerp in native space)
use spanda::Interpolate;
let a = Srgba::new(1.0, 0.0, 0.0, 1.0);
let b = Srgba::new(0.0, 0.0, 1.0, 1.0);
let mid = a.lerp(&b, 0.5);
```

### Colour-Space Wrappers (for Tween<T>)
```rust
// Perceptually uniform interpolation via Lab space:
let mut tween = Tween::new(InLab(red), InLab(blue)).duration(1.0).build();
tween.update(dt);
let color: Srgba = tween.value().0; // unwrap the wrapper

// Alternatives:
let mut tween = Tween::new(InOklch(red), InOklch(blue)).duration(1.0).build();
let mut tween = Tween::new(InLinear(red), InLinear(blue)).duration(1.0).build();
```

### Free Functions
```rust
let mid = lerp_in_lab(a, b, 0.5);     // one-shot Lab-space lerp
let mid = lerp_in_oklch(a, b, 0.5);   // one-shot Oklch-space lerp
let mid = lerp_in_linear(a, b, 0.5);  // one-shot linear RGB lerp
```

### SpringAnimatable for Colours
```rust
// palette types + wrappers implement SpringAnimatable
let mut spring = SpringN::new(SpringConfig::gentle(), Srgba::new(1.0, 0.0, 0.0, 1.0));
spring.set_target(Srgba::new(0.0, 0.0, 1.0, 1.0));
```

**Supported palette types:** `Srgba`, `Srgb`, `LinSrgba`, `LinSrgb`, `Lab`, `Laba`, `Oklch`, `Oklcha`, `Hsla`

---

## SplitText

```rust
use spanda::integrations::split_text::SplitText;
```

### Core (always available)
```rust
let split = SplitText::from_str("Hello World");
let chars: &[SplitChar] = split.chars();     // each char with index, word_index
let words: &[SplitWord] = split.words();     // each word with text, indexes
let original: &str = split.original();
```

### Stagger Timelines
```rust
let tl: Timeline = split.stagger_chars(0.0_f32, 1.0, 0.3, 0.05, Easing::EaseOutCubic);
let tl: Timeline = split.stagger_words(0.0_f32, 1.0, 0.4, 0.1, Easing::EaseOutCubic);
// Parameters: from, to, duration_each, delay_between, easing
```

### DOM Injection (feature = "wasm-dom")
```rust
split.inject_chars(&parent_element);    // wraps each char in <span>
split.inject_words(&parent_element);    // wraps each word in <span>
let lines: Vec<Vec<usize>> = SplitText::detect_lines(&container);
```

---

## Bevy Integration (feature = "bevy")

```rust
use spanda::integrations::bevy::{SpandaPlugin, TweenCompleted, SpringSettled, AnimationLabel};
```

```rust
// Add the plugin:
app.add_plugins(SpandaPlugin);

// Attach tweens/springs as components:
commands.spawn((
    Tween::new(0.0_f32, 1.0).duration(1.0).build(),
    AnimationLabel::new("fade"),
));
commands.spawn((
    Spring::new(SpringConfig::gentle()).with_position(0.0),
));

// Listen for completion events:
fn on_complete(mut events: EventReader<TweenCompleted>) {
    for ev in events.read() {
        println!("Entity {:?} tween done", ev.entity);
    }
}
```

Auto-ticks: `Tween<f32>`, `Tween<[f32; 2]>`, `Tween<[f32; 3]>`, `Tween<[f32; 4]>`, `Spring`

---

## WASM Integration (feature = "wasm")

```rust
use spanda::integrations::wasm::{RafDriver, start_raf_loop};
```

### RafDriver
```rust
let mut driver = RafDriver::new();
let id = driver.add(my_tween);

start_raf_loop(move |timestamp_ms| {
    driver.tick(timestamp_ms);
});

// Control:
driver.pause();
driver.resume();
driver.set_time_scale(2.0);
driver.on_visibility_change(document_hidden);
driver.cancel(id);
driver.cancel_all();
```

---

## WASM-DOM Plugins (feature = "wasm-dom")

### Observer (Pointer Event Normaliser)
```rust
use spanda::integrations::observer::{Observer, ObserverCallbacks};
use spanda::PointerData;

let observer = Observer::bind(&element, ObserverCallbacks {
    on_press: Some(Box::new(|data: PointerData| { /* pointer down */ })),
    on_move: Some(Box::new(|data: PointerData| { /* pointer move */ })),
    on_release: Some(Box::new(|data: PointerData| { /* pointer up */ })),
    on_wheel: Some(Box::new(|dx: f32, dy: f32| { /* wheel scroll */ })),
});

observer.unbind(); // cleanup
```

### FLIP Animations
```rust
use spanda::integrations::flip::{FlipState, FlipAnimationBuilder, FlipAnimation};

let first = FlipState::capture(&element);           // snapshot before layout change
// ... change layout ...
let last = FlipState::capture(&element);             // snapshot after

let mut anim: FlipAnimation = FlipState::diff(&first, &last)
    .duration(0.6)
    .easing(Easing::EaseOutCubic)
    .build();

anim.update(dt);
let (tx, ty, sx, sy) = anim.transform();               // raw values
let css: String = anim.css_transform();                 // "translate(Xpx, Ypx) scale(SX, SY)"

// Or from known rects:
let state = FlipState::from_rect(0.0, 0.0, 100.0, 100.0);
```

### ScrollSmoother
```rust
use spanda::integrations::scroll_smoother::ScrollSmoother;
use spanda::SpringConfig;

let mut smoother = ScrollSmoother::new(content_element, SpringConfig::gentle());
smoother.attach();  // listen to scroll events

// Each frame:
smoother.tick(dt);
let y = smoother.position();
let target = smoother.target();

smoother.detach(); // cleanup
```

### Draggable (DOM)
```rust
use spanda::integrations::draggable::Draggable;
use spanda::DragConstraints;

let draggable = Draggable::bind(&element);
// Or with constraints:
let draggable = Draggable::bind_with_constraints(&element, DragConstraints {
    bounds: Some([0.0, 0.0, 500.0, 500.0]),
    ..Default::default()
});

let pos: [f32; 2] = draggable.position();
let dragging: bool = draggable.is_dragging();
let state: DragState = draggable.state();

draggable.unbind(); // cleanup
```

---

## Common Integration Patterns

### Basic Frame Loop (std)
```rust
use spanda::{WallClock, AnimationDriver, Tween, Easing, Clock, Update};

let mut clock = WallClock::new();
let mut driver = AnimationDriver::new();

let tween = Tween::new(0.0_f32, 100.0)
    .duration(1.0)
    .easing(Easing::EaseOutCubic)
    .build();
let id = driver.add(tween);

loop {
    let dt = clock.delta();
    driver.tick(dt);
    if driver.active_count() == 0 { break; }
}
```

### Leptos / Yew Pattern (wasm)
```rust
use spanda::integrations::wasm::{RafDriver, start_raf_loop};
use spanda::{Tween, Easing};

let mut driver = RafDriver::new();
driver.add(Tween::new(0.0_f32, 1.0).duration(0.5).easing(Easing::EaseOutCubic).build());

start_raf_loop(move |ts| {
    driver.tick(ts);
    // read values, update DOM
});
```

### Bevy ECS Pattern
```rust
use bevy::prelude::*;
use spanda::{Tween, Spring, SpringConfig, Easing};
use spanda::integrations::bevy::{SpandaPlugin, TweenCompleted};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, SpandaPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, read_values)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Tween::new(0.0_f32, 100.0).duration(2.0).build());
}

fn read_values(query: Query<&Tween<f32>>) {
    for tween in &query {
        let val = tween.value();
    }
}
```

### Spring-to-Target Pattern
```rust
use spanda::{Spring, SpringConfig, Update};

let mut spring = Spring::new(SpringConfig::wobbly()).with_position(0.0);

// Change target any time — spring smoothly follows:
spring.set_target(100.0);
// later:
spring.set_target(50.0);

// Each frame:
spring.update(dt);
let pos = spring.position();
```

### Drag → Fling Pattern
```rust
use spanda::{DragState, DragConstraints, InertiaN, Update};

let mut drag = DragState::new().with_position([0.0, 0.0]);
drag.on_pointer_down(x, y);
// ... on move:
drag.on_pointer_move(x, y, dt);

// On release, transition to momentum:
let mut inertia = drag.on_pointer_up();
while !inertia.is_settled() {
    inertia.update(dt);
    let pos = inertia.position();
}
```

---

## Testing

```bash
cargo test                         # all tests
cargo test --features palette      # include colour tests
cargo test --no-default-features   # no_std check
cargo clippy --all-features        # lint
```
