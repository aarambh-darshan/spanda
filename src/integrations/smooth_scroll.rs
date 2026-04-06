//! Lenis-style **window** smooth scrolling (`wasm-dom`).
//!
//! [`SmoothScroll`] drives [`SmoothScroll1D`](crate::scroll_smooth::SmoothScroll1D) with wheel,
//! keyboard, touch + [`InertiaN`](crate::inertia::InertiaN) (1D) momentum, resize, hash / anchor
//! navigation, and `prefers-reduced-motion`. Scroll position is applied only via
//! [`Window::scroll_to_with_x_and_y`](web_sys::Window::scroll_to_with_x_and_y).
//!
//! [`ScrollSmoother`](super::scroll_smoother::ScrollSmoother) (spring + transform) is unchanged;
//! use this type for virtual smooth scroll on the window.

#![allow(clippy::type_complexity)]

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{
    AddEventListenerOptions, Event, HtmlElement, KeyboardEvent, MediaQueryList, TouchEvent,
    WheelEvent,
};

use crate::inertia::{InertiaConfig, InertiaN};
use crate::scroll_smooth::SmoothScroll1D;
use crate::traits::Update;

/// Options for [`SmoothScroll`].
#[derive(Clone, Debug)]
pub struct SmoothScrollOptions {
    /// Exponential smoothing strength (see [`SmoothScroll1D`]).
    pub lerp_factor: f32,
    /// Multiplier for wheel delta (before `add_delta`).
    pub wheel_multiplier: f32,
    /// Multiplier for touch drag delta.
    pub touch_multiplier: f32,
    /// Momentum after touch release.
    pub inertia_config: InertiaConfig,
    /// Arrow key step (pixels).
    pub line_height: f32,
    /// Fraction of viewport height for PageUp / PageDown / Space.
    pub page_fraction: f32,
}

impl Default for SmoothScrollOptions {
    fn default() -> Self {
        Self {
            lerp_factor: 8.0,
            wheel_multiplier: 1.0,
            touch_multiplier: 1.0,
            inertia_config: InertiaConfig::default_flick(),
            line_height: 64.0,
            page_fraction: 0.9,
        }
    }
}

struct Inner {
    core: SmoothScroll1D,
    inertia: InertiaN<[f32; 1]>,
    touch_active: bool,
    touch_last_y: f32,
    touch_last_t: f64,
    touch_last_velocity: f32,
    options: SmoothScrollOptions,
    reduced_motion: bool,
    attached: bool,
}

impl Inner {
    fn new(options: SmoothScrollOptions) -> Self {
        let max = compute_max_scroll().unwrap_or(0.0);
        let y = window_scroll_y().unwrap_or(0.0);
        Self {
            core: SmoothScroll1D::new(y, 0.0, max, options.lerp_factor),
            inertia: InertiaN::new(options.inertia_config.clone(), [0.0_f32]),
            touch_active: false,
            touch_last_y: 0.0,
            touch_last_t: 0.0,
            touch_last_velocity: 0.0,
            options,
            reduced_motion: prefers_reduced_motion(),
            attached: false,
        }
    }

    fn refresh_limits(&mut self) {
        if let Some(max) = compute_max_scroll() {
            self.core.set_limits(0.0, max);
            let y = self.core.target().clamp(0.0, max);
            self.core.set_target(y);
        }
    }

    fn tick(&mut self, dt: f32) {
        if !self.attached {
            return;
        }

        if !self.inertia.is_settled() {
            let before = self.inertia.position()[0];
            self.inertia.update(dt);
            let after = self.inertia.position()[0];
            self.core.add_delta(after - before);
        }

        if self.reduced_motion {
            self.core.snap_to_target();
        } else {
            self.core.update(dt);
        }

        if let Some(win) = web_sys::window() {
            let y = self.core.current() as f64;
            win.scroll_to_with_x_and_y(0.0, y);
        }
    }

    fn scroll_to(&mut self, y: f32, smooth: bool) {
        self.core.set_target(y);
        if !smooth || self.reduced_motion {
            self.core.snap_to_target();
            if let Some(win) = web_sys::window() {
                let y = self.core.current() as f64;
                win.scroll_to_with_x_and_y(0.0, y);
            }
        }
    }
}

