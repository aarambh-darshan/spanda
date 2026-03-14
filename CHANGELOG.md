# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] — 2026-03-14

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

## [0.2.0] — 2026-03-14

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

## [0.1.0] — 2026-03-13

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
