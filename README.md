# spanda

*Sanskrit: स्पन्द — vibration, pulse, the throb of motion.*

A general-purpose **animation library** for Rust.  Zero mandatory dependencies,
`no_std`-ready, and designed to work anywhere: terminal UIs, web (WASM),
game engines (Bevy), or native desktop apps.

## ✨ Features

- **Tweening** — animate any value from A to B with 31 built-in easing curves
- **Keyframe tracks** — multi-stop animations with per-segment easing
- **Timeline & Sequence** — compose animations concurrently or sequentially
- **Physics springs** — damped harmonic oscillator with 4 presets
- **Animation driver** — manage multiple animations with auto-cleanup
- **Clock abstraction** — wall clock, manual clock, and mock clock for testing

## 🏗️ Getting Started

```toml
[dependencies]
spanda = "0.1"
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

## 🔌 Feature Flags

| Flag       | What it adds                                          |
|------------|-------------------------------------------------------|
| `std`      | *(default)* wall-clock driver, thread-safe internals  |
| `serde`    | `Serialize`/`Deserialize` on all public types         |
| `bevy`     | `SpandaPlugin` for Bevy 0.13                          |
| `wasm`     | `requestAnimationFrame` driver                        |
| `palette`  | Colour interpolation via the `palette` crate          |
| `tokio`    | `async` / `.await` on timeline completion             |

## 🎮 Bevy Integration

```rust,ignore
use bevy::prelude::*;
use spanda::integrations::bevy::SpandaPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SpandaPlugin)
        .run();
}
```

## 🌐 WASM Integration

```rust,ignore
use spanda::integrations::wasm::RafDriver;

let mut driver = RafDriver::new();
// Call driver.tick(timestamp_ms) from your rAF callback.
```

## 📊 Benchmarks

```bash
cargo bench
```

## 🧪 Tests

```bash
cargo test                # unit + doc tests
cargo test --tests        # integration tests only
```

## 📁 Project Structure

```
src/
├── lib.rs           — crate root, re-exports
├── traits.rs        — Interpolate, Animatable, Update
├── easing.rs        — 31 easing functions + Easing enum
├── tween.rs         — Tween<T>, TweenBuilder, TweenState
├── keyframe.rs      — KeyframeTrack, Keyframe, Loop
├── timeline.rs      — Timeline, Sequence
├── spring.rs        — Spring, SpringConfig
├── clock.rs         — Clock, WallClock, ManualClock, MockClock
├── driver.rs        — AnimationDriver, AnimationId
└── integrations/
    ├── mod.rs
    ├── bevy.rs      — SpandaPlugin  (feature = "bevy")
    └── wasm.rs      — RafDriver     (feature = "wasm")
```

## License

Licensed under the [MIT License](LICENSE).
