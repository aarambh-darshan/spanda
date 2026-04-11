# Spanda Documentation

> *Sanskrit: स्पन्द — vibration, pulse, the throb of motion.*

Welcome to the **spanda** documentation! This library is built around a single, powerful idea: **any value that can be linearly interpolated can be animated.**

Everything else — easing curves, keyframe tracks, timelines, physics springs, scroll-linked animations, and motion paths — is layered on top of that one primitive.

---

## Philosophy

Spanda is a **pure value interpolation engine**. It computes animation values mathematically; it never touches the DOM, Canvas, or any rendering target. You provide time (`dt`), spanda returns values — you render however you want.

This architecture makes spanda work **everywhere**:
- Terminal UIs (ratatui, crossterm)
- Web / WASM (Leptos, Dioxus, Yew)
- Game engines (Bevy)
- Native desktop apps
- Embedded / `no_std` environments

---

## Core Modules

| Module | Description | Guide |
|--------|-------------|-------|
| **Tween** | Animate a single value from A to B with easing, looping, callbacks | [tween.md](tween.md) |
| **Easing** | 38+ easing curves (31 standard + CSS + 5 advanced) | [easing.md](easing.md) |
| **Keyframes** | Multi-stop animation with per-segment easing | [keyframe.md](keyframe.md) |
| **Timeline & Sequence** | Compose animations concurrently or sequentially | [timeline.md](timeline.md) |
| **Spring** | Physics-based damped harmonic oscillator + generic SpringN<T> | [spring.md](spring.md) |
| **Scroll** | Scroll-linked animations with ScrollDriver/ScrollClock | [scroll.md](scroll.md) |
| **Motion Paths** | Bezier curves, CatmullRom, PolyPath, CompoundPath, SVG parser | [path.md](path.md) |
| **Colour** | 9 palette types, InLab/InOklch/InLinear colour-space wrappers | [colour.md](colour.md) |
| **DrawSVG** | Stroke-dashoffset draw-on/draw-off helpers | [v080.md](v080.md) |
| **Morph** | MorphPath point-by-point shape morphing + resample | [v080.md](v080.md) |
| **Inertia** | Friction deceleration (Inertia, InertiaN) with presets | [v080.md](v080.md) |
| **Drag** | Pure-math drag tracking with velocity, constraints, grid snap | [v080.md](v080.md) |
| **Advanced Easings** | RoughEase, SlowMo, ExpoScale, Wiggle, CustomBounce | [easing.md](easing.md) |
| **WASM-DOM Plugins** | FLIP, SplitText, ScrollSmoother, SmoothScroll, Draggable, Observer | [v080.md](v080.md) |
| **Layout** | FLIP-style layout animation, SharedElementTransition | [layout.md](layout.md) |
| **Gesture** | GestureRecognizer — tap, swipe, pinch, rotate, long press | [gesture.md](gesture.md) |
| **GPU** | GpuAnimationBatch — batch 10K+ tweens on GPU compute shaders | [gpu.md](gpu.md) |
| **Driver & Clock** | Manage multiple animations with time abstraction | [integrations.md](integrations.md) |
| **Leptos** | Reactive signal integration guide | [leptos_guide.md](leptos_guide.md) |
| **Dioxus** | Coroutine-based animation guide | [dioxus_guide.md](dioxus_guide.md) |
| **Feature Matrix** | Which flags enable which modules | [feature_matrix.md](feature_matrix.md) |

---

## Quick Start

Add `spanda` to your `Cargo.toml`:

```toml
[dependencies]
spanda = "0.9.3"
```

### Basic Tween

```rust
use spanda::{Tween, Easing};
use spanda::traits::Update;

let mut tween = Tween::new(0.0_f32, 100.0)
    .duration(1.0)
    .easing(Easing::EaseOutCubic)
    .build();

// Inside your rendering loop:
for _ in 0..10 {
    tween.update(0.1);
    let current = tween.value();
    // render(current);
}

assert!(tween.is_complete());
assert!((tween.value() - 100.0).abs() < 1e-6);
```

### Looping Tween

```rust
use spanda::{Tween, Loop};
use spanda::traits::Update;

let mut tween = Tween::new(0.0_f32, 100.0)
    .duration(1.0)
    .looping(Loop::PingPong)
    .build();

// Bounces between 0 and 100 forever
for _ in 0..600 {
    tween.update(1.0 / 60.0);
}
```

### Spring Animation

```rust
use spanda::spring::{Spring, SpringConfig};
use spanda::traits::Update;

let mut spring = Spring::new(SpringConfig::wobbly());
spring.set_target(100.0);

// Springs have no fixed duration — they settle naturally
for _ in 0..300 {
    spring.update(1.0 / 60.0);
    let pos = spring.position();
    // render(pos);
}

assert!(spring.is_settled());
```

### Timeline Composition

