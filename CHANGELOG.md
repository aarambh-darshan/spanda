# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
