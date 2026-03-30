# Leptos Integration Guide

Integrate spanda animations directly into [Leptos](https://leptos.dev) reactive components. The `on_update` callback bridges animation values into Leptos signals with zero boilerplate.

---

## Setup

```toml
[dependencies]
spanda = { version = "0.9", features = ["wasm"] }
leptos = "0.6"
```

---

## Basic: Animated Opacity

Use `on_update` to pipe interpolated values into a signal:

```rust
use leptos::*;
use spanda::tween::Tween;
use spanda::easing::Easing;
use spanda::traits::Update;

#[component]
fn FadeIn() -> impl IntoView {
    let (opacity, set_opacity) = create_signal(0.0_f32);

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

---

## Staggered List Animation

Animate multiple elements with offset starts using `stagger`:

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

    view! {
        <ul>
            {items.iter().enumerate().map(|(i, item)| {
                let (opacity, _) = signals[i];
                view! {
                    <li style:opacity=move || opacity.get().to_string()
                        style:transform=move || {
                            let o = opacity.get();
                            format!("translateY({}px)", (1.0 - o) * 20.0)
                        }>
                        {item.clone()}
                    </li>
                }
            }).collect_view()}
        </ul>
    }
}
```

---

## Spring-Driven Drag

Springs are ideal for interactive UI — retarget mid-flight with velocity preservation:

```rust
use leptos::*;
use spanda::spring::{Spring, SpringConfig};
use spanda::traits::Update;

#[component]
fn DraggableBox() -> impl IntoView {
    let (x, set_x) = create_signal(0.0_f32);
    let (y, set_y) = create_signal(0.0_f32);

    let spring_x = store_value(Spring::new(SpringConfig::wobbly()));
    let spring_y = store_value(Spring::new(SpringConfig::wobbly()));

    // Tick springs every frame
    set_interval(
        move || {
            spring_x.update_value(|s| {
                s.update(1.0 / 60.0);
                set_x.set(s.position());
            });
            spring_y.update_value(|s| {
                s.update(1.0 / 60.0);
                set_y.set(s.position());
            });
        },
        std::time::Duration::from_millis(16),
    );

    let on_mouse_move = move |ev: web_sys::MouseEvent| {
        spring_x.update_value(|s| s.set_target(ev.client_x() as f32));
        spring_y.update_value(|s| s.set_target(ev.client_y() as f32));
    };

    view! {
        <div on:mousemove=on_mouse_move style="width:100%;height:100vh;position:relative;">
            <div style:left=move || format!("{}px", x.get())
                 style:top=move || format!("{}px", y.get())
                 style="width:40px;height:40px;background:#00d4ff;border-radius:50%;position:absolute;">
            </div>
        </div>
    }
}
```

---

## RafDriver for Complex Animations

For managing many animations, use `RafDriver` with Leptos:

```rust
use leptos::*;
use spanda::integrations::wasm::RafDriver;
use spanda::tween::Tween;
use spanda::easing::Easing;
use std::rc::Rc;
use std::cell::RefCell;

#[component]
fn MultiAnimation() -> impl IntoView {
    let driver = Rc::new(RefCell::new(RafDriver::new()));

    // Add multiple animations
    {
        let mut d = driver.borrow_mut();
        d.add(Tween::new(0.0_f32, 100.0).duration(1.0).easing(Easing::EaseOutBounce).build());
        d.add(Tween::new(0.0_f32, 1.0).duration(0.5).easing(Easing::EaseOutCubic).build());
    }

    // Use rAF for smooth animation
    let d = driver.clone();
    // In a real app, use start_raf_loop or set_interval
    set_interval(
        move || {
            let timestamp = js_sys::Date::now();
            d.borrow_mut().tick(timestamp);
        },
        std::time::Duration::from_millis(16),
    );

    view! { <p>"Animating with RafDriver..."</p> }
}
```

---

## Tips

- Use `on_update(move |val| set_signal.set(val))` to bridge tweens to signals
- Use `store_value()` for mutable animation state in Leptos closures
- For multiple synchronized animations, use `Timeline` or `stagger`
- Springs are ideal for drag interactions — call `set_target()` from mouse events
- Use `RafDriver` when managing 5+ concurrent animations
