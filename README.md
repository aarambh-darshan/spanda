# spanda

*Sanskrit: स्पन्द — vibration, pulse, the throb of motion.*

A general-purpose **animation library** for Rust.  Zero mandatory dependencies,
`no_std`-ready, and designed to work anywhere: terminal UIs, web (WASM),
game engines (Bevy), or native desktop apps.

## Features

- **Tweening** — animate any value from A to B with 38+ easing curves
- **Keyframe tracks** — multi-stop animations with per-segment easing
- **Timeline & Sequence** — compose animations concurrently or sequentially
- **Relative positioning** — GSAP-style `At::Start`, `At::End`, `At::Label`, `At::Offset`
- **Stagger** — offset N animations with a single call
- **Physics springs** — damped harmonic oscillator with 4 presets + `SpringN<T>` for 2D/3D/4D
- **Looping** — `Loop::Once`, `Times(n)`, `Forever`, `PingPong` on tweens and keyframes
- **Time scale** — speed up / slow down tweens and timelines at runtime
- **Callbacks** — `on_start`, `on_update`, `on_complete` on tweens (`std` feature)
- **Value modifiers** — `snap_to(grid)`, `round_to(decimals)`, custom modifiers
- **Scroll-linked animation** — `ScrollDriver` / `ScrollClock` for position-driven animations
- **Motion paths** — quadratic/cubic Bezier, CatmullRom splines, SVG path parsing, arc-length parameterization
- **Full motion path system** — `PolyPath`, `CompoundPath`, `SvgPathParser`, auto-rotate, start/end offsets
- **CSS easing** — `CubicBezier(x1,y1,x2,y2)` and `Steps(n)` on the `Easing` enum
- **Colour animation** — 9 palette types + `InLab`/`InOklch`/`InLinear` colour-space-aware wrappers (`palette` feature)
- **DrawSVG** — `draw_on` / `draw_on_reverse` stroke-dashoffset helpers
- **Shape morphing** — `MorphPath` point-by-point morph with auto-resampling
- **Inertia physics** — `Inertia` / `InertiaN<T>` friction deceleration with presets
- **Advanced easings** — `RoughEase`, `SlowMo`, `ExpoScale`, `Wiggle`, `CustomBounce`
- **Drag tracking** — `DragState` with velocity EMA, bounds, axis lock, grid snap → `InertiaN` on release
- **WASM-DOM plugins** — FLIP animations, SplitText, ScrollSmoother, Draggable, Observer (`wasm-dom` feature)
- **Layout animation** — automatic FLIP-style transitions with `LayoutAnimator`, shared element transitions
- **Gesture recognition** — `GestureRecognizer` for tap, swipe, long press, pinch, rotation
- **GPU compute shaders** — `GpuAnimationBatch` for batch-evaluating 10,000+ tweens on the GPU (`gpu` feature)
- **Animation driver** — manage multiple animations with auto-cleanup
- **Clock abstraction** — wall clock, manual clock, scroll clock, and mock clock for testing

## Getting Started

```toml
[dependencies]
spanda = "0.9.1"
```

### Quick Example

```rust
use spanda::{Tween, Easing};
use spanda::traits::Update;

let mut tween = Tween::new(0.0_f32, 100.0)
    .duration(1.0)
    .easing(Easing::EaseOutCubic)
    .build();

// Simulate 10 frames:
for _ in 0..10 {
    tween.update(0.1);
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

// Runs forever, bouncing between 0 and 100
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

for _ in 0..300 {
    spring.update(1.0 / 60.0);
}
assert!(spring.is_settled());
```

### Multi-Dimensional Spring (SpringN)

```rust
use spanda::spring::{SpringN, SpringConfig};
use spanda::traits::Update;

let mut spring = SpringN::new(SpringConfig::wobbly(), [0.0_f32, 0.0]);
spring.set_target([100.0, 200.0]);

for _ in 0..1000 {
    spring.update(1.0 / 60.0);
}

let pos = spring.position(); // [f32; 2]
assert!(spring.is_settled());
```

### Timeline with Relative Positioning

```rust
use spanda::timeline::{Timeline, At};
use spanda::tween::Tween;
use spanda::easing::Easing;

let mut tl = Timeline::new()
    .add("fade", Tween::new(0.0_f32, 1.0).duration(0.5).build(), 0.0);

// Place "slide" right after "fade" ends
tl.add_at("slide", Tween::new(0.0_f32, 100.0).duration(0.8).build(), 0.8, At::End);

// Place "glow" at the same time as "fade"
tl.add_at("glow", Tween::new(0.0_f32, 1.0).duration(0.3).build(), 0.3, At::Label("fade"));

tl.play();
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

// Drive animation from scroll position instead of time
driver.set_position(500.0); // 50% scroll
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
let pos = tween.value(); // position along the curve
```

### Smooth Path Through Points (PolyPath)

```rust
use spanda::motion_path::PolyPath;

let path = PolyPath::from_points(vec![
    [0.0, 0.0],
    [100.0, 50.0],
    [200.0, 0.0],
    [300.0, 50.0],
]);

let pos = path.position(0.5);     // arc-length parameterized
let angle = path.rotation_deg(0.5); // auto-rotate angle
```

