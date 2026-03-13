# Spanda Documentation

Welcome to the `spanda` documentation! This library is built around a single, powerful idea: **any value that can be linearly interpolated can be animated.**

Everything else — easing curves, keyframe tracks, timelines, physics springs — is layered on top of that one primitive.

## Core Concepts

- **[Tweens (`Tween<T>`)](tween.md)**: Animate a value from point A to point B over a specific duration.
- **[Easings (`Easing`)](easing.md)**: 31 built-in standard easing curves to give your animations character.
- **[Keyframes (`KeyframeTrack<T>`)](keyframe.md)**: Multi-stop animations that pass through an arbitrary number of points.
- **[Timelines & Sequences](timeline.md)**: Compose multiple animations to run concurrently or sequentially.
- **[Springs (`Spring`)](spring.md)**: Physics-based animations that create natural, organic motion without a fixed duration.

## Integrations

Spanda is designed to be completely decoupled from any rendering or main loop, making it easy to plug into any ecosystem:

- **[Bevy Plugin](integrations.md#bevy-plugin)**
- **[WASM & Web](integrations.md#wasm--web)**
- **[Terminal / CLI](integrations.md#tui--cli)**

## Getting Started

Add `spanda` to your `Cargo.toml`:

```toml
[dependencies]
spanda = "0.1"
```

The simplest animation is a `Tween`. You build it, provide it "ticks" of time (delta `dt`), and read the value:

```rust
use spanda::{Tween, Easing};
use spanda::traits::Update;

let mut tween = Tween::new(0.0_f32, 100.0)
    .duration(1.0)
    .easing(Easing::EaseOutBounce)
    .build();

// Inside your rendering loop:
fn on_frame(dt: f32) {
    tween.update(dt);
    let current_x = tween.value();
    // draw(current_x);
}
```
