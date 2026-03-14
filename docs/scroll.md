# Scroll-Linked Animations

Traditional animations are driven by wall-clock time — they play at a fixed rate regardless of user interaction. **Scroll-linked animations** instead tie animation progress to scroll position, so elements animate as the user scrolls through content.

Spanda provides two types for this:

- **ScrollClock** — a `Clock` implementation that converts scroll position changes into delta-time values
- **ScrollDriver** — a full driver (like `AnimationDriver`) that manages animations and ticks them based on scroll position

---

## ScrollClock

`ScrollClock` implements the `Clock` trait. It maps a scroll range (e.g. 0 to 1000 pixels) to normalised progress (0.0 to 1.0). Position changes are accumulated and returned by `delta()`.

```rust
use spanda::scroll::ScrollClock;
use spanda::clock::Clock;
use spanda::tween::Tween;
use spanda::traits::Update;

// Map scroll offset 0..500px to animation progress 0..1
let mut clock = ScrollClock::new(0.0, 500.0);
let mut tween = Tween::new(0.0_f32, 100.0).duration(1.0).build();

// On each scroll event:
clock.set_position(current_scroll_offset);
let dt = clock.delta();
tween.update(dt);

let value = tween.value(); // moves as user scrolls
```

### Key Methods

| Method | Description |
|--------|-------------|
| `ScrollClock::new(start, end)` | Create a clock mapping position range `[start, end]` |
| `.set_position(pos)` | Update the current scroll position |
| `.delta()` | Returns accumulated progress change since last call, resets accumulator |
| `.position()` | Current scroll position |
| `.progress()` | Current progress `0.0..=1.0` (clamped) |
| `.set_range(start, end)` | Update the scroll range at runtime |

### How Delta Works

When you call `set_position()`, the clock computes the *change in normalised progress*:

```
delta = (new_position - old_position) / (end - start)
```

Multiple `set_position()` calls between `delta()` calls accumulate. Scrolling backward produces negative delta.

---

## ScrollDriver

`ScrollDriver` combines a `ScrollClock` with an animation collection. You add animations, call `set_position()` on each scroll event, and completed animations are auto-removed — just like `AnimationDriver`.

```rust
use spanda::scroll::ScrollDriver;
use spanda::tween::Tween;
use spanda::easing::Easing;

let mut driver = ScrollDriver::new(0.0, 1000.0);

// Animations should use duration 1.0 — the driver normalises scroll to [0, 1]
driver.add(
    Tween::new(0.0_f32, 1.0)
        .duration(1.0)
        .easing(Easing::EaseOutCubic)
        .build()
);

// In your scroll handler:
driver.set_position(scroll_offset);
```

### Key Methods

| Method | Description |
|--------|-------------|
| `ScrollDriver::new(start, end)` | Create a driver for the given scroll range |
| `.add(animation)` | Add an animation, returns `AnimationId` |
| `.set_position(pos)` | Set scroll position and tick all animations |
| `.cancel(id)` | Cancel a specific animation |
| `.cancel_all()` | Cancel all animations |
| `.active_count()` | Number of active animations |
| `.progress()` | Current scroll progress `0.0..=1.0` |
| `.position()` | Current scroll position |
| `.clock()` / `.clock_mut()` | Access the underlying `ScrollClock` |

---

## Scroll Range Design

The scroll range (`start` to `end`) maps to animation progress 0.0 to 1.0:

```
Scroll position:  0 -------- 500 -------- 1000
Animation progress: 0.0       0.5         1.0
```

- Scrolling from `start` to `end` plays the animation forward
- Scrolling backward reverses the animation (negative delta)
- Positions outside the range produce progress outside 0..1 (the animation still receives the delta)

### Variable Scroll Ranges

You can change the range at runtime via `clock_mut().set_range()`:

```rust
// Recalculate after layout changes
driver.clock_mut().set_range(new_start, new_end);
```

---

## Integration Patterns

### With Leptos

```rust,ignore
use leptos::*;
use spanda::scroll::ScrollDriver;
use spanda::tween::Tween;
use spanda::easing::Easing;

#[component]
fn ScrollAnimated() -> impl IntoView {
    let (opacity, set_opacity) = create_signal(0.0_f32);

    let mut driver = ScrollDriver::new(0.0, 500.0);
    let mut tween = Tween::new(0.0_f32, 1.0)
        .duration(1.0)
        .easing(Easing::EaseOutCubic)
        .build();
    tween.on_update(move |val| set_opacity.set(val));
    driver.add(tween);

    // On scroll event:
    // driver.set_position(window.scroll_y());

    view! {
        <div style:opacity=move || opacity.get().to_string()>
            "Fades in as you scroll"
        </div>
    }
}
```

### With a Web Framework (WASM)

```rust,ignore
use spanda::scroll::ScrollDriver;

// In your JS interop:
#[wasm_bindgen]
pub fn on_scroll(scroll_y: f32) {
    // driver is stored globally or in component state
    driver.set_position(scroll_y);
}
```

---

## ScrollClock vs. ScrollDriver

| | ScrollClock | ScrollDriver |
|---|-------------|-------------|
| **What it is** | A `Clock` — produces `dt` | A full driver — manages animations |
| **Use when** | You have your own driver or need per-animation control | You want a self-contained scroll animation manager |
| **Manages animations** | No | Yes (add, cancel, auto-remove) |
| **Implements** | `Clock` trait | — |

---

## Edge Cases

| Scenario | Behaviour |
|----------|-----------|
| Zero-width range (`start == end`) | `set_position()` is a no-op, `delta()` returns 0.0 |
| Scroll position outside range | Delta is computed normally (may exceed 0..1 progress) |
| Multiple `set_position()` calls before `delta()` | Accumulated into one delta |
| No position change | `set_position()` with the same value produces zero delta |
