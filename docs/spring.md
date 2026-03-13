# Springs

Physics-based animation creates motion that feels organic, natural, and inherently interactive. Unlike a [Tween](tween.md), a `Spring` has **no fixed duration**. Instead, it uses a damped harmonic oscillator simulation.

You set a `target`, and the spring pulls the value toward that target based on its tension and friction (stiffness and damping).

## Creating a Spring

The easiest way to create a spring is using one of the 4 built-in presets:

```rust
use spanda::spring::{Spring, SpringConfig};

// Presets: gentle(), wobbly(), stiff(), slow()
let config = SpringConfig::wobbly();
let mut spring = Spring::new(config);
```

By default, a spring starts at position `0.0`. You can change this using `with_position`:

```rust
let mut spring = Spring::new(SpringConfig::gentle())
    .with_position(50.0); // Start at 50
```

## Moving the Spring

To animate the spring, you change its target. It will immediately begin accelerating toward the new destination, preserving its current velocity. This is what makes springs perfect for interactive UI elements.

```rust
spring.set_target(100.0);

// In your loop:
spring.update(dt);
let current_pos = spring.position();
```

### Checking for Completion

Because springs never *truly* stop mathematically, `spanda` uses an `epsilon` value. Once the spring's velocity and distance from the target both fall below `epsilon`, the spring is clamped to the target and considered "settled".

```rust
if spring.is_settled() {
    println!("Spring has stopped moving.");
}
```

## Custom Spring Configs

If the presets don't quite fit, you can define your own exact physics parameters:

```rust
let custom_config = SpringConfig {
    stiffness: 150.0, // "Tension" — higher = faster pull
    damping: 10.0,    // "Friction" — higher = less bounce
    mass: 1.0,        // "Weight" — higher = slower acceleration
    epsilon: 0.001,   // Rest threshold
};

let spring = Spring::new(custom_config);
```