/// Window smooth scroller (Lenis-style): exponential follow + full input.
pub struct SmoothScroll {
    state: Rc<RefCell<Inner>>,
    window_closures: Vec<(&'static str, Closure<dyn FnMut(Event)>)>,
    document_closures: Vec<(&'static str, Closure<dyn FnMut(Event)>)>,
    /// Media query for `prefers-reduced-motion` (listener removed in [`detach`](Self::detach)).
    mql: Option<MediaQueryList>,
    mql_closure: Option<Closure<dyn FnMut(Event)>>,
}

impl core::fmt::Debug for SmoothScroll {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SmoothScroll").finish_non_exhaustive()
    }
}

impl SmoothScroll {
    /// Build with default options; call [`Self::attach`] before [`Self::tick`].
    pub fn new(options: SmoothScrollOptions) -> Self {
        Self {
            state: Rc::new(RefCell::new(Inner::new(options))),
            window_closures: Vec::new(),
            document_closures: Vec::new(),
            mql: None,
            mql_closure: None,
        }
    }

    /// Install listeners and root CSS (`touch-action`, `overscroll-behavior`).
    pub fn attach(&mut self) -> Result<(), JsValue> {
        let mut inner = self.state.borrow_mut();
        if inner.attached {
            return Ok(());
        }

        let win = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
        let doc = win
            .document()
            .ok_or_else(|| JsValue::from_str("no document"))?;

        if let Some(html) = doc.document_element() {
            if let Ok(html_el) = html.clone().dyn_into::<HtmlElement>() {
                let _ = html_el.style().set_property("touch-action", "none");
                let _ = html_el.style().set_property("overscroll-behavior", "none");
            }
        }

        inner.reduced_motion = prefers_reduced_motion();
        inner.refresh_limits();
        if let Some(y) = window_scroll_y() {
            inner.core.sync_both(y);
        }
        inner.attached = true;
        drop(inner);

        self.register_listeners(&win)?;
        self.register_media_query(&win)?;
        Ok(())
    }

    /// Remove listeners and drop media-query handler.
    pub fn detach(&mut self) {
        let mut inner = self.state.borrow_mut();
        if !inner.attached {
            return;
        }
        inner.attached = false;
        drop(inner);

        if let Some(win) = web_sys::window() {
            let win_target: web_sys::EventTarget = win.clone().into();
            for (name, c) in self.window_closures.drain(..) {
                let _ = win_target
                    .remove_event_listener_with_callback(name, c.as_ref().unchecked_ref());
            }
            if let Some(doc) = win.document() {
                let doc_target: web_sys::EventTarget = doc.into();
                for (name, c) in self.document_closures.drain(..) {
                    let _ = doc_target
                        .remove_event_listener_with_callback(name, c.as_ref().unchecked_ref());
                }
            }
        }
        if let Some(mql) = self.mql.take() {
            if let Some(c) = self.mql_closure.take() {
                let _ =
                    mql.remove_event_listener_with_callback("change", c.as_ref().unchecked_ref());
            }
        }

        if let Some(win) = web_sys::window() {
            if let Some(doc) = win.document() {
                if let Some(html) = doc.document_element() {
                    if let Ok(html_el) = html.dyn_into::<HtmlElement>() {
                        let _ = html_el.style().remove_property("touch-action");
                        let _ = html_el.style().remove_property("overscroll-behavior");
                    }
                }
            }
        }
    }

    /// Advance physics and apply [`Window::scroll_to_with_x_and_y`].
    pub fn tick(&mut self, dt: f32) {
        self.state.borrow_mut().tick(dt);
    }

    /// Smoothed scroll position (pass to [`ScrollDriver::set_position`](crate::scroll::ScrollDriver::set_position)).
    pub fn current_scroll(&self) -> f32 {
        self.state.borrow().core.current()
    }

    /// Target scroll offset.
    pub fn target_scroll(&self) -> f32 {
        self.state.borrow().core.target()
    }

    /// Programmatic scroll. `smooth == false` or reduced motion snaps immediately.
    pub fn scroll_to(&mut self, y: f32, smooth: bool) {
        self.state.borrow_mut().scroll_to(y, smooth);
    }

    /// Recompute max scroll from layout (call after content size changes).
    pub fn refresh_limits(&mut self) {
        self.state.borrow_mut().refresh_limits();
    }

    fn register_media_query(&mut self, win: &web_sys::Window) -> Result<(), JsValue> {
        let mq = win.match_media("(prefers-reduced-motion: reduce)")?;
        let Some(mql) = mq else {
            return Ok(());
        };
        let state = self.state.clone();
        let closure = Closure::wrap(Box::new(move |_e: Event| {
            state.borrow_mut().reduced_motion = prefers_reduced_motion();
        }) as Box<dyn FnMut(Event)>);
        mql.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())?;
        self.mql = Some(mql);
        self.mql_closure = Some(closure);
        Ok(())
    }

