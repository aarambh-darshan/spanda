//! Unified pointer/touch/mouse event normaliser.
//!
//! `Observer` attaches event listeners to a DOM element and normalises
//! `PointerEvent`, `MouseEvent`, and `TouchEvent` into a common
//! [`PointerData`](crate::drag::PointerData) struct.
//!
//! Requires the `wasm-dom` feature.
#![allow(clippy::type_complexity)]

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{Element, Event, EventTarget};

use crate::drag::PointerData;

/// Callbacks for input events.
pub struct ObserverCallbacks {
    /// Called on pointer/mouse/touch down.
    pub on_press: Option<Box<dyn FnMut(PointerData)>>,
    /// Called on pointer/mouse/touch move.
    pub on_move: Option<Box<dyn FnMut(PointerData)>>,
    /// Called on pointer/mouse/touch up.
    pub on_release: Option<Box<dyn FnMut(PointerData)>>,
    /// Called on wheel event. Arguments: `(delta_x, delta_y)`.
    pub on_wheel: Option<Box<dyn FnMut(f32, f32)>>,
}

/// Configuration options for [`Observer`].
///
/// GSAP equivalent: Observer `tolerance`, `preventDefault`, `allowClicks`.
#[derive(Debug, Clone)]
pub struct ObserverOptions {
    /// Minimum movement distance (in pixels) before triggering move callbacks.
    ///
    /// GSAP equivalent: `tolerance`. Default is 0.0 (no tolerance).
    pub tolerance: f32,

    /// Whether to call `preventDefault()` on events.
    ///
    /// GSAP equivalent: `preventDefault`. Default is false.
    pub prevent_default: bool,

    /// Whether to allow click events to pass through.
    ///
    /// If true, small movements (below tolerance) are treated as clicks.
    /// GSAP equivalent: `allowClicks`. Default is true.
    pub allow_clicks: bool,

    /// Event capture phase. If true, uses capture phase instead of bubbling.
    ///
    /// GSAP equivalent: `capture`. Default is false.
    pub capture: bool,

    /// Whether to handle touch events on touch devices.
    ///
    /// Default is true.
    pub touch_enabled: bool,

    /// Lock axis to horizontal or vertical only.
    ///
    /// - `None` - no axis lock (default)
    /// - `Some(true)` - lock to horizontal (x-axis)
    /// - `Some(false)` - lock to vertical (y-axis)
    pub lock_axis: Option<bool>,
}

impl Default for ObserverOptions {
    fn default() -> Self {
        Self {
            tolerance: 0.0,
            prevent_default: false,
            allow_clicks: true,
            capture: false,
            touch_enabled: true,
            lock_axis: None,
        }
    }
}

impl ObserverOptions {
    /// Create new options with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the movement tolerance in pixels.
    pub fn tolerance(mut self, pixels: f32) -> Self {
        self.tolerance = pixels;
        self
    }

    /// Set whether to prevent default browser behavior.
    pub fn prevent_default(mut self, prevent: bool) -> Self {
        self.prevent_default = prevent;
        self
    }

    /// Set whether to allow click events.
    pub fn allow_clicks(mut self, allow: bool) -> Self {
        self.allow_clicks = allow;
        self
    }

    /// Set whether to use capture phase.
    pub fn capture(mut self, capture: bool) -> Self {
        self.capture = capture;
        self
    }

    /// Enable or disable touch handling.
    pub fn touch_enabled(mut self, enabled: bool) -> Self {
        self.touch_enabled = enabled;
        self
    }

    /// Lock movement to horizontal axis only.
    pub fn lock_horizontal(mut self) -> Self {
        self.lock_axis = Some(true);
        self
    }

    /// Lock movement to vertical axis only.
    pub fn lock_vertical(mut self) -> Self {
        self.lock_axis = Some(false);
        self
    }
}

impl core::fmt::Debug for ObserverCallbacks {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ObserverCallbacks")
            .field("on_press", &self.on_press.as_ref().map(|_| "<fn>"))
            .field("on_move", &self.on_move.as_ref().map(|_| "<fn>"))
            .field("on_release", &self.on_release.as_ref().map(|_| "<fn>"))
            .field("on_wheel", &self.on_wheel.as_ref().map(|_| "<fn>"))
            .finish()
    }
}

/// Unified input event observer for DOM elements.
///
/// Attaches pointer, mouse, touch, and wheel event listeners and normalises
/// them into [`PointerData`] callbacks. Store the `Observer` to keep the
/// listeners alive; call [`Observer::unbind`] to remove them.
pub struct Observer {
    target: EventTarget,
    closures: Vec<(String, Closure<dyn FnMut(Event)>)>,
}

impl core::fmt::Debug for Observer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Observer")
            .field("listeners", &self.closures.len())
            .finish()
    }
}

fn pointer_data_from_pointer(e: &web_sys::PointerEvent) -> PointerData {
    PointerData {
        x: e.client_x() as f32,
        y: e.client_y() as f32,
        pressure: e.pressure(),
        pointer_id: e.pointer_id(),
    }
}

#[allow(dead_code)]
fn pointer_data_from_mouse(e: &web_sys::MouseEvent) -> PointerData {
    PointerData {
        x: e.client_x() as f32,
        y: e.client_y() as f32,
        pressure: 0.5,
        pointer_id: 0,
    }
}

#[allow(dead_code)]
fn pointer_data_from_touch(t: &web_sys::Touch) -> PointerData {
    PointerData {
        x: t.client_x() as f32,
        y: t.client_y() as f32,
        pressure: t.force(),
        pointer_id: t.identifier(),
    }
}

