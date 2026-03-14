# Springs

Physics-based animation creates motion that feels organic, natural, and inherently interactive. Unlike a [Tween](tween.md), a `Spring` has **no fixed duration**. Instead, it uses a damped harmonic oscillator simulation — the same mathematics that governs a ball on a rubber band.

You set a `target`, and the spring pulls the value toward that target based on its tension and friction (stiffness and damping). The result is motion that **overshoots**, **bounces**, and **settles** — just like real physics.

---

## Creating a Spring

The easiest way to create a spring is using one of the 4 built-in presets:

```rust
use spanda::spring::{Spring, SpringConfig};

let mut spring = Spring::new(SpringConfig::wobbly());
```

### Presets

| Preset | Stiffness | Damping | Character |
|--------|-----------|---------|-----------|
| `gentle()` | 60 | 14 | Slow, smooth — great for background elements |
| `wobbly()` | 180 | 12 | Bouncy, playful — great for interactive UI |
| `stiff()` | 210 | 20 | Fast, minimal bounce — great for snappy responses |
| `slow()` | 37 | 14 | Very relaxed, lazy — great for ambient motion |

### Starting Position

By default, a spring starts at position `0.0`. Use `with_position()` to start elsewhere:

```rust
let mut spring = Spring::new(SpringConfig::gentle())
    .with_position(50.0); // Start at 50
```

---

## Moving the Spring

To animate the spring, change its target. It immediately begins accelerating toward the new destination, **preserving its current velocity**. This is what makes springs perfect for interactive UI elements — you can retarget mid-flight without jarring transitions:

```rust
use spanda::traits::Update;

spring.set_target(100.0);

// In your render loop:
spring.update(dt);
let current_pos = spring.position();
// render(current_pos);

// User clicks elsewhere? Retarget instantly:
spring.set_target(250.0); // velocity carries over smoothly
```

---

## Settle Detection

Because springs never *truly* stop mathematically (they approach the target asymptotically), spanda uses an `epsilon` threshold. Once the spring's **velocity** and **distance** from the target both fall below `epsilon`, the spring is:

1. **Clamped** to the exact target value (no sub-pixel jitter)
2. **Velocity zeroed** out
3. Marked as **settled**

```rust
if spring.is_settled() {
    println!("Spring has stopped moving.");
}
```

The default `epsilon` is `0.001`. For pixel-based animations, you might want a slightly larger value (e.g., `0.5`) to settle faster.

---

## Custom Spring Configs

If the presets don't quite fit, define your own exact physics parameters:

```rust
let custom_config = SpringConfig {
    stiffness: 150.0, // "Tension" — higher = faster pull toward target
    damping: 10.0,    // "Friction" — higher = less bounce, settles faster
    mass: 1.0,        // "Weight" — higher = slower acceleration
    epsilon: 0.001,   // Rest threshold — lower = more precise but slower to settle
};

let spring = Spring::new(custom_config);
```

### Parameter Guide

| Parameter | Low Value | High Value |
|-----------|-----------|------------|
| **Stiffness** | Slow, lazy pull | Snappy, fast pull |
| **Damping** | More bounce, oscillation | Less bounce, direct path |
| **Mass** | Quick to accelerate | Sluggish, heavy feel |
| **Epsilon** | Very precise settling | Faster settling (less precise) |

### Understanding the Physics

The spring uses the **damped harmonic oscillator** equation:

```
acceleration = (-stiffness × displacement - damping × velocity) / mass
```

- **Stiffness** is the "pull force" — how strongly the spring pulls toward the target
- **Damping** is the "friction" — how quickly oscillation dies down
- **Mass** is the "inertia" — how resistant the spring is to acceleration

---

## Sub-Stepping (Stability)

Large `dt` values (e.g., when a browser tab is inactive or a game hitches) can cause springs to "explode" — velocity grows without bound. Spanda prevents this with automatic **sub-stepping**:

- The maximum internal step size is `1/120` seconds (120 Hz)
- If `dt` is larger, it's broken into multiple smaller steps
- This guarantees **numerical stability** even with `dt` spikes of 1+ seconds

You don't need to do anything to enable this — it's always active.

---

## NaN Safety

Springs guard against degenerate configurations:

| Scenario | Behaviour |
|----------|-----------|
| `stiffness = 0.0` | Spring snaps directly to target, no oscillation |
| Negative `dt` | Treated as `0.0` — no backward time |
| Position is `NaN` | *Should not occur* due to sub-stepping. If it does, `debug_assert!` will catch it in debug builds |

---

## Springs vs. Easing-Based Tweens

| | Tween | Spring |
|---|-------|--------|
| **Duration** | Fixed (you specify it) | Dynamic (settles naturally) |
| **Retargeting** | Must reset and create a new tween | Call `.set_target()` mid-flight |
| **Overshoot** | Only with certain easings (Back, Elastic) | Natural and physically correct |
| **Interactivity** | Awkward — "cancel and restart" | Seamless — velocity preserves momentum |
| **Predictability** | Exact timing, exact progress | Approximate (settled within epsilon) |

**Use tweens when**: you need exact timing (e.g., a 0.3s fade-in, a loading bar).

**Use springs when**: you need responsiveness (e.g., a slider thumb, a tooltip following the cursor, drag interactions).

---

## Key Methods

| Method | Description |
|--------|-------------|
| `Spring::new(config)` | Create a new spring at position `0.0` |
| `.with_position(pos)` | Builder — set starting position |
| `.set_target(target)` | Set a new target (spring begins moving immediately) |
| `.position()` | Current position |
| `.velocity()` | Current velocity |
| `.target()` | Current target value |
| `.is_settled()` | Whether the spring has settled within `epsilon` |
| `.reset()` | Reset position and velocity to `0.0` |
| `.update(dt)` | Advance physics by `dt` seconds (returns `false` when settled) |

---

## Bevy Integration

With the `bevy` feature, `Spring` is a Bevy `Component`. The `SpandaPlugin` automatically ticks all Spring components using Bevy's `Time` resource:

```rust
commands.spawn((
    SpriteBundle { /* ... */ },
    Spring::new(SpringConfig::wobbly()),
));
```
