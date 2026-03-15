//! Unified pointer/touch/mouse event normaliser.
//!
//! `Observer` attaches event listeners to a DOM element and normalises
//! `PointerEvent`, `MouseEvent`, and `TouchEvent` into a common
//! [`PointerData`](crate::drag::PointerData) struct.
//!
//! Requires the `wasm-dom` feature.

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
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

fn pointer_data_from_mouse(e: &web_sys::MouseEvent) -> PointerData {
    PointerData {
        x: e.client_x() as f32,
        y: e.client_y() as f32,
        pressure: 0.5,
        pointer_id: 0,
    }
}

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
        use std::cell::RefCell;
        use std::rc::Rc;

        let target: EventTarget = element.clone().into();
        let mut closures: Vec<(String, Closure<dyn FnMut(Event)>)> = Vec::new();

        let on_press = Rc::new(RefCell::new(callbacks.on_press));
        let on_move = Rc::new(RefCell::new(callbacks.on_move));
        let on_release = Rc::new(RefCell::new(callbacks.on_release));
        let on_wheel = Rc::new(RefCell::new(callbacks.on_wheel));

        // --- Pointer events ---
        {
            let cb = on_press.clone();
            let closure = Closure::wrap(Box::new(move |e: Event| {
                if let Ok(pe) = e.dyn_into::<web_sys::PointerEvent>() {
                    if let Some(ref mut f) = *cb.borrow_mut() {
                        f(pointer_data_from_pointer(&pe));
                    }
                }
            }) as Box<dyn FnMut(Event)>);
            let _ = target.add_event_listener_with_callback("pointerdown", closure.as_ref().unchecked_ref());
            closures.push(("pointerdown".into(), closure));
        }

        {
            let cb = on_move.clone();
            let closure = Closure::wrap(Box::new(move |e: Event| {
                if let Ok(pe) = e.dyn_into::<web_sys::PointerEvent>() {
                    if let Some(ref mut f) = *cb.borrow_mut() {
                        f(pointer_data_from_pointer(&pe));
                    }
                }
            }) as Box<dyn FnMut(Event)>);
            let _ = target.add_event_listener_with_callback("pointermove", closure.as_ref().unchecked_ref());
            closures.push(("pointermove".into(), closure));
        }

        {
            let cb = on_release.clone();
            let closure = Closure::wrap(Box::new(move |e: Event| {
                if let Ok(pe) = e.dyn_into::<web_sys::PointerEvent>() {
                    if let Some(ref mut f) = *cb.borrow_mut() {
                        f(pointer_data_from_pointer(&pe));
                    }
                }
            }) as Box<dyn FnMut(Event)>);
            let _ = target.add_event_listener_with_callback("pointerup", closure.as_ref().unchecked_ref());
            closures.push(("pointerup".into(), closure));
        }

        // --- Wheel ---
        {
            let cb = on_wheel;
            let closure = Closure::wrap(Box::new(move |e: Event| {
                if let Ok(we) = e.dyn_into::<web_sys::WheelEvent>() {
                    if let Some(ref mut f) = *cb.borrow_mut() {
                        f(we.delta_x() as f32, we.delta_y() as f32);
                    }
                }
            }) as Box<dyn FnMut(Event)>);
            let _ = target.add_event_listener_with_callback("wheel", closure.as_ref().unchecked_ref());
            closures.push(("wheel".into(), closure));
        }

        Self { target, closures }
    }

    /// Remove all event listeners. Consumes the observer.
    pub fn unbind(self) {
        for (event_name, closure) in &self.closures {
            let _ = self.target.remove_event_listener_with_callback(
                event_name,
                closure.as_ref().unchecked_ref(),
            );
        }
        // closures are dropped here, releasing the JS closures
    }
}
