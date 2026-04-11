# Integrations

`spanda` is designed as a **pure data-transformer**. It knows nothing about the screen, pixels, or your windowing library. You call `update(dt)`, it returns values — you decide how to render them.

This architecture makes integrating spanda trivial across any Rust target.

---

## AnimationDriver & Clock

Before diving into specific integrations, it helps to understand the two infrastructure pieces that tie everything together.

### AnimationDriver

The `AnimationDriver` manages a collection of active animations. You add animations, tick the driver each frame, and completed animations are auto-removed:

```rust
use spanda::driver::AnimationDriver;
use spanda::tween::Tween;

let mut driver = AnimationDriver::new();

// Add multiple animations
let id = driver.add(Tween::new(0.0_f32, 100.0).duration(1.0).build());

// Tick all animations each frame
driver.tick(dt);

// Check active count, cancel by id, or cancel all
driver.cancel(id);
driver.cancel_all();
println!("Active: {}", driver.active_count());
```

### AnimationDriverArc (Thread-Safe)

For multi-threaded scenarios (audio thread + render thread), use `AnimationDriverArc` — a `Clone`-able wrapper backed by `Arc<Mutex<AnimationDriver>>`:

```rust
use spanda::driver::AnimationDriverArc;

let driver = AnimationDriverArc::new();

// Clone and send to another thread
let driver_clone = driver.clone();
std::thread::spawn(move || {
    driver_clone.tick(0.016);
});
```

> **Note**: `AnimationDriverArc` requires `feature = "std"`.

### Clock Trait

The `Clock` trait abstracts time sourcing. Spanda ships three implementations:

| Clock | Description | Use Case |
|-------|-------------|----------|
| `WallClock` | Real wall time via `std::time::Instant` | Production apps (`std` only) |
| `ManualClock` | Caller provides `dt` via `.advance()` | Game engines with their own time step |
| `MockClock` | Fixed `dt` on every call | Deterministic unit tests |

```rust
use spanda::clock::{Clock, WallClock, ManualClock, MockClock};

// Real time
let mut clock = WallClock::new();
let dt = clock.delta(); // seconds since last call

// Manual (game engine)
let mut clock = ManualClock::new();
clock.advance(0.016); // you tell it how much time passed
let dt = clock.delta(); // returns 0.016, resets accumulator

// Mock (testing)
let mut clock = MockClock::new(1.0 / 60.0);
let dt = clock.delta(); // always returns 1/60
```

---

## TUI / CLI

In a terminal UI (like `ratatui` or `crossterm`), you run a standard frame loop. Pair `WallClock` with your render:

```rust
use spanda::clock::{Clock, WallClock};
use spanda::tween::Tween;
use spanda::easing::Easing;
use spanda::traits::Update;

let mut clock = WallClock::new();
let mut tween = Tween::new(0.0_f32, 100.0)
    .duration(2.0)
    .easing(Easing::EaseOutCubic)
    .build();

loop {
    let dt = clock.delta();
    let running = tween.update(dt);
    
    let progress = tween.value();
    // draw_progress_bar(progress);
    
    if !running { break; }
    std::thread::sleep(std::time::Duration::from_millis(16));
}
```

For managing multiple animations in a TUI, use the `AnimationDriver`:

```rust
use spanda::driver::AnimationDriver;

let mut driver = AnimationDriver::new();
driver.add(progress_tween);
driver.add(spinner_opacity);
driver.add(spring_element);

loop {
    let dt = clock.delta();
    driver.tick(dt);
    
    if driver.active_count() == 0 { break; }
    // render...
}
```

---

## Bevy Plugin

