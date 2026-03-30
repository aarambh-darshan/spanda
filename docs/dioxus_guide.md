# Dioxus Integration Guide

Integrate spanda animations into [Dioxus](https://dioxuslabs.com) components using coroutines and reactive hooks.

---

## Setup

```toml
[dependencies]
spanda = { version = "0.9", features = ["wasm"] }
dioxus = "0.5"
```

---

## Basic: Animated Opacity

Use `use_signal` and a coroutine to drive the animation loop:

```rust
use dioxus::prelude::*;
use spanda::tween::Tween;
use spanda::easing::Easing;
use spanda::traits::Update;

#[component]
fn FadeIn() -> Element {
    let mut opacity = use_signal(|| 0.0_f32);

    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        let mut tween = Tween::new(0.0_f32, 1.0)
            .duration(1.0)
            .easing(Easing::EaseOutCubic)
            .build();

        loop {
            if !tween.update(1.0 / 60.0) { break; }
            opacity.set(tween.value());
            // Yield to the runtime (~16ms between frames)
            gloo_timers::future::TimeoutFuture::new(16).await;
        }
        opacity.set(1.0); // ensure final value
    });

    rsx! {
        div {
            style: "opacity: {opacity};",
            "Fading in..."
        }
    }
}
```

---

## Staggered Animations

Use `Timeline` with stagger to animate multiple elements with offsets:

```rust
use dioxus::prelude::*;
use spanda::tween::Tween;
use spanda::easing::Easing;
use spanda::timeline::stagger;
use spanda::traits::Update;

#[component]
fn StaggeredCards(items: Vec<String>) -> Element {
    let mut opacities: Vec<Signal<f32>> = (0..items.len())
        .map(|_| use_signal(|| 0.0_f32))
        .collect();

    let item_count = items.len();
    let opacity_clones: Vec<_> = opacities.clone();

    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        let tweens: Vec<_> = (0..item_count).map(|i| {
            let mut sig = opacity_clones[i];
            let mut tween = Tween::new(0.0_f32, 1.0)
                .duration(0.3)
                .easing(Easing::EaseOutCubic)
                .build();
            tween.on_update(move |val| sig.set(val));
            (tween, 0.3)
        }).collect();

        let mut timeline = stagger(tweens, 0.08);
        timeline.play();

        while timeline.update(1.0 / 60.0) {
            gloo_timers::future::TimeoutFuture::new(16).await;
        }
    });

    rsx! {
        div {
            for (i, item) in items.iter().enumerate() {
                div {
                    style: "opacity: {opacities[i]};
                            transform: translateY({(1.0 - *opacities[i].read()) * 20.0}px);
                            transition: none;",
                    "{item}"
                }
            }
        }
    }
}
```

---

## Spring-Driven Motion

Springs work well for interactive elements like tooltips and drag targets:

```rust
use dioxus::prelude::*;
use spanda::spring::{Spring, SpringConfig};
use spanda::traits::Update;

#[component]
fn SpringFollower() -> Element {
    let mut x = use_signal(|| 0.0_f32);
    let mut y = use_signal(|| 0.0_f32);
    let mut target_x = use_signal(|| 0.0_f32);
    let mut target_y = use_signal(|| 0.0_f32);

    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        let mut sx = Spring::new(SpringConfig::wobbly());
        let mut sy = Spring::new(SpringConfig::wobbly());

        loop {
            sx.set_target(*target_x.read());
            sy.set_target(*target_y.read());

            sx.update(1.0 / 60.0);
            sy.update(1.0 / 60.0);

            x.set(sx.position());
            y.set(sy.position());

            gloo_timers::future::TimeoutFuture::new(16).await;
        }
    });

    rsx! {
        div {
            style: "width:100%;height:100vh;position:relative;",
            onmousemove: move |ev: MouseEvent| {
                let coords = ev.page_coordinates();
                target_x.set(coords.x as f32);
                target_y.set(coords.y as f32);
            },
            div {
                style: "width:40px;height:40px;background:#00d4ff;border-radius:50%;
                        position:absolute;left:{x}px;top:{y}px;pointer-events:none;",
            }
        }
    }
}
```

---

## Using RafDriver

For complex multi-animation scenarios:

```rust
use dioxus::prelude::*;
use spanda::integrations::wasm::RafDriver;
use spanda::tween::Tween;
use spanda::easing::Easing;
use std::rc::Rc;
use std::cell::RefCell;

#[component]
fn AnimationManager() -> Element {
    let driver = use_signal(|| Rc::new(RefCell::new(RafDriver::new())));

    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        let d = driver.read().clone();
        {
            let mut d = d.borrow_mut();
            d.add(Tween::new(0.0_f32, 100.0).duration(2.0).easing(Easing::EaseOutBounce).build());
        }

        loop {
            let timestamp = js_sys::Date::now();
            d.borrow_mut().tick(timestamp);

            if d.borrow().active_count() == 0 { break; }
            gloo_timers::future::TimeoutFuture::new(16).await;
        }
    });

    rsx! { p { "Animations running..." } }
}
```

---

## Tips

- Use `use_coroutine` for animation loops in Dioxus
- `gloo_timers::future::TimeoutFuture::new(16).await` approximates 60fps frame timing
- For WASM targets, use `spanda::integrations::wasm::RafDriver` for precise timing
- Springs are ideal for cursor-following elements — `set_target()` from mouse events
- Use `use_signal` for each animated property so components re-render on value changes