```rust
use spanda::timeline::Sequence;
use spanda::tween::Tween;
use spanda::traits::Update;

let mut timeline = Sequence::new()
    .then(Tween::new(0.0_f32, 100.0).duration(0.5).build(), 0.5)
    .gap(0.1)
    .then(Tween::new(100.0_f32, 0.0).duration(0.3).build(), 0.3)
    .build();

timeline.play();
timeline.update(0.9);
```

### Staggered Animations

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
// Animations start at t=0.0, 0.1, 0.2, 0.3, 0.4
```

### Scroll-Linked Animation

```rust
use spanda::scroll::ScrollDriver;
use spanda::tween::Tween;

let mut driver = ScrollDriver::new(0.0, 1000.0);
driver.add(Tween::new(0.0_f32, 100.0).duration(1.0).build());

// Drive from scroll position instead of time
driver.set_position(500.0);
```

### Motion Path Animation

```rust
use spanda::path::{MotionPath, MotionPathTween, PathEvaluate};
use spanda::easing::Easing;
use spanda::traits::Update;

let path = MotionPath::new()
    .cubic([0.0_f32, 0.0], [50.0, 100.0], [100.0, 100.0], [150.0, 0.0])
    .line([150.0, 0.0], [200.0, 0.0]);

let mut tween = MotionPathTween::new(path)
    .duration(2.0)
    .easing(Easing::EaseInOutCubic);

tween.update(1.0);
let pos = tween.value();
```

### Colour Animation (palette feature)

```rust,ignore
use palette::Srgba;
use spanda::{Tween, Easing};
use spanda::colour::InLab;
use spanda::traits::Update;

// Interpolate in CIE L*a*b* for perceptually smooth gradients
let mut tween = Tween::new(
    InLab(Srgba::new(1.0, 0.0, 0.0, 1.0)),  // red
    InLab(Srgba::new(0.0, 0.0, 1.0, 1.0)),  // blue
)
    .duration(1.0)
    .easing(Easing::EaseInOutCubic)
    .build();

tween.update(0.5);
let colour = tween.value().0;  // Srgba
```

### Using the Animation Driver

```rust
use spanda::driver::AnimationDriver;
use spanda::tween::Tween;
use spanda::easing::Easing;

let mut driver = AnimationDriver::new();

// Add multiple animations — they're managed automatically
let id = driver.add(
    Tween::new(0.0_f32, 1.0).duration(1.0).build()
);

// Tick all animations each frame
driver.tick(0.5);
assert_eq!(driver.active_count(), 1);

driver.tick(0.5);
assert_eq!(driver.active_count(), 0); // completed animations are auto-removed
```

---

## Feature Flags

| Flag | What it adds | Use case |
|------|-------------|----------|
| `std` | *(default)* Wall-clock driver, thread-safe `AnimationDriverArc` | TUI apps, CLI tools |
| `serde` | `Serialize`/`Deserialize` on all public types | State persistence, network sync |
| `bevy` | `SpandaPlugin` for Bevy 0.18 — auto-ticks Tween/Spring components | Game development |
| `wasm` | `RafDriver` for browser `requestAnimationFrame` | Web apps (Leptos/Dioxus/Yew) |
| `wasm-dom` | DOM plugins: FLIP, SplitText, ScrollSmoother, SmoothScroll, Draggable, Observer | Web apps with DOM interaction |
| `palette` | Colour interpolation via the `palette` crate | Smooth colour animations |
| `tokio` | `async`/`.await` on timeline completion | Async workflows |
| `gpu` | GPU compute shader batch animation via `wgpu` | Particle systems, large batches |

### Feature Flag Decision Guide

| You are building... | Recommended features |
|---------------------|----------------------|
| A TUI app | `default` (just `std`) |
| A Bevy game | `bevy` |
| A WASM web app | `wasm` |
| A WASM app with DOM plugins | `wasm-dom` |
| A CLI tool | `default` |
| Embedded / `no_std` | `default-features = false` |
| GPU-accelerated particle animations | `gpu` |
| Full everything | `std,serde,bevy,wasm,wasm-dom,palette,tokio,gpu` |

---

## Architecture Overview

```
┌──────────────────────────────────────────────┐
│              Your Application                │
│   (TUI / Bevy / Leptos / Desktop / CLI)      │
└──────────────────────┬───────────────────────┘
                       │ reads value()
┌──────────────────────▼───────────────────────┐
│         AnimationDriver / ScrollDriver       │
│    manages multiple active animations,       │
│    auto-removes completed ones               │
└──────────────────────┬───────────────────────┘
                       │ .tick(dt) / .set_position(pos)
        ┌──────────────┼──────────────┬──────────────┐
        ▼              ▼              ▼              ▼
   ┌─────────┐   ┌──────────┐  ┌──────────┐  ┌────────────┐
   │ Tween<T>│   │ Keyframe │  │  Spring  │  │MotionPath  │
   │         │   │ Track<T> │  │ Inertia  │  │  Tween     │
   └─────────┘   └──────────┘  └──────────┘  └────────────┘
        │              │              │              │
        ▼              ▼              ▼              ▼
   ┌──────────────────────────────────────────────────────┐
   │          Interpolate / Animatable                    │
   │   (f32, f64, [f32;2..4], i32, palette colours)      │
   └──────────────────────────────────────────────────────┘
        │                                          │
        ▼                                          ▼
   ┌─────────────┐                         ┌──────────────┐
   │  MorphPath  │                         │  DragState   │
   │  DrawSVG    │                         │  → InertiaN  │
   └─────────────┘                         └──────────────┘