If you use [Bevy](https://bevyengine.org), activate the `bevy` feature:

```toml
[dependencies]
spanda = { version = "0.9.3", features = ["bevy"] }
```

This adds `SpandaPlugin`, which automatically:
- Registers `Tween<f32>`, `Tween<[f32;2]>`, `Tween<[f32;3]>`, `Tween<[f32;4]>` as ECS **Components**
- Registers `Spring` as an ECS **Component**
- Ticks them in the `Update` schedule using Bevy's `Time` resource
- Fires `TweenCompleted` events when tweens finish
- Fires `SpringSettled` events when springs reach their target

```rust
use bevy::prelude::*;
use spanda::integrations::bevy::{SpandaPlugin, TweenCompleted};
use spanda::{Tween, Easing};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SpandaPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, listen)
        .run();
}

fn setup(mut commands: Commands) {
    // The plugin ticks this component automatically every frame
    commands.spawn((
        // Transform, SpriteBundle, etc...
        Tween::new(0.0_f32, 100.0)
            .duration(1.0)
            .easing(Easing::EaseOutCubic)
            .build(),
    ));
}

fn listen(mut events: MessageReader<TweenCompleted>) {
    for event in events.read() {
        println!("Entity {:?} finished its tween!", event.entity);
    }
}
```

### Springs in Bevy

`Spring` is also a Bevy `Component` — it's ticked automatically:

```rust
commands.spawn((
    SpriteBundle { /* ... */ },
    Spring::new(SpringConfig::wobbly()),
));
```

---

## WASM / Web

For WebAssembly apps (Leptos, Dioxus, Yew), standard `std::time` doesn't work for smooth visuals. You need the browser's `requestAnimationFrame`.

Activate the `wasm` feature:

```toml
[dependencies]
spanda = { version = "0.9.3", features = ["wasm"] }
```

Use `RafDriver` — pass it the high-resolution timestamp from JavaScript.  New in 0.6: pause/resume, time scale, visibility change handling, and `start_raf_loop`:

```rust
use spanda::integrations::wasm::RafDriver;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct App {
    driver: RafDriver,
}

#[wasm_bindgen]
impl App {
    pub fn new() -> Self {
        let mut driver = RafDriver::new();
        // driver.add(Tween::new(...).build());
        Self { driver }
    }

    // Called from JS: requestAnimationFrame(timestamp => app.tick(timestamp))
    pub fn tick(&mut self, timestamp_ms: f64) {
        self.driver.tick(timestamp_ms);
    }

    pub fn pause(&mut self) { self.driver.pause(); }
    pub fn resume(&mut self) { self.driver.resume(); }

    // Handle tab visibility changes
    pub fn on_visibility_change(&mut self, hidden: bool) {
        self.driver.on_visibility_change(hidden);
    }
}
```

#### Automatic rAF Loop

Use `start_raf_loop` to avoid manual `requestAnimationFrame` scheduling:

```rust
use spanda::integrations::wasm::{RafDriver, start_raf_loop};
use std::rc::Rc;
use std::cell::RefCell;

let driver = Rc::new(RefCell::new(RafDriver::new()));
let d = driver.clone();

start_raf_loop(move |timestamp_ms| {
    d.borrow_mut().tick(timestamp_ms);
});
```

### Leptos Integration Pattern

In Leptos, spanda's `on_update` callback bridges animation values directly into signals — no manual polling needed.

See the full [Leptos Integration Guide](leptos_guide.md) for complete examples including staggered lists and spring-driven drag.

```rust
use leptos::*;
use spanda::tween::Tween;
use spanda::easing::Easing;
use spanda::traits::Update;

#[component]
fn AnimatedBox() -> impl IntoView {
    let (opacity, set_opacity) = create_signal(0.0_f32);

    // Build the tween
    let mut tween = Tween::new(0.0_f32, 1.0)
        .duration(1.0)
        .easing(Easing::EaseOutCubic)
        .build();

    // Bridge to signal — on_update receives the interpolated value directly
    tween.on_update(move |val: f32| set_opacity.set(val));
    tween.on_complete(move || log::info!("Fade complete"));

    let tween = store_value(tween);

    // Drive with set_interval
    set_interval(
        move || {
            tween.update_value(|t| { t.update(1.0 / 60.0); });
        },
        std::time::Duration::from_millis(16),
    );

    view! {
        <div style:opacity=move || opacity.get().to_string()>
            "Fading in..."
        </div>
    }
}
```

#### Staggering in Leptos

Use `spanda::timeline::stagger` to animate multiple elements with offset starts:

```rust
use leptos::*;
use spanda::tween::Tween;
use spanda::easing::Easing;
use spanda::timeline::stagger;
use spanda::traits::Update;

#[component]
fn StaggeredList(items: Vec<String>) -> impl IntoView {
    let signals: Vec<_> = items.iter()
        .map(|_| create_signal(0.0_f32))
        .collect();

    let tweens: Vec<_> = signals.iter().map(|(_, set_sig)| {
        let set_sig = *set_sig;
        let mut tween = Tween::new(0.0_f32, 1.0)
            .duration(0.3)
            .easing(Easing::EaseOutCubic)
            .build();
        tween.on_update(move |val| set_sig.set(val));
        (tween, 0.3)
    }).collect();

    let mut timeline = stagger(tweens, 0.08);
    timeline.play();
    let timeline = store_value(timeline);

    set_interval(
        move || { timeline.update_value(|tl| { tl.update(1.0 / 60.0); }); },
        std::time::Duration::from_millis(16),
    );

    // Render items with animated opacity from signals...
}
```

### Dioxus Integration Pattern

In Dioxus, use a coroutine or `use_future` for animation loops.

See the full [Dioxus Integration Guide](dioxus_guide.md) for complete examples including springs, staggered animations, and RafDriver.

```rust
use dioxus::prelude::*;
use spanda::tween::Tween;
use spanda::easing::Easing;
use spanda::traits::Update;

#[component]
fn AnimatedBox() -> Element {
    let mut opacity = use_signal(|| 0.0_f32);

    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        let mut tween = Tween::new(0.0_f32, 1.0)
            .duration(1.0)
            .easing(Easing::EaseOutCubic)
            .build();

        loop {
            if !tween.update(1.0 / 60.0) { break; }
            opacity.set(tween.value());
            gloo_timers::future::TimeoutFuture::new(16).await;
        }
    });

    rsx! {
        div { style: "opacity: {opacity};", "Fading in..." }
    }
}
```

---

## WASM-DOM Plugins

For web apps that need direct DOM interaction, the `wasm-dom` feature adds five high-level plugins built on spanda's pure-math primitives.

```toml
[dependencies]
spanda = { version = "0.9.3", features = ["wasm-dom"] }
```

> `wasm-dom` implies `wasm`, so you don't need to specify both.

### Observer

Unified pointer/touch/mouse event normaliser. Binds to any DOM element and delivers all input as `PointerData`:

```rust
use spanda::integrations::observer::{Observer, ObserverCallbacks};
use spanda::drag::PointerData;

let callbacks = ObserverCallbacks {
    on_press: Some(Box::new(|data: PointerData| {
        log::info!("Press at ({}, {})", data.x, data.y);
    })),
    on_move: Some(Box::new(|data: PointerData| {
        // track position
    })),
    on_release: Some(Box::new(|data: PointerData| {
        // handle release
    })),
    on_wheel: Some(Box::new(|delta_y: f64| {
        // handle scroll wheel
    })),
};

let observer = Observer::bind(&element, callbacks);
// observer.unbind() to remove all listeners
```

### FLIP Animations

The [FLIP technique](https://aerotwist.com/blog/flip-your-animations/) (First, Last, Invert, Play) for layout-driven animations:

```rust
use spanda::integrations::flip::{FlipState, FlipAnimation};
use spanda::traits::Update;

// 1. Capture the FIRST state
let first = FlipState::capture(&element);

// 2. Make your layout change (add class, move element, etc.)
element.class_list().add_1("expanded").unwrap();

// 3. Capture the LAST state
let last = FlipState::capture(&element);

// 4. Create and play the animation
let mut anim: FlipAnimation = FlipState::diff(&first, &last)
    .duration(0.3)
    .build();

// 5. Each frame, update and apply
anim.update(dt);
let css = anim.css_transform(); // "translate(Xpx, Ypx) scale(Sx, Sy)"
```

For pure-math use (no DOM), use `FlipState::from_rect(x, y, w, h)` to create states manually.

### SplitText

Split text into individual characters or words for staggered animation. The core splitting is always available (no feature gate needed); DOM injection requires `wasm-dom`:

```rust
use spanda::integrations::split_text::SplitText;
use spanda::easing::Easing;

// Pure string splitting — works everywhere
let split = SplitText::from_str("Hello World");
assert_eq!(split.char_count(), 11);
assert_eq!(split.word_count(), 2);

// Create a staggered fade-in timeline for each character
let timeline = split.stagger_chars(
    0.0_f32,             // from value
    1.0_f32,             // to value
    0.3,                 // per-character duration
    0.05,                // stagger delay between characters
    Easing::EaseOutCubic,
);

// DOM injection (wasm-dom only):
// split.inject_chars(&parent_element);  // wraps each char in <span>
// split.inject_words(&parent_element);  // wraps each word in <span>
```

### ScrollSmoother

Spring-driven smooth scroll overlay. Intercepts native scrolling and smooths it through a `Spring`:

```rust
use spanda::integrations::scroll_smoother::ScrollSmoother;
use spanda::spring::SpringConfig;

let mut smoother = ScrollSmoother::new(
    content_element,           // the scrollable content HtmlElement
    SpringConfig::gentle(),    // spring config for smoothing
);

smoother.attach(); // sets overflow: hidden, listens to scroll events

// Each frame:
smoother.tick(dt);
let smooth_pos = smoother.position(); // spring-smoothed scroll position
// smoother.detach() to restore native scrolling
```

### SmoothScroll

Lenis-style **window** smooth scrolling: virtual target + exponential decay toward it, applied only with `Window::scroll_to_with_x_and_y`. Handles wheel (non-passive), keyboard (`Space`, `PageDown`/`PageUp`, arrows, `Home`/`End`), touch drag + [`InertiaN`](https://docs.rs/spanda/latest/spanda/inertia/struct.InertiaN.html) momentum on release, `resize`, `hashchange` and same-document anchor clicks (capture-phase `click` + `history.pushState`), and `prefers-reduced-motion`. Sets `touch-action: none` and `overscroll-behavior: none` on the document element while attached.

For scroll-linked tweens, pass [`SmoothScroll::current_scroll`](https://docs.rs/spanda/latest/spanda/integrations/smooth_scroll/struct.SmoothScroll.html#method.current_scroll) into [`ScrollDriver::set_position`](https://docs.rs/spanda/latest/spanda/scroll/struct.ScrollDriver.html#method.set_position) so animation progress matches the smoothed viewport.

[`ScrollSmoother`](https://docs.rs/spanda/latest/spanda/integrations/scroll_smoother/struct.ScrollSmoother.html) remains the spring + `transform` lag variant; choose one or the other.

```rust,ignore
use spanda::integrations::smooth_scroll::{SmoothScroll, SmoothScrollOptions};

let mut smooth = SmoothScroll::new(SmoothScrollOptions::default());
smooth.attach().expect("attach");

// In your requestAnimationFrame callback, dt in seconds:
smooth.tick(dt);

// Optional: programmatic scroll (instant if smooth == false or reduced motion)
smooth.scroll_to(400.0, true);
```

### Draggable

DOM-bound pointer event wrapper over `DragState`. Handles all the event binding so you just read position:

```rust
use spanda::integrations::draggable::Draggable;
use spanda::drag::DragConstraints;

// Simple — drag anywhere
let draggable = Draggable::bind(&element);

// With constraints
let draggable = Draggable::bind_with_constraints(&element, DragConstraints {
    bounds: Some([0.0, 0.0, 500.0, 500.0]),
    snap_to_grid: Some([20.0, 20.0]),
    ..Default::default()
});

// Read state
let pos = draggable.position();     // [f32; 2]
let active = draggable.is_dragging();

// Cleanup
draggable.unbind();
```

---

## Scroll-Linked Animations

Use `ScrollDriver` / `ScrollClock` to drive animations from scroll position instead of wall time:

```rust
use spanda::scroll::{ScrollDriver, ScrollClock};
use spanda::tween::Tween;
use spanda::easing::Easing;

// Map scroll range 0..1000 pixels to animation progress
let mut driver = ScrollDriver::new(0.0, 1000.0);

// Animations should use duration 1.0 — the driver normalises scroll to [0, 1]
driver.add(
    Tween::new(0.0_f32, 1.0)
        .duration(1.0)
        .easing(Easing::EaseOutCubic)
        .build()
);

// In your scroll handler, use the smoothed offset when using `SmoothScroll`:
// driver.set_position(smooth.current_scroll());
driver.set_position(scroll_offset);
```

### ScrollClock for Manual Use

If you already have a driver or want per-animation control, use `ScrollClock` directly:

```rust
use spanda::scroll::ScrollClock;
use spanda::clock::Clock;
use spanda::tween::Tween;
use spanda::traits::Update;

let mut clock = ScrollClock::new(0.0, 1000.0);
let mut tween = Tween::new(0.0_f32, 100.0).duration(1.0).build();

// On each scroll event:
clock.set_position(current_scroll);
let dt = clock.delta();
tween.update(dt);
```

## Motion Paths

Animate values along Bezier curves instead of straight lines:

```rust
use spanda::path::{BezierPath, MotionPath, MotionPathTween, PathEvaluate};
use spanda::easing::Easing;
use spanda::traits::Update;

// Single cubic Bezier
let curve = BezierPath::cubic(
    [0.0_f32, 0.0],
    [0.0, 100.0],
    [100.0, 100.0],
    [100.0, 0.0],
);
let point = curve.evaluate(0.5); // [50, 75] approximately

// Multi-segment motion path
let path = MotionPath::new()
    .cubic([0.0, 0.0], [50.0, 100.0], [100.0, 100.0], [150.0, 0.0])
    .line([150.0, 0.0], [200.0, 0.0]);

// Animate along the path
let mut tween = MotionPathTween::new(path)
    .duration(2.0)
    .easing(Easing::EaseInOutCubic);

tween.update(1.0); // 50% through
let pos = tween.value(); // position on the path
```

---

## Embedded / `no_std`

Spanda works in `no_std` environments — disable the default `std` feature:

```toml
[dependencies]
spanda = { version = "0.9.3", default-features = false }
```

In `no_std` mode:
- **Available**: `Tween`, `Easing`, `KeyframeTrack`, `Spring`, all math
- **Available**: `ManualClock`, `MockClock`
- **Unavailable**: `WallClock` (requires `std::time`), `AnimationDriverArc` (requires `Arc<Mutex<>>`)
- **Unavailable**: Timeline callbacks (requires `Box<dyn FnMut()>`)

> **Note**: `Timeline`, `Sequence`, `AnimationDriver`, and `KeyframeTrack` use `alloc` (Vec, Box, String). In `no_std` environments, you need `extern crate alloc`.

### Embedded Example

```rust
#![no_std]
extern crate alloc;

use spanda::tween::Tween;
use spanda::easing::Easing;
use spanda::traits::Update;

fn animate_led_brightness() {
    let mut tween = Tween::new(0.0_f32, 255.0)
        .duration(2.0)
        .easing(Easing::EaseInOutSine)
        .build();
    
    // In your embedded loop:
    let dt = 0.01; // 100 Hz timer
    tween.update(dt);
    let brightness = tween.value() as u8;
    // set_led_pwm(brightness);
}
```

---

## Feature Flag Decision Guide

| You are building... | Recommended features | Why |
|---------------------|----------------------|-----|
| A TUI app | `default` (just `std`) | `WallClock` for real-time frame loop |
| A Bevy game | `bevy` | Auto-ticks components via `SpandaPlugin` |
| A Leptos/Yew web app | `wasm` | `RafDriver` for `requestAnimationFrame` |
| A CLI tool | `default` | Standard `WallClock` + `AnimationDriver` |
| Embedded / `no_std` | `default-features = false` | Pure math, zero OS dependencies |
| State persistence | `serde` | Serialize/deserialize all animation types |
| Colour animations | `palette` | `Interpolate` impl for `palette` colour types |
| Async workflows | `tokio` | `.await` on timeline completion |
| Full everything | `std,serde,bevy,wasm,wasm-dom,palette,tokio` | All features enabled |
