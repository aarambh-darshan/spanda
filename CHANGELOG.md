# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [0.9.3]

### Added

- **Smooth window scroll** (`wasm-dom`):
  - [`SmoothScroll1D`](src/scroll_smooth.rs) — frame-rate independent exponential decay toward a target scroll offset (pure math, unit-tested).
  - [`integrations::smooth_scroll::SmoothScroll`](src/integrations/smooth_scroll.rs) — wheel, keyboard, touch + `InertiaN<[f32; 1]>` fling, `resize` / `hashchange` / anchor `click`, `prefers-reduced-motion`; applies scroll only via `Window::scroll_to_with_x_and_y`; sets `touch-action` / `overscroll-behavior` on `<html>` while active.
  - [`SpringAnimatable` for `[f32; 1]`](src/spring.rs) — used by 1D `InertiaN` in touch momentum.

## [0.9.2]

### Added

- **Tween Enhancements**:
  - `Tween::set()` — immediately set a value without animation (GSAP `gsap.set()` equivalent)
  - `on_repeat()` callback — fires each time a looping tween repeats
  - `on_reverse_complete()` callback — fires when ping-pong tweens reverse

- **Timeline Enhancements**:
  - `Timeline::call()` — insert callback functions at specific points in timeline
  - `Timeline::add_pause()` — insert pause points in timeline sequence
  - `total_duration()` / `total_progress()` — query overall timeline metrics
  - `get_entries_by_label()` — retrieve timeline entries by label name

- **ScrollDriver Enhancements**:
  - `on_enter`, `on_leave`, `on_enter_back`, `on_leave_back` callbacks — scroll position event hooks
  - `snap_points` — define snap positions for scroll-linked animations
  - `nearest_snap_point()` — find closest snap point to current scroll position

- **DragState Enhancements**:
  - `on_drag_start`, `on_drag_end`, `on_click`, `on_throw_update` callbacks
  - `snap_on_release` — automatic snapping when drag ends
  - Click vs drag detection via `click_threshold`

