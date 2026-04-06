# Feature Matrix

> Which modules and features are available under which feature flags.

---

## Feature Flags

| Flag | Dependencies | Implies |
|------|-------------|---------|
| `std` *(default)* | — | — |
| `serde` | `serde` | — |
| `bevy` | `bevy_app`, `bevy_ecs`, `bevy_time` | `std` |
| `wasm` | `wasm-bindgen`, `js-sys` | `std` |
| `wasm-dom` | `web-sys` | `wasm` |
| `palette` | `palette` | — |
| `tokio` | `tokio` | `std` |

---

## Module Availability

**Legend**: **Y** = available, **req** = this feature flag is required, blank = not affected by this flag

| Module / Feature | no\_std | std | serde | bevy | wasm | wasm-dom | palette | tokio |
|-----------------|---------|-----|-------|------|------|----------|---------|-------|
| **Core** | | | | | | | | |
| `traits` (Interpolate, Animatable, Update) | Y | Y | | | | | | |
| `easing` (31 standard + Custom) | Y | Y | derives | | | | | |
| `easing` (CubicBezier, Steps) | Y | Y | | | | | | |
| `easing` (5 advanced parametric) | Y | Y | | | | | | |
| `tween` (Tween\<T\>, TweenBuilder) | Y | Y | | | | | | |
| `tween` (callbacks: on\_start/update/complete) | | Y | | | | | | |
| `keyframe` (KeyframeTrack, Loop) | Y | Y | | | | | | |
| `timeline` (Timeline, Sequence, At, stagger) | Y | Y | | | | | | |
| **Physics** | | | | | | | | |
| `spring` (Spring, SpringConfig) | Y | Y | | | | | | |
| `spring` (SpringN\<T\>, SpringAnimatable) | Y | Y | | | | | | |
| `inertia` (Inertia, InertiaN\<T\>) | Y | Y | | | | | | |
| `drag` (DragState, DragConstraints) | Y | Y | | | | | | |
| **Paths** | | | | | | | | |
| `path` (BezierPath, MotionPath) | Y | Y | | | | | | |
| `bezier` (CatmullRomSpline, PathEvaluate2D) | Y | Y | | | | | | |
| `motion_path` (PolyPath, CompoundPath) | Y | Y | | | | | | |
| `svg_path` (SvgPathParser) | Y | Y | | | | | | |
| **Effects** | | | | | | | | |
| `svg_draw` (draw\_on, draw\_on\_reverse) | Y | Y | | | | | | |
| `morph` (MorphPath, resample) | Y | Y | | | | | | |
| **Infrastructure** | | | | | | | | |
| `clock` (ManualClock, MockClock) | Y | Y | | | | | | |
| `clock` (WallClock) | | **req** | | | | | | |
| `driver` (AnimationDriver) | Y | Y | | | | | | |
| `driver` (AnimationDriverArc) | | **req** | | | | | | |
| `scroll` (ScrollClock, ScrollDriver) | Y | Y | | | | | | |
| `scroll_smooth` (SmoothScroll1D) | Y | Y | | | | | | |
| **Colour** | | | | | | | | |
| `colour` (Interpolate for 9 palette types) | | | | | | | **req** | |
| `colour` (InLab, InOklch, InLinear wrappers) | | | | | | | **req** | |
| `colour` (SpringAnimatable for palette types) | | | | | | | **req** | |
| **Integrations** | | | | | | | | |
| `integrations::bevy` (SpandaPlugin) | | | | **req** | | | | |
| `integrations::wasm` (RafDriver) | | | | | **req** | | | |
| `integrations::split_text` (core splitting) | Y | Y | | | | | | |
| `integrations::split_text` (DOM injection) | | | | | | **req** | | |
| `integrations::flip` (FlipState, FlipAnimation) | | | | | | **req** | | |
| `integrations::scroll_smoother` | | | | | | **req** | | |
| `integrations::smooth_scroll` (SmoothScroll) | | | | | | **req** | | |
| `integrations::draggable` | | | | | | **req** | | |
| `integrations::observer` | | | | | | **req** | | |

---

## Feature-Gated Behaviours Within Modules

Some modules are always compiled but have specific behaviours that require feature flags:

| Behaviour | Required Feature | Notes |
|-----------|-----------------|-------|
| Tween callbacks (`on_start`, `on_update`, `on_complete`) | `std` | Uses `Box<dyn Fn>`. Also excluded when `bevy` feature is active. |
| `WallClock` | `std` | Uses `std::time::Instant` |
| `AnimationDriverArc` | `std` | Uses `Arc<Mutex<>>` |
| Serde derives on `Easing` and other types | `serde` | `Easing::Custom` is `#[serde(skip)]` |
| SplitText DOM methods (`inject_chars`, `inject_words`, `detect_lines`) | `wasm-dom` | Core string splitting always available |
| `FlipState::capture(element)` | `wasm-dom` | `FlipState::from_rect()` works without it (but module is gated) |
| Async timeline completion (`.wait().await`) | `tokio` | Uses `tokio::sync::watch` |

---

## Recommended Combinations

| You are building... | Features | Why |
|---------------------|----------|-----|
| A TUI app | `default` (just `std`) | `WallClock` for real-time frame loop |
| A Bevy game | `bevy` | Auto-ticks components via `SpandaPlugin` |
| A Leptos/Yew web app | `wasm` | `RafDriver` for `requestAnimationFrame` |
| A web app with DOM interaction | `wasm-dom` | FLIP, SplitText, Draggable, ScrollSmoother, SmoothScroll |
| A CLI tool | `default` | Standard `WallClock` + `AnimationDriver` |
| Embedded / `no_std` | `default-features = false` | Pure math, zero OS dependencies |
| Colour animations | `palette` | `Interpolate` impl for palette colour types |
| State persistence | `serde` | Serialize/deserialize all animation types |
| Async workflows | `tokio` | `.await` on timeline completion |
| Full everything | `std,serde,bevy,wasm,wasm-dom,palette,tokio` | All features enabled |