impl Observer {
    /// Bind event listeners to the given element.
    ///
    /// Prefers `PointerEvent` (modern browsers), falls back to
    /// `MouseEvent` + `TouchEvent`. The closures are kept alive inside
    /// the returned `Observer`; drop it or call [`unbind`](Self::unbind) to remove them.
    pub fn bind(element: &Element, callbacks: ObserverCallbacks) -> Self {
        Self::bind_with_options(element, callbacks, ObserverOptions::default())
    }

    /// Bind event listeners with custom options.
    ///
    /// GSAP equivalent: `Observer.create({ tolerance, preventDefault, ... })`
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let observer = Observer::bind_with_options(
    ///     &element,
    ///     callbacks,
    ///     ObserverOptions::new()
    ///         .tolerance(10.0)
    ///         .prevent_default(true),
    /// );
    /// ```
    pub fn bind_with_options(
        element: &Element,
        callbacks: ObserverCallbacks,
        options: ObserverOptions,
    ) -> Self {
        use std::cell::RefCell;
        use std::rc::Rc;

        let target: EventTarget = element.clone().into();
        let mut closures: Vec<(String, Closure<dyn FnMut(Event)>)> = Vec::new();

        let on_press = Rc::new(RefCell::new(callbacks.on_press));
        let on_move = Rc::new(RefCell::new(callbacks.on_move));
        let on_release = Rc::new(RefCell::new(callbacks.on_release));
        let on_wheel = Rc::new(RefCell::new(callbacks.on_wheel));

        // Track initial position for tolerance checking
        let start_pos: Rc<RefCell<Option<(f32, f32)>>> = Rc::new(RefCell::new(None));
        let tolerance = options.tolerance;
        let prevent_default = options.prevent_default;
        let lock_axis = options.lock_axis;

        // --- Pointer events ---
        {
            let cb = on_press.clone();
            let start = start_pos.clone();
            let prevent = prevent_default;
            let closure = Closure::wrap(Box::new(move |e: Event| {
                if prevent {
                    e.prevent_default();
                }
                if let Ok(pe) = e.dyn_into::<web_sys::PointerEvent>() {
                    let data = pointer_data_from_pointer(&pe);
                    *start.borrow_mut() = Some((data.x, data.y));
                    if let Some(ref mut f) = *cb.borrow_mut() {
                        f(data);
                    }
                }
            }) as Box<dyn FnMut(Event)>);
            let _ = target
                .add_event_listener_with_callback("pointerdown", closure.as_ref().unchecked_ref());
            closures.push(("pointerdown".into(), closure));
        }

        {
            let cb = on_move.clone();
            let start = start_pos.clone();
            let prevent = prevent_default;
            let closure = Closure::wrap(Box::new(move |e: Event| {
                if prevent {
                    e.prevent_default();
                }
                if let Ok(pe) = e.dyn_into::<web_sys::PointerEvent>() {
                    let mut data = pointer_data_from_pointer(&pe);

                    // Apply tolerance check
                    if tolerance > 0.0 {
                        if let Some((sx, sy)) = *start.borrow() {
                            let dx = data.x - sx;
                            let dy = data.y - sy;
                            let dist = (dx * dx + dy * dy).sqrt();
                            if dist < tolerance {
                                return; // Below tolerance, don't fire callback
                            }
                        }
                    }

                    // Apply axis lock
                    if let Some(horizontal) = lock_axis {
                        if let Some((sx, sy)) = *start.borrow() {
                            if horizontal {
                                data.y = sy; // Lock Y
                            } else {
                                data.x = sx; // Lock X
                            }
                        }
                    }

                    if let Some(ref mut f) = *cb.borrow_mut() {
                        f(data);
                    }
                }
            }) as Box<dyn FnMut(Event)>);
            let _ = target
                .add_event_listener_with_callback("pointermove", closure.as_ref().unchecked_ref());
            closures.push(("pointermove".into(), closure));
        }

        {
            let cb = on_release.clone();
            let start = start_pos.clone();
            let prevent = prevent_default;
            let closure = Closure::wrap(Box::new(move |e: Event| {
                if prevent {
                    e.prevent_default();
                }
                if let Ok(pe) = e.dyn_into::<web_sys::PointerEvent>() {
                    *start.borrow_mut() = None;
                    if let Some(ref mut f) = *cb.borrow_mut() {
                        f(pointer_data_from_pointer(&pe));
                    }
                }
            }) as Box<dyn FnMut(Event)>);
            let _ = target
                .add_event_listener_with_callback("pointerup", closure.as_ref().unchecked_ref());
            closures.push(("pointerup".into(), closure));
        }

        // --- Wheel ---
        {
            let cb = on_wheel;
            let prevent = prevent_default;
            let closure = Closure::wrap(Box::new(move |e: Event| {
                if prevent {
                    e.prevent_default();
                }
                if let Ok(we) = e.dyn_into::<web_sys::WheelEvent>() {
                    if let Some(ref mut f) = *cb.borrow_mut() {
                        f(we.delta_x() as f32, we.delta_y() as f32);
                    }
                }
            }) as Box<dyn FnMut(Event)>);
            let _ =
                target.add_event_listener_with_callback("wheel", closure.as_ref().unchecked_ref());
            closures.push(("wheel".into(), closure));
        }

        Self { target, closures }
    }

    /// Remove all event listeners. Consumes the observer.
    pub fn unbind(self) {
        for (event_name, closure) in &self.closures {
            let _ = self
                .target
                .remove_event_listener_with_callback(event_name, closure.as_ref().unchecked_ref());
        }
        // closures are dropped here, releasing the JS closures
    }
}
