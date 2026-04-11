//! WASM tween example — animate a DOM element using spanda + RafDriver.
//!
//! Build with: `trunk serve --open` from this directory.
//!
//! For Lenis-style **window** smooth scrolling, enable crate features `wasm,wasm-dom` and use
//! [`spanda::integrations::smooth_scroll::SmoothScroll`](https://docs.rs/spanda/latest/spanda/integrations/smooth_scroll/struct.SmoothScroll.html)
//! alongside your rAF loop (`SmoothScroll::tick`).

use spanda::easing::Easing;
use spanda::integrations::wasm::RafDriver;
use spanda::tween::Tween;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

// ── App state in thread-local ───────────────────────────────────────────────

thread_local! {
    static DRIVER: RefCell<Option<RafDriver>> = RefCell::new(None);
}

fn new_driver() -> RafDriver {
    let mut d = RafDriver::new();

    let ball: web_sys::HtmlElement = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("ball")
        .unwrap()
        .dyn_into()
        .unwrap();
    let ball2 = ball.clone();

    // Animate ball position 0→500px over 2s with bounce easing
    let mut pos_tween = Tween::new(0.0_f32, 500.0)
        .duration(2.0)
        .easing(Easing::EaseOutBounce)
        .build();
    pos_tween.on_update(move |val: f32| {
        let _ = ball.style().set_property("left", &format!("{val}px"));
    });
    d.add(pos_tween);

    // Fade opacity 0→1 over 1s
    let mut fade_tween = Tween::new(0.0_f32, 1.0)
        .duration(1.0)
        .easing(Easing::EaseOutCubic)
        .build();
    fade_tween.on_update(move |val: f32| {
        let _ = ball2.style().set_property("opacity", &format!("{val}"));
    });
    d.add(fade_tween);

    d
}

// ── Entry point — trunk calls this after WASM init ──────────────────────────

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    DRIVER.with(|d| *d.borrow_mut() = Some(new_driver()));

    register_globals()?;
    setup_visibility()?;
    start_animation_loop()?;
    update_status();

    Ok(())
}

// ── Register global JS functions for the HTML buttons ───────────────────────

fn register_globals() -> Result<(), JsValue> {
    // window.restart()
    let restart = Closure::wrap(Box::new(|| {
        DRIVER.with(|d| *d.borrow_mut() = Some(new_driver()));
    }) as Box<dyn FnMut()>);
    js_sys::Reflect::set(
        &js_sys::global(),
        &"restart".into(),
        restart.as_ref().unchecked_ref(),
    )?;
    restart.forget();

    // window.togglePause()
    let toggle = Closure::wrap(Box::new(|| {
        DRIVER.with(|d| {
            if let Some(drv) = d.borrow_mut().as_mut() {
                if drv.is_paused() {
                    drv.resume();
                } else {
                    drv.pause();
                }
            }
        });
    }) as Box<dyn FnMut()>);
    js_sys::Reflect::set(
        &js_sys::global(),
        &"togglePause".into(),
        toggle.as_ref().unchecked_ref(),
    )?;
    toggle.forget();

    Ok(())
}

// ── Visibility change handler ───────────────────────────────────────────────

fn setup_visibility() -> Result<(), JsValue> {
    let doc = web_sys::window()
        .and_then(|w| w.document())
        .ok_or("no document")?;

    let cb = Closure::wrap(Box::new(|| {
        let hidden = web_sys::window()
            .and_then(|w| w.document())
            .map_or(false, |d| d.hidden());
        DRIVER.with(|d| {
            if let Some(drv) = d.borrow_mut().as_mut() {
                drv.on_visibility_change(hidden);
            }
        });
    }) as Box<dyn FnMut()>);

    doc.add_event_listener_with_callback("visibilitychange", cb.as_ref().unchecked_ref())?;
    cb.forget();

    Ok(())
}

// ── rAF animation loop ─────────────────────────────────────────────────────

fn start_animation_loop() -> Result<(), JsValue> {
    let f: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move |timestamp: f64| {
        // Tick the driver
        DRIVER.with(|d| {
            if let Some(drv) = d.borrow_mut().as_mut() {
                drv.tick(timestamp);
            }
        });

        update_status();

        // Schedule next frame
        if let Some(ref cb) = *f.borrow() {
            let _ = raf_request(cb);
        }
    }) as Box<dyn FnMut(f64)>));

    // Kick off the first frame
    {
        let guard = g.borrow();
        if let Some(ref cb) = *guard {
            raf_request(cb)?;
        }
    }

    Ok(())
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn update_status() {
    let active = DRIVER.with(|d| d.borrow().as_ref().map_or(0, |drv| drv.active_count()));
    let paused = DRIVER.with(|d| d.borrow().as_ref().map_or(false, |drv| drv.is_paused()));

    let text = if paused {
        format!("Paused | Active: {active}")
    } else {
        format!("Active: {active} | Press Pause/Resume or Restart")
    };

    if let Some(el) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("status"))
    {
        el.set_text_content(Some(&text));
    }
}

fn raf_request(cb: &Closure<dyn FnMut(f64)>) -> Result<i32, JsValue> {
    js_sys::Reflect::get(&js_sys::global(), &"requestAnimationFrame".into())
        .and_then(|raf| {
            let raf = js_sys::Function::from(raf);
            raf.call1(&JsValue::NULL, cb.as_ref().unchecked_ref())
        })
        .map(|v| v.as_f64().unwrap_or(0.0) as i32)
}