    fn register_listeners(&mut self, win: &web_sys::Window) -> Result<(), JsValue> {
        let target: web_sys::EventTarget = win.clone().into();
        let state = self.state.clone();

        // wheel (non-passive)
        {
            let s = state.clone();
            let closure = Closure::wrap(Box::new(move |e: Event| {
                if let Ok(we) = e.dyn_into::<WheelEvent>() {
                    let dy = we.delta_y() as f32;
                    let mult = s.borrow().options.wheel_multiplier;
                    s.borrow_mut().core.add_delta(dy * mult);
                    we.prevent_default();
                }
            }) as Box<dyn FnMut(Event)>);
            let opts = AddEventListenerOptions::new();
            opts.set_passive(false);
            target.add_event_listener_with_callback_and_add_event_listener_options(
                "wheel",
                closure.as_ref().unchecked_ref(),
                &opts,
            )?;
            self.window_closures.push(("wheel", closure));
        }

        // keydown
        {
            let s = state.clone();
            let closure = Closure::wrap(Box::new(move |e: Event| {
                if let Ok(ke) = e.dyn_into::<KeyboardEvent>() {
                    let page = page_step(&s.borrow().options);
                    let line = s.borrow().options.line_height;
                    let key = ke.key();
                    let delta: Option<f32> = match key.as_str() {
                        " " => Some(if ke.shift_key() { -page } else { page }),
                        "PageDown" => Some(page),
                        "PageUp" => Some(-page),
                        "ArrowDown" => Some(line),
                        "ArrowUp" => Some(-line),
                        "Home" => {
                            let t = s.borrow().core.target();
                            Some(-t)
                        }
                        "End" => {
                            let max = compute_max_scroll().unwrap_or(0.0);
                            let t = s.borrow().core.target();
                            Some(max - t)
                        }
                        _ => None,
                    };
                    if let Some(d) = delta {
                        s.borrow_mut().core.add_delta(d);
                        ke.prevent_default();
                    }
                }
            }) as Box<dyn FnMut(Event)>);
            let opts = AddEventListenerOptions::new();
            opts.set_passive(false);
            target.add_event_listener_with_callback_and_add_event_listener_options(
                "keydown",
                closure.as_ref().unchecked_ref(),
                &opts,
            )?;
            self.window_closures.push(("keydown", closure));
        }

        // touch
        {
            let s = state.clone();
            let closure = Closure::wrap(Box::new(move |e: Event| {
                if let Ok(te) = e.dyn_into::<TouchEvent>() {
                    if let Some(t) = te.touches().get(0) {
                        let y = t.client_y() as f32;
                        let now = js_sys::Date::now();
                        let mut inner = s.borrow_mut();
                        if !inner.touch_active {
                            inner.touch_active = true;
                            inner.touch_last_y = y;
                            inner.touch_last_t = now;
                            inner.touch_last_velocity = 0.0;
                            inner.inertia.reset([0.0_f32]);
                        }
                        te.prevent_default();
                    }
                }
            }) as Box<dyn FnMut(Event)>);
            let opts = AddEventListenerOptions::new();
            opts.set_passive(false);
            target.add_event_listener_with_callback_and_add_event_listener_options(
                "touchstart",
                closure.as_ref().unchecked_ref(),
                &opts,
            )?;
            self.window_closures.push(("touchstart", closure));
        }

        {
            let s = state.clone();
            let closure = Closure::wrap(Box::new(move |e: Event| {
                if let Ok(te) = e.dyn_into::<TouchEvent>() {
                    if let Some(t) = te.touches().get(0) {
                        let y = t.client_y() as f32;
                        let now = js_sys::Date::now();
                        let tm = s.borrow().options.touch_multiplier;
                        let mut inner = s.borrow_mut();
                        if inner.touch_active {
                            let dt_ms = now - inner.touch_last_t;
                            let dy = y - inner.touch_last_y;
                            if dt_ms > 0.0 {
                                let dt_sec = (dt_ms / 1000.0) as f32;
                                inner.touch_last_velocity = -(dy / dt_sec) * tm;
                            }
                            inner.core.add_delta(-dy * tm);
                            inner.touch_last_y = y;
                            inner.touch_last_t = now;
                        }
                        te.prevent_default();
                    }
                }
            }) as Box<dyn FnMut(Event)>);
            let opts = AddEventListenerOptions::new();
            opts.set_passive(false);
            target.add_event_listener_with_callback_and_add_event_listener_options(
                "touchmove",
                closure.as_ref().unchecked_ref(),
                &opts,
            )?;
            self.window_closures.push(("touchmove", closure));
        }

        {
            let s = state.clone();
            let closure = Closure::wrap(Box::new(move |e: Event| {
                if let Ok(te) = e.dyn_into::<TouchEvent>() {
                    let mut inner = s.borrow_mut();
                    if inner.touch_active {
                        inner.touch_active = false;
                        let v = inner.touch_last_velocity;
                        let cfg = inner.options.inertia_config.clone();
                        inner.inertia = InertiaN::new(cfg, [0.0_f32]).with_velocity([v]);
                    }
                    te.prevent_default();
                }
            }) as Box<dyn FnMut(Event)>);
            let opts = AddEventListenerOptions::new();
            opts.set_passive(false);
            target.add_event_listener_with_callback_and_add_event_listener_options(
                "touchend",
                closure.as_ref().unchecked_ref(),
                &opts,
            )?;
            self.window_closures.push(("touchend", closure));
        }

        {
            let s = state.clone();
            let closure = Closure::wrap(Box::new(move |_e: Event| {
                s.borrow_mut().refresh_limits();
            }) as Box<dyn FnMut(Event)>);
            target.add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())?;
            self.window_closures.push(("resize", closure));
        }