```

### Data Flow

1. **Clock** produces `dt` (seconds since last frame) — or `ScrollClock` produces dt from scroll position
2. **AnimationDriver** / **ScrollDriver** calls `update(dt)` on every active animation
3. Each animation (**Tween**, **KeyframeTrack**, **Spring**, **MotionPathTween**) advances its internal state
4. Your app reads `.value()` / `.position()` and renders

---

## Type System

The trait hierarchy is minimal and powerful:

```
Interpolate       — lerp(&self, other: &Self, t: f32) -> Self
    └── Animatable  — Interpolate + Clone + 'static (blanket impl)
            └── Used as bounds on: Tween<T>, KeyframeTrack<T>

PathEvaluate<T>   — evaluate(&self, t: f32) -> T
    └── Implemented by: BezierPath<T>, MotionPath<T>

Update            — update(&mut self, dt: f32) -> bool
    └── Implemented by: Tween, KeyframeTrack, Timeline, Spring, MotionPathTween
```

To animate your own type, just implement `Interpolate`:

```rust
use spanda::traits::Interpolate;

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

// Now you can do: Tween::new(red, blue).duration(1.0).build()
```

---

## Project Structure

```
src/
├── lib.rs           — crate root, re-exports
├── traits.rs        — Interpolate, Animatable, Update
├── easing.rs        — 38 easing functions + CubicBezier + Steps + 5 advanced
├── tween.rs         — Tween<T>, TweenBuilder, TweenState
├── keyframe.rs      — KeyframeTrack, Keyframe, Loop
├── timeline.rs      — Timeline, Sequence, At, stagger
├── spring.rs        — Spring, SpringConfig, SpringN<T>, SpringAnimatable
├── clock.rs         — Clock trait, WallClock, ManualClock, MockClock
├── driver.rs        — AnimationDriver, AnimationDriverArc, AnimationId
├── scroll.rs        — ScrollClock, ScrollDriver
├── scroll_smooth.rs — SmoothScroll1D (exponential smooth scroll core)
├── path.rs          — BezierPath, MotionPath, MotionPathTween
├── bezier.rs        — CatmullRomSpline, PathEvaluate2D
├── motion_path.rs   — PolyPath, CompoundPath, PathCommand
├── svg_path.rs      — SvgPathParser (SVG d-attribute parser)
├── colour.rs        — colour interpolation (feature = "palette")
├── svg_draw.rs      — DrawSVG stroke-dashoffset helpers
├── morph.rs         — MorphPath shape morphing + resample
├── inertia.rs       — Inertia, InertiaN friction deceleration
├── drag.rs          — DragState, DragConstraints, PointerData
├── layout.rs        — LayoutAnimator, Rect, SharedElementTransition
├── gesture.rs       — GestureRecognizer, Gesture, GestureConfig
├── gpu.rs           — GpuAnimationBatch (feature = "gpu")
├── gpu_tween.wgsl   — WGSL compute shader
└── integrations/
    ├── mod.rs
    ├── bevy.rs      — SpandaPlugin  (feature = "bevy")
    ├── wasm.rs      — RafDriver     (feature = "wasm")
    ├── split_text.rs — SplitText character/word splitting
    ├── flip.rs      — FlipState, FlipAnimation (feature = "wasm-dom")
    ├── scroll_smoother.rs — ScrollSmoother (feature = "wasm-dom")
    ├── smooth_scroll.rs — SmoothScroll (feature = "wasm-dom")
    ├── draggable.rs — Draggable DOM binding (feature = "wasm-dom")
    └── observer.rs  — Observer unified input (feature = "wasm-dom")
```

---

## Integrations

See the full [Integrations Guide](integrations.md) for:
- **Bevy** — `SpandaPlugin` auto-ticks components
- **WASM** — `RafDriver` for browser `requestAnimationFrame`
- **TUI/CLI** — `WallClock` + render loop
- **Leptos/Dioxus** — WASM driver patterns, `on_update` callback bridge
- **Scroll-linked** — `ScrollDriver` for position-driven animations
- **Embedded / `no_std`** — zero dependency math-only mode

---

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Run `cargo test && cargo clippy --all-features -- -D warnings`
4. Submit a pull request

---

## License

Licensed under either of

- [Apache License, Version 2.0](../LICENSE-APACHE)
- [MIT License](../LICENSE-MIT)

at your option.
