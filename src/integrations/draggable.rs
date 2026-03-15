//! DOM-bound draggable element.
//!
//! Wraps [`DragState`](crate::drag::DragState) with pointer event listeners
//! on a DOM element. The pure-math drag tracking lives in `src/drag.rs`;
//! this module provides the DOM binding.
//!
//! Requires the `wasm-dom` feature.

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Element, Event, PointerEvent};

use crate::drag::{DragConstraints, DragState};

use std::cell::RefCell;
use std::rc::Rc;

/// DOM-bound draggable element.
///
/// Attaches pointer event listeners to an element and feeds them into
/// a [`DragState`]. Drop or call [`unbind`](Self::unbind) to remove listeners.
pub struct Draggable {
    state: Rc<RefCell<DragState>>,
    element: Element,
    closures: Vec<(String, Closure<dyn FnMut(Event)>)>,
    last_time: Rc<RefCell<f64>>,
}

impl core::fmt::Debug for Draggable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Draggable")
            .field("state", &*self.state.borrow())
            .finish()
    }
}

impl Draggable {
    /// Bind pointer event listeners to the element.
    pub fn bind(element: &Element) -> Self {
        Self::bind_with_constraints(element, DragConstraints::default())
    }

    /// Bind with custom drag constraints.
    pub fn bind_with_constraints(element: &Element, constraints: DragConstraints) -> Self {
        let state = Rc::new(RefCell::new(DragState::new().with_constraints(constraints)));
        let last_time = Rc::new(RefCell::new(0.0_f64));
        let mut closures: Vec<(String, Closure<dyn FnMut(Event)>)> = Vec::new();

        let target: web_sys::EventTarget = element.clone().into();

        // pointerdown
        {
            let s = state.clone();
            let lt = last_time.clone();
            let closure = Closure::wrap(Box::new(move |e: Event| {
                if let Ok(pe) = e.dyn_into::<PointerEvent>() {
                    s.borrow_mut().on_pointer_down(pe.client_x() as f32, pe.client_y() as f32);
                    *lt.borrow_mut() = js_sys::Date::now();
                }
            }) as Box<dyn FnMut(Event)>);
            let _ = target.add_event_listener_with_callback("pointerdown", closure.as_ref().unchecked_ref());
            closures.push(("pointerdown".into(), closure));
        }

        // pointermove
        {
            let s = state.clone();
            let lt = last_time.clone();
            let closure = Closure::wrap(Box::new(move |e: Event| {
                if let Ok(pe) = e.dyn_into::<PointerEvent>() {
                    let now = js_sys::Date::now();
                    let prev = *lt.borrow();
                    let dt = if prev > 0.0 { ((now - prev) / 1000.0) as f32 } else { 1.0 / 60.0 };
                    *lt.borrow_mut() = now;
                    s.borrow_mut().on_pointer_move(pe.client_x() as f32, pe.client_y() as f32, dt);
                }
            }) as Box<dyn FnMut(Event)>);
            let _ = target.add_event_listener_with_callback("pointermove", closure.as_ref().unchecked_ref());
            closures.push(("pointermove".into(), closure));
        }

        // pointerup (on window to catch releases outside the element)
        {
            let s = state.clone();
            let closure = Closure::wrap(Box::new(move |e: Event| {
                if let Ok(_pe) = e.dyn_into::<PointerEvent>() {
                    let _ = s.borrow_mut().on_pointer_up();
                }
            }) as Box<dyn FnMut(Event)>);
            if let Some(win) = web_sys::window() {
                let _ = win.add_event_listener_with_callback("pointerup", closure.as_ref().unchecked_ref());
            }
            closures.push(("pointerup".into(), closure));
        }

        Self {
            state,
            element: element.clone(),
            closures,
            last_time,
        }
    }

    /// Get a snapshot of the current drag state.
    pub fn state(&self) -> DragState {
        self.state.borrow().clone()
    }

    /// Current position.
    pub fn position(&self) -> [f32; 2] {
        self.state.borrow().position()
    }

    /// Whether the element is currently being dragged.
    pub fn is_dragging(&self) -> bool {
        self.state.borrow().is_dragging()
    }

    /// Remove all event listeners. Consumes the draggable.
    pub fn unbind(self) {
        let target: web_sys::EventTarget = self.element.into();
        for (name, closure) in &self.closures {
            if name == "pointerup" {
                // Was bound to window
                if let Some(win) = web_sys::window() {
                    let _ = win.remove_event_listener_with_callback(
                        name,
                        closure.as_ref().unchecked_ref(),
                    );
                }
            } else {
                let _ = target.remove_event_listener_with_callback(
                    name,
                    closure.as_ref().unchecked_ref(),
                );
            }
        }
    }
}