- **Color Parsing** (`colour` module, `palette` feature):
  - `parse_hex()` — parse hex color strings (#RGB, #RGBA, #RRGGBB, #RRGGBBAA)
  - `parse_named()` — parse CSS named colors (red, blue, transparent, etc.)
  - `parse_color()` — auto-detect and parse either format

- **MotionPath Enhancements**:
  - `get_relative_position()` — find progress along path for any world point
  - `closest_point()` — find nearest point on path with distance

- **MorphPath Enhancements**:
  - `ShapeIndex` enum — control point correspondence during shape morphing
  - `ShapeIndex::auto()` — automatic rotation optimization for smoother morphs
  - `shape_index()` builder method

- **Observer Enhancements** (`wasm-dom` feature):
  - `ObserverOptions` — tolerance, preventDefault, allowClicks, capture, lockAxis
  - `bind_with_options()` — configurable event observation

- **SplitText Enhancements**:
  - `SplitTextOptions` — word_delimiter, chars_class, words_class, lines_class
  - `from_str_with_options()` — split with custom configuration
  - `rebuild()` — re-split with new options

- **FLIP Animation Callbacks**:
  - `on_enter`, `on_leave`, `on_complete` callbacks for FlipAnimation

### Fixed & Optimized (v0.9.2 Stabilization)

- **Tween / Timeline API Polish**:
  - Fixed `PingPong` directionality test logic (refactored to use `forward` flag instead of swapping `start`/`end` fields, avoiding history mutation).
  - Fixed `Timeline::add` positioning logic for 0-duration tweens via new `add_with_duration` method.
  - Re-typed `KeyframeTrack::value()` and `value_at()` to return `Option<T>` instead of panicking on empty tracks.

- **Optimizations & Refactors**:
  - Optimized `SpringN` to cache `target_components`, completely avoiding per-frame `Vec` allocation in the `Update` trait's hot path.
  - Refactored `Easing::Custom`'s `PartialEq` implementation to always gracefully return `false`, mitigating cross-compilation function pointer identity issues.
  - Expanded `Easing::all_named()` to include 7 representative parameterized variants, and introduced `Easing::all_classic()` for the 31 classic names.
  - Stripped redundant `std::vec::Vec` and extra feature-gated `no_std` imports across the codebase.
  - Inserted missing `#[derive(Debug)]` on Bevy integration types (`SpandaPlugin`, `TweenCompleted`, `SpringSettled`).
  - Implemented `#[inline]` markings on fast-path functions (`progress()`, `is_complete()`, `apply()`) to boost runtime performance.
  - Updated to use **Rust 2024 edition** and bumped `rust-version` to `1.85`.

### Dependencies

- Updated **Bevy** from `0.13` to `0.18` (migrated to the new queue-based `Message` API).

## [0.9.1]

### Fixed
- Added missing `HtmlCollection` feature to `web-sys` dependency
  (required for `wasm-dom` plugin DOM traversal)

## [0.9.0]

### Added

- **GPU Compute Shaders** (`gpu` module, `gpu` feature):
  - `GpuAnimationBatch` — batch evaluate thousands of `f32` tweens on the GPU
  - `GpuContext` — shared wgpu `Device` and `Queue`
  - `try_create_gpu_context()` — auto-detect GPU adapter
  - `new_auto()` — GPU with automatic CPU fallback
  - `new_cpu_fallback()` — same API, CPU-only
  - WGSL compute shader with 10 core easing functions
  - Automatic buffer resizing for growing batches
  - Example: `examples/gpu_batch_demo.rs`
- **Layout Animation** (`layout` module):
  - `Rect` — captured element bounding rect
  - `LayoutAnimator` — track elements by ID, auto-generate FLIP animations
  - `LayoutAnimation` — translate + scale tween bundle
  - `LayoutTransition` — element ID + animation
  - `SharedElementTransition` — cross-view hero transitions
  - `animate_reorder()` — batch list reorder with FLIP
  - `animate_enter()` / `animate_exit()` — addition/removal animations
  - `css_transform()` — ready-to-use CSS transform strings
  - DOM binding via `wasm-dom` feature (`Rect::from_element`, `track_element`)
- **Gesture Recognition** (`gesture` module):
  - `GestureRecognizer` — platform-agnostic gesture detection
  - `Gesture` enum: `Tap`, `LongPress`, `Swipe`, `Pinch`, `Rotate`
  - `SwipeDirection` — cardinal direction (Up, Down, Left, Right)
  - `GestureConfig` — configurable detection thresholds
  - Callback support: `on_gesture()` (std feature)
  - Multi-touch: simultaneous pinch and rotation detection
  - Example: `examples/gesture_demo.rs`

### Dependencies

- `wgpu` 24 (optional, `gpu` feature)
- `pollster` 0.4 (optional, `gpu` feature)
- `bytemuck` 1 + `bytemuck_derive` 1 (optional, `gpu` feature)

## [0.8.0]

### Added

- **Advanced easing variants** (5 new parameterized easings):
  - `RoughEase { strength, points, seed }` — deterministic noise overlay
  - `SlowMo { ratio, power, yoyo_mode }` — slow-fast-slow piecewise curve
  - `ExpoScale { start_scale, end_scale }` — perceptual scale correction
  - `Wiggle { frequency, amplitude }` — sinusoidal oscillation
  - `CustomBounce { strength, squash }` — parametric bounce
- **DrawSVG helper** (`svg_draw` module):
  - `draw_on(path_length)` — tween for stroke-dashoffset draw-on effect
  - `draw_on_reverse(path_length)` — reverse draw-off effect
- **MorphPath** (`morph` module):
  - `MorphPath::new(from, to).duration(d).easing(e).build()` — shape morphing
  - `resample(points, target_count)` — arc-length polyline resampling
  - Auto-resamples mismatched point counts
- **Inertia physics** (`inertia` module):
  - `Inertia` — single-axis friction deceleration (no target, just coasts to stop)
  - `InertiaN<T>` — multi-dimensional inertia via `SpringAnimatable`
  - `InertiaConfig` with presets: `default_flick()`, `heavy()`, `snappy()`
  - Frame-rate independent exponential decay
- **DragState** (`drag` module):
  - `DragState` — pure-math pointer drag tracker with velocity EMA
  - `DragConstraints` — bounds, axis lock, grid snapping
  - `PointerData` — unified pointer/mouse/touch data struct
  - `on_pointer_up()` returns `InertiaN<[f32;2]>` for momentum throw
- **WASM-DOM plugins** (`wasm-dom` feature):
  - `Observer` — unified pointer/touch/mouse event normaliser
  - `FlipState` / `FlipAnimation` — FLIP animation technique
  - `SplitText` — character/word splitting + staggered timelines + DOM injection
  - `ScrollSmoother` — spring-driven smooth scroll interception
  - `Draggable` — DOM-bound drag with pointer event listeners
- New feature flag: `wasm-dom` (enables `wasm` + `web-sys` DOM plugins)
- `#[non_exhaustive]` on `Easing` enum (future-proofing)
- New examples: `morph_demo`, `inertia_demo`
- New integration test: `tests/v080_pure.rs` (10 tests)

## [0.7.0]

### Added

- **Colour interpolation** (`palette` feature):
  - `Interpolate` impls for 9 palette types: `Srgba`, `Srgb`, `LinSrgba`,
    `LinSrgb`, `Laba`, `Lab`, `Oklcha`, `Oklch`, `Hsla`
  - Shortest-arc hue interpolation for hue-based types (Oklch, Hsl)
  - `SpringAnimatable` impls for palette colour types (use with `SpringN`)
- **Colour-space-aware wrappers** — interpolate in a perceptual space while
  keeping sRGB start/end values:
  - `InLab(Srgba)` — CIE L\*a\*b\* for perceptually smooth gradients
  - `InOklch(Srgba)` — OKLCh for smooth hue rotation
  - `InLinear(Srgba)` — linear RGB for physically correct blending
  - All wrappers work with `Tween<T>`, `KeyframeTrack<T>`, `SpringN<T>`
- **Convenience functions**: `lerp_in_lab()`, `lerp_in_oklch()`, `lerp_in_linear()`
- New module: `src/colour.rs` (gated behind `palette` feature)
- New example: `examples/colour_demo.rs` — terminal gradient comparison
- New integration test: `tests/colour_interpolation.rs` (7 tests)
- New documentation: `docs/colour.md`

## [0.6.0]

### Added

- **WASM RafDriver enhancements**:
  - `pause()` / `resume()` / `is_paused()` — pause/resume animation playback
  - `set_time_scale(scale)` / `get_time_scale()` — global speed control
  - `on_visibility_change(hidden)` — handle page visibility changes (resets
    timestamp to avoid jumps after tab switch)
  - Automatic dt capping at 500ms to prevent huge jumps after tab inactivity
  - `driver()` / `driver_mut()` — access the inner `AnimationDriver`
- **`start_raf_loop(callback)`** (`wasm` feature): Self-scheduling
  `requestAnimationFrame` loop that calls a closure every frame. No more
  manual rAF management in JavaScript.
- **`examples/wasm_tween/`**: Complete WASM example project with `Cargo.toml`,
  `src/lib.rs`, and `index.html`. Shows how to build an animated web app
  with `wasm-pack build --target web`.
- **Leptos integration guide** (`docs/leptos_guide.md`): Complete guide with
  4 patterns — basic fade, staggered list, spring-driven drag, RafDriver
- **Dioxus integration guide** (`docs/dioxus_guide.md`): Complete guide with
  4 patterns — basic fade, staggered cards, spring follower, RafDriver
- Updated `docs/integrations.md` with new WASM features, Leptos/Dioxus
  cross-references, and Dioxus 0.5 API examples

## [0.5.0]

### Added

- **`SpringN<T: SpringAnimatable>`** — generic multi-dimensional spring that
  internally manages one position+velocity pair per component. Same physics
  as `Spring` (semi-implicit Euler, sub-stepping, epsilon settle detection).
  Built-in support for `f32`, `[f32; 2]`, `[f32; 3]`, `[f32; 4]`.
- **`SpringAnimatable` trait** — decompose/reconstruct types as `Vec<f32>`
  component arrays. Implement on custom types to use them with `SpringN`.
- **`SpringSettled` event** (Bevy): Fired when a `Spring` component settles
  to its target. Complements the existing `TweenCompleted` event.
- **`AnimationLabel` component** (Bevy): Optional label for identifying
  animations by name in event handlers.
- Updated `SpandaPlugin` to fire `SpringSettled` events (was only tracking
  `TweenCompleted` before).
- `SpringN`, `SpringAnimatable` re-exported from `lib.rs`
- **`examples/bevy_bounce.rs`**: Demonstrates `SpandaPlugin`, `Spring`,
  `SpringN`, `TweenCompleted`, and `SpringSettled` events
- New integration test: `tests/spring_generic.rs` (7 tests) covering
  2D/3D/4D springs, retarget mid-flight, overshoot, f32 parity
- Updated `docs/spring.md` with `SpringN` documentation, custom types,
  and `SpringSettled` event guide
- 10 new unit tests for `SpringN` in `src/spring.rs`

### Added

- **Full Motion Path System** — GSAP MotionPathPlugin equivalent, all outputting
  raw `[f32; 2]` values:
  - **CatmullRomSpline** (`bezier.rs`): Smooth curve through ordered 2D points
    with configurable `tension` parameter (0.0 = straight, 0.5 = standard,
    >1.0 = exaggerated curvature like GSAP's `curviness`). Automatically
    converts to cubic Bezier segments for evaluation.
  - **PathEvaluate2D trait** (`bezier.rs`): Evaluate position and tangent on
    any 2D curve at progress `t ∈ [0.0, 1.0]`.
  - **tangent_angle / tangent_angle_deg** (`bezier.rs`): Compute auto-rotation
    angle from tangent vectors. Equivalent to GSAP's `autoRotate: true`.
  - **PolyPath** (`motion_path.rs`): Smooth path through `[f32; 2]` point arrays.
    Uses CatmullRomSpline internally + arc-length parameterization (256-sample
    LUT) for constant-speed motion. Supports `start_offset`, `end_offset`,
    `rotation_offset`, and custom tension.
  - **CompoundPath** (`motion_path.rs`): Multi-segment path from SVG-style
    commands (MoveTo, LineTo, QuadTo, CubicTo, Close). Arc-length parameterized
    with the same offset and auto-rotate API as PolyPath.
  - **PathCommand enum** (`motion_path.rs`): SVG-style path commands for
    building CompoundPaths programmatically.
  - **SvgPathParser** (`svg_path.rs`): Zero-dependency SVG `d` attribute parser
    supporting M/L/H/V/Q/C/Z commands (absolute and relative). Parses strings
    like `"M 0 0 C 50 100 100 100 150 0 L 200 0"` into `Vec<PathCommand>`.
    Handles compact notation, negative coordinates, implicit LineTo after MoveTo.
  - **Easing::CubicBezier(x1, y1, x2, y2)**: CSS `cubic-bezier()` equivalent
    using Newton-Raphson iteration (8 steps) with bisection fallback (20 steps).
    Same algorithm used by browsers.
  - **Easing::Steps(n)**: CSS `steps()` equivalent for discrete step easing.
- New modules: `src/bezier.rs` (10 unit tests), `src/motion_path.rs` (15 unit
  tests), `src/svg_path.rs` (12 unit tests)
- New integration test suite: `tests/motion_path_full.rs` (14 tests) covering
  CatmullRom, PolyPath, CompoundPath, SvgPathParser, and new easing variants
- Re-exports in `lib.rs`: `CatmullRomSpline`, `PathEvaluate2D`, `tangent_angle`,
  `tangent_angle_deg`, `PolyPath`, `CompoundPath`, `PathCommand`, `SvgPathParser`

### Changed

- `Easing` enum extended with `CubicBezier` and `Steps` variants
- Updated `Easing::apply()`, `Debug`, `PartialEq`, and `name()` for new variants

## [0.3.0]

### Added

- **ScrollDriver / ScrollClock**: Scroll-linked animation driver that maps a
  position value (e.g. scroll offset in pixels) to animation progress instead
  of wall time. `ScrollClock` implements the `Clock` trait for manual use.
  `ScrollDriver` wraps a `ScrollClock` and manages a collection of animations,
  auto-removing completed ones.
- **Relative timeline positioning (`At` enum)**: GSAP-style placement tokens
  for `Timeline::add_at()`:
  - `At::Start` — place at absolute t=0
  - `At::End` — place after the latest-ending entry
  - `At::Label("name")` — sync with a named entry's start time
  - `At::Offset(f32)` — gap or overlap relative to the previous entry's end
- **Bezier path interpolation**: `BezierPath<T>` supports linear, quadratic,
  and cubic Bezier curves via De Casteljau's algorithm. Works with any type
  implementing `Interpolate + Clone`.
- **MotionPath**: Multi-segment path composed of Bezier curves with weighted
  segments. `MotionPathTween` implements `Update` so it works with timelines,
  drivers, and sequences like a regular Tween.
- **PathEvaluate trait**: Common interface for evaluating any curve at progress
  `t ∈ [0.0, 1.0]`.
- **AnimationId::new()**: Public constructor on `AnimationId` for use by
  custom drivers (e.g. `ScrollDriver`).
- New module: `src/scroll.rs` (9 unit tests)
- New module: `src/path.rs` (15 unit tests)
- New integration tests: `tests/scroll_driver.rs` (5 tests),
  `tests/motion_path.rs` (6 tests), `tests/relative_positioning.rs` (3 tests)
- 6 new unit tests for `At` positioning in `src/timeline.rs`
- Documentation: `docs/scroll.md`, `docs/path.md`, updated `docs/integrations.md`
  with scroll and motion path sections

### Changed

- `timeline::At` re-exported from `lib.rs`
- `scroll::ScrollClock` and `scroll::ScrollDriver` re-exported from `lib.rs`
- `path::BezierPath`, `path::MotionPath`, `path::MotionPathTween`, and
  `path::PathEvaluate` re-exported from `lib.rs`

## [0.2.0]

### Added

- **Stagger utilities**: `spanda::timeline::stagger()` creates a `Timeline`
  where each animation starts `stagger_delay` seconds after the previous one.
  Equivalent of GSAP's stagger property.
- **`Tween::from()` / `Tween::from_to()`**: Convenience aliases for
  `Tween::new()` — matches common animation library vocabulary.
- **Full callbacks on `Tween`** (`std` feature, excluded from `bevy`):
  - `on_start(callback)` — fires when state transitions to Running
  - `on_update(callback)` — fires every frame with the interpolated `T` value
  - `on_complete(callback)` — fires when state transitions to Completed
  - Designed for reactive frameworks: `tween.on_update(move |val| set_signal.set(val))`
- **Loop support on individual `Tween`s**: Reuses the existing `keyframe::Loop`
  enum. Supports `Loop::Once` (default), `Loop::Times(n)`, `Loop::Forever`,
  and `Loop::PingPong` (reverses start/end each cycle).
- **Time scale control**: `set_time_scale(scale)` / `time_scale()` on both
  `Tween` and `Timeline`. Values > 1.0 speed up, < 1.0 slow down, 0.0
  effectively pauses. Available via builder (`.time_scale(2.0)`) and at
  runtime (`.set_time_scale(0.5)`).
- **Value modifiers / snapping**: Post-interpolation value transformation.
  `set_modifier(fn)` on Tween applies a pure function after interpolation.
  Utility functions: `snap_to(grid)` and `round_to(decimals)` return closures
  suitable for use as modifiers.
- **Leptos integration pattern**: Updated `docs/integrations.md` with
  `on_update` callback example and stagger-in-Leptos example.
- New integration tests: `tests/tween_looping.rs` (4 tests),
  `tests/stagger_timeline.rs` (2 tests)
- 27+ new unit tests for time scale, looping, callbacks, modifiers, and stagger

### Changed

- `Tween<T>` now has additional fields: `time_scale`, `looping`, `loop_count`,
  `forward`, `started`, and cfg-gated callback/modifier fields
- `Timeline` now has `time_scale` field, applied to `dt` in `update()`
- `lib.rs` re-exports updated: `snap_to`, `round_to`, `stagger`

## [0.1.0]

### Added

- **Core traits**: `Interpolate`, `Animatable`, `Update` with blanket impls for
  `f32`, `f64`, `i32`, `[f32; 2..4]`, and `(f32, f32, f32, f32)` (RGBA)
- **Easing**: 31 built-in easing functions + `Easing` enum with `.apply()`,
  `.name()`, and `Custom(fn(f32) -> f32)` variant
- **Tween**: `Tween<T>` with builder pattern, delay, seek, reverse, pause/resume
- **KeyframeTrack**: Multi-stop animation with `Loop::Once`, `Times(n)`,
  `Forever`, and `PingPong` modes
- **Timeline**: Concurrent animation composition with staggered offsets
- **Sequence**: Sequential animation chaining with gaps
- **Spring**: Damped harmonic oscillator with 4 presets (gentle, wobbly, stiff, slow)
- **Clock**: `Clock` trait + `WallClock`, `ManualClock`, `MockClock`
- **AnimationDriver**: Manages multiple animations with auto-removal of completed ones
- **AnimationDriverArc**: Thread-safe driver wrapper (`std` feature)
- **Bevy integration**: `SpandaPlugin` with `TweenCompleted` event (`bevy` feature)
- **WASM integration**: `RafDriver` for `requestAnimationFrame` loops (`wasm` feature)
- **Examples**: `tui_progress`, `tui_spinner`, `spring_demo`
- **Benchmarks**: Criterion benchmarks for easing functions
- **Integration tests**: 4 test suites covering full lifecycles