### SVG Path Parsing

```rust
use spanda::svg_path::SvgPathParser;
use spanda::motion_path::CompoundPath;

let commands = SvgPathParser::parse("M 0 0 C 50 100 100 100 150 0 L 200 0");
let path = CompoundPath::new(commands)
    .start_offset(0.1)
    .end_offset(0.9);

let pos = path.position(0.5);
```

### CSS Cubic-Bezier Easing

```rust
use spanda::{Tween, Easing};
use spanda::traits::Update;

let mut tween = Tween::new(0.0_f32, 100.0)
    .duration(1.0)
    .easing(Easing::CubicBezier(0.25, 0.1, 0.25, 1.0)) // CSS ease
    .build();

tween.update(0.5);
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

### Sequence Composition

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

### Layout Animation (FLIP)

```rust
use spanda::layout::{LayoutAnimator, Rect};
use spanda::easing::Easing;

let mut layout = LayoutAnimator::new();
layout.track("card-1", Rect::new(0.0, 0.0, 200.0, 100.0));
layout.track("card-2", Rect::new(0.0, 120.0, 200.0, 100.0));

// After layout mutation (e.g. items reordered)
let transitions = layout.compute_transitions(
    &[
        ("card-1", Rect::new(0.0, 120.0, 200.0, 100.0)),
        ("card-2", Rect::new(0.0, 0.0, 200.0, 100.0)),
    ],
    0.4,
    Easing::EaseOutCubic,
);

// In animation loop:
layout.update(dt);
if let Some(css) = layout.css_transform("card-1") {
    // Apply transform to DOM element
}
```

### Gesture Recognition

```rust
use spanda::gesture::{GestureRecognizer, Gesture};
use spanda::drag::PointerData;

let mut recognizer = GestureRecognizer::new();

// Feed pointer events
recognizer.on_pointer_down(PointerData { x: 100.0, y: 100.0, pressure: 0.5, pointer_id: 0 });
recognizer.update(0.05);

if let Some(gesture) = recognizer.on_pointer_up(PointerData { x: 400.0, y: 105.0, pressure: 0.0, pointer_id: 0 }) {
    match gesture {
        Gesture::Swipe { direction, velocity, .. } => println!("Swipe {:?} at {} px/s", direction, velocity),
        Gesture::Tap { position } => println!("Tap at {:?}", position),
        _ => {}
    }
}
```

### GPU Batch Animation

```rust,ignore
use spanda::gpu::GpuAnimationBatch;
use spanda::{Tween, Easing};

let mut batch = GpuAnimationBatch::new_auto(); // GPU with CPU fallback
for i in 0..10_000 {
    batch.push(Tween::new(0.0_f32, 1.0).duration(1.0).easing(Easing::EaseOutCubic).build());
}
batch.tick(1.0 / 60.0);
let positions: &[f32] = batch.read_back();
```

## Feature Flags

| Flag       | What it adds                                          |
|------------|-------------------------------------------------------|
| `std`      | *(default)* wall-clock driver, thread-safe internals  |
| `serde`    | `Serialize`/`Deserialize` on all public types         |
| `bevy`     | `SpandaPlugin` for Bevy 0.13                          |
| `wasm`     | `requestAnimationFrame` driver                        |
| `wasm-dom` | DOM plugins: FLIP, SplitText, ScrollSmoother, Draggable, Observer |
| `palette`  | Colour interpolation via the `palette` crate          |
| `tokio`    | `async` / `.await` on timeline completion             |
| `gpu`      | GPU compute shader batch animation via `wgpu`         |

## Bevy Integration

```rust,ignore
use bevy::prelude::*;
use spanda::integrations::bevy::{SpandaPlugin, TweenCompleted, SpringSettled};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SpandaPlugin)
        .run();
}
```

## WASM Integration

```rust,ignore
use spanda::integrations::wasm::RafDriver;

let mut driver = RafDriver::new();
driver.pause();       // pause animations
driver.resume();      // resume
driver.set_time_scale(2.0); // 2x speed
// Call driver.tick(timestamp_ms) from your rAF callback.
```

## Benchmarks

```bash
cargo bench
```

## Tests

```bash
cargo test                # unit + integration + doc tests
cargo test --tests        # integration tests only
```

## Project Structure

```
src/
├── lib.rs           — crate root, re-exports
├── traits.rs        — Interpolate, Animatable, Update
├── easing.rs        — 38 easing functions + CubicBezier + Steps + 5 advanced
├── tween.rs         — Tween<T>, TweenBuilder, TweenState
├── keyframe.rs      — KeyframeTrack, Keyframe, Loop
├── timeline.rs      — Timeline, Sequence, At, stagger
├── spring.rs        — Spring, SpringConfig, SpringN, SpringAnimatable
├── clock.rs         — Clock, WallClock, ManualClock, MockClock
├── driver.rs        — AnimationDriver, AnimationId
├── scroll.rs        — ScrollClock, ScrollDriver
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
    ├── draggable.rs — Draggable DOM binding (feature = "wasm-dom")
    └── observer.rs  — Observer unified input (feature = "wasm-dom")
```

## License

Licensed under either of

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT License](LICENSE-MIT)

at your option.
