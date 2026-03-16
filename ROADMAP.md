# spanda — Roadmap

> *From working prototype → battle-tested → crates.io staple*

---

## Current Status

All core modules are **complete and tested**:

| Module | Status | Tests |
|--------|--------|-------|
| `traits.rs` — `Interpolate`, `Animatable`, `Update` | ✅ Complete | Passing |
| `easing.rs` — 31 easing curves + `Custom` | ✅ Complete | Benchmarked |
| `tween.rs` — `Tween<T>`, `TweenBuilder`, `TweenState` | ✅ Complete | 12 tests |
| `clock.rs` — `WallClock`, `ManualClock`, `MockClock` | ✅ Complete | 4 tests |
| `driver.rs` — `AnimationDriver`, `AnimationDriverArc` | ✅ Complete | 5 tests |
| `keyframe.rs` — `KeyframeTrack`, `Loop` modes | ✅ Complete | 10 tests |
| `timeline.rs` — `Timeline`, `Sequence`, callbacks | ✅ Complete | 8 tests |
| `spring.rs` — `Spring`, `SpringConfig`, 4 presets | ✅ Complete | 8 tests |
| `integrations/bevy.rs` — `SpandaPlugin` | ✅ Written | — |
| `integrations/wasm.rs` — `RafDriver` | ✅ Written | — |
| **Total** | | **60+ unit, 13 doc, 10 integration** |

---

### Stagger Utilities

Automatically offset animations across a collection of targets — no more manual delay math:

```rust
// Instead of manually calculating 10 timeline offsets...
let staggered = spanda::timeline::stagger(animations, 0.1);
// Supports: linear, grid, random, and center-out patterns
```

### Scroll-Linked Driver

A `ScrollDriver` / `ScrollClock` that maps scroll position to animation progress instead of wall time:

```rust
let scroll_driver = ScrollDriver::new(0.0, 1000.0); // scroll range in pixels
// Animation progress = scroll percentage
```

### Relative Timeline Positioning

GSAP-style placement tokens for `Timeline` and `Sequence`:

```rust
timeline.add_at("fade", fade_tween, At::Start);           // absolute start
timeline.add_at("slide", slide_tween, At::End);            // after everything
timeline.add_at("scale", scale_tween, At::Label("fade"));  // start with "fade"
timeline.add_at("glow", glow_tween, At::Offset(0.5));      // 0.5s after previous
```

### Path / Bezier Interpolation

Animate values along curves, not just straight lines:

```rust
// Quadratic and cubic Bezier paths
let path = BezierPath::cubic(start, control1, control2, end);
let tween = Tween::along(path).duration(2.0).build();
```

### `Tween::from()` / `Tween::from_to()`

GSAP-style helpers that capture current state at activation time:

```rust
// Animate FROM a value to the current state
let tween = Tween::from(0.0_f32, current_opacity);

// Animate between two explicit values
let tween = Tween::from_to(start, end);
```

### Full Callback System

`on_start`, `on_update`, and `on_reverse_complete` at the Tween level:

```rust
tween.on_start(|| log::info!("Animation started"));
tween.on_update(|progress| update_ui(progress));
tween.on_complete(|| remove_element());
```

### Repeat / Yoyo on Individual Tweens

Currently only `KeyframeTrack` supports `Loop`. Extend to `Tween` and `Timeline`:

```rust
let tween = Tween::new(0.0_f32, 1.0)
    .duration(0.5)
    .looping(Loop::PingPong) // yoyo on a single tween
    .build();
```

### Time Scale Control

Speed up or slow down individual animations:

```rust
tween.set_time_scale(0.5);    // half speed (slow-mo)
timeline.set_time_scale(2.0); // double speed
```

### Value Modifiers / Snapping

Intercept and transform values before they're read:

```rust
let tween = Tween::new(0.0_f32, 100.0)
    .modifier(|v| (v / 10.0).round() * 10.0) // snap to nearest 10
    .build();
```

---

## Planned Releases

### 0.2.0 — Ergonomics & Missing Primitives ✅

- ✅ Stagger utilities (`spanda::timeline::stagger()`)
- ✅ `Tween::from()` / `Tween::from_to()` helpers
- ✅ Full callbacks on `Tween` (`on_start`, `on_update`, `on_complete`)
- ✅ `Loop` support on individual `Tween`s (repeat, yoyo)
- ✅ Time scale control on `Tween` and `Timeline`
- ✅ Value modifiers / snapping utilities (`snap_to`, `round_to`)
- ✅ API refinements from real-world Leptos usage (`on_update` receives value, stagger example)

### 0.3.0 — Scroll & Motion Paths ✅