        {
            let s = state.clone();
            let closure = Closure::wrap(Box::new(move |_e: Event| {
                if let Some(y) = hash_to_scroll_y() {
                    s.borrow_mut().scroll_to(y, true);
                }
            }) as Box<dyn FnMut(Event)>);
            target
                .add_event_listener_with_callback("hashchange", closure.as_ref().unchecked_ref())?;
            self.window_closures.push(("hashchange", closure));
        }

        // Anchor clicks: prevent instant jump, use history + smooth target
        {
            let s = state.clone();
            let win_hist = win.clone();
            let doc_target: web_sys::EventTarget = win.document().unwrap().into();
            let closure = Closure::wrap(Box::new(move |e: Event| {
                if let Some(t) = e.target() {
                    if let Ok(el) = t.dyn_into::<web_sys::Element>() {
                        if let Ok(Some(a)) = el.closest("a[href^='#']") {
                            if let Ok(anchor) = a.dyn_into::<web_sys::HtmlAnchorElement>() {
                                let href = anchor.href();
                                if let Ok(url) = web_sys::Url::new(&href) {
                                    let hash = url.hash();
                                    if hash.len() > 1 {
                                        if let Some(y) = element_offset_for_hash(&hash[1..]) {
                                            e.prevent_default();
                                            if let Ok(h) = win_hist.history() {
                                                let _ = h.push_state_with_url(
                                                    &JsValue::NULL,
                                                    "",
                                                    Some(&hash),
                                                );
                                            }
                                            s.borrow_mut().scroll_to(y, true);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }) as Box<dyn FnMut(Event)>);
            let opts = AddEventListenerOptions::new();
            opts.set_capture(true);
            opts.set_passive(false);
            doc_target.add_event_listener_with_callback_and_add_event_listener_options(
                "click",
                closure.as_ref().unchecked_ref(),
                &opts,
            )?;
            self.document_closures.push(("click", closure));
        }

        Ok(())
    }
}

impl Drop for SmoothScroll {
    fn drop(&mut self) {
        self.detach();
    }
}

fn prefers_reduced_motion() -> bool {
    web_sys::window()
        .and_then(|w| {
            w.match_media("(prefers-reduced-motion: reduce)")
                .ok()
                .flatten()
        })
        .map(|m| m.matches())
        .unwrap_or(false)
}

fn window_scroll_y() -> Option<f32> {
    web_sys::window().map(|w| w.scroll_y().unwrap_or(0.0) as f32)
}

fn compute_max_scroll() -> Option<f32> {
    let win = web_sys::window()?;
    let doc = win.document()?;
    let doc_el = doc.document_element()?;
    let sh = doc_el.scroll_height() as f32;
    let ih = win
        .inner_height()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0) as f32;
    Some((sh - ih).max(0.0))
}

fn hash_to_scroll_y() -> Option<f32> {
    let win = web_sys::window()?;
    let hash = win.location().hash().ok()?;
    if hash.is_empty() || hash == "#" {
        return Some(0.0);
    }
    let id = hash.trim_start_matches('#');
    element_offset_for_hash(id)
}

fn element_offset_for_hash(id: &str) -> Option<f32> {
    let win = web_sys::window()?;
    let doc = win.document()?;
    let el = doc.get_element_by_id(id)?;
    let rect = el.get_bounding_client_rect();
    Some(rect.top() as f32 + win.scroll_y().ok()? as f32)
}

fn page_step(options: &SmoothScrollOptions) -> f32 {
    web_sys::window()
        .and_then(|w| w.inner_height().ok())
        .and_then(|v| v.as_f64())
        .map(|h| h as f32 * options.page_fraction)
        .unwrap_or(600.0)
}