- ✅ `ScrollDriver` / `ScrollClock` (scroll-linked animations)
- ✅ Relative timeline positioning (`At::Start`, `At::End`, `At::Label`, `At::Offset`)
- ✅ Quadratic and cubic Bezier path interpolation
- ✅ `MotionPath` type for complex curves

### 0.4.0 — Full Motion Path System (GSAP MotionPathPlugin Equivalent) ✅

- ✅ `CatmullRomSpline` — smooth curve through points with tension control
- ✅ `PathEvaluate2D` trait — evaluate position + tangent at progress t
- ✅ `PolyPath` — smooth path through `[f32; 2]` arrays with arc-length parameterization
- ✅ `CompoundPath` — multi-segment path from SVG-style commands (M/L/Q/C/Z)
- ✅ `SvgPathParser::parse(d_str)` — zero-dependency SVG `d` attribute string parser
- ✅ Arc-length parameterization (256-sample LUT) for constant-speed traversal
- ✅ Auto-rotate: `tangent_angle()` / `rotation()` / `rotation_deg()`
- ✅ Start/end offsets for partial path traversal
- ✅ Rotation offset for auto-rotate adjustments
- ✅ CatmullRom `.tension()` parameter (GSAP `curviness` equivalent)
- ✅ `Easing::CubicBezier(x1, y1, x2, y2)` — CSS cubic-bezier() equivalent
- ✅ `Easing::Steps(n)` — CSS steps() equivalent

### 0.5.0 — Spring Generics & Bevy Polish ✅

- ✅ `SpringN<T: SpringAnimatable>` — generic springs for 2D/3D/4D physics
- ✅ `SpringAnimatable` trait — component array approach (to_components/from_components)
- ✅ Bevy `SpandaPlugin` enhanced with `SpringSettled` event
- ✅ `AnimationLabel` component for named animations in Bevy
- ✅ `TweenCompleted` event system working (was already in 0.1.0)
- ✅ Ship `examples/bevy_bounce.rs`

### 0.6.0 — WASM & Web Polish ✅

- ✅ `RafDriver` enhanced: pause/resume, time scale, visibility handling, dt capping
- ✅ `start_raf_loop(callback)` — self-scheduling rAF loop helper
- ✅ Ship `examples/wasm_tween/` project (Cargo.toml + lib.rs + index.html)
- ✅ Leptos integration guide with working examples (`docs/leptos_guide.md`)
- ✅ Dioxus integration guide with working examples (`docs/dioxus_guide.md`)

### 0.7.0 — Colour & Advanced Interpolation ✅

- ✅ `Interpolate` impl for 9 `palette` colour types (Srgba, Srgb, LinSrgba, LinSrgb, Laba, Lab, Oklcha, Oklch, Hsla)
- ✅ Colour space-aware interpolation (sRGB, Lab, Oklch, linear RGB) via `InLab`, `InOklch`, `InLinear` wrappers
- ✅ Shortest-arc hue interpolation for hue-based types
- ✅ `SpringAnimatable` impls for palette colour types
- ✅ Convenience functions: `lerp_in_lab`, `lerp_in_oklch`, `lerp_in_linear`
- ✅ Ship colour animation example (`examples/colour_demo.rs`)

### 1.0.0 — Stable

- No breaking API changes for at least one minor version cycle
- All examples compile and run
- Full docs.rs coverage
- CI via GitHub Actions
- Published to crates.io

---

## Version History

| Version | Date | Highlights |
|---------|------|------------|
| `0.1.0` | TBD | Core complete — tweening, keyframes, timelines, springs, driver, clock, Bevy/WASM integrations |
| `0.2.0` | March 2026 | Stagger, tween looping, time scale, callbacks, value modifiers, Leptos ergonomics |
| `0.3.0` | March 2026 | ScrollDriver/ScrollClock, relative timeline positioning (At), Bezier paths, MotionPath |
| `0.4.0` | March 2026 | Full Motion Path System — CatmullRom, PolyPath, CompoundPath, SvgPathParser, arc-length, CSS cubic-bezier/steps easing |
| `0.5.0` | March 2026 | SpringN<T> generic springs, SpringSettled event, AnimationLabel, bevy_bounce example |
| `0.6.0` | March 2026 | WASM RafDriver enhancements (pause/resume/visibility), start_raf_loop, Leptos & Dioxus guides |
| `0.7.0` | March 2026 | Colour interpolation — 9 palette types, InLab/InOklch/InLinear wrappers, SpringAnimatable, colour_demo |
| `0.8.0` | March 2026 | GSAP-class features — DrawSVG, MorphPath, Inertia, 5 advanced easings, DragState, WASM-DOM plugins (FLIP, SplitText, ScrollSmoother, Draggable, Observer) |

---

*Roadmap version: 2.0 — Aarambh Dev Hub / spanda*
*Updated: March 2026*
