//! Spring-driven smooth scrolling.
//!
//! `ScrollSmoother` intercepts native scroll events, feeds the scroll position
//! into a [`Spring`](crate::spring::Spring), and writes the smoothed output to
//! a content element's `transform: translateY()`.
//!
//! Requires the `wasm-dom` feature.

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{Event, HtmlElement};

use crate::spring::{Spring, SpringConfig};
use crate::traits::Update;

/// Smooth-scrolling overlay for a content element.
///
/// Intercepts native scroll and outputs spring-smoothed position.
/// Call [`tick`](Self::tick) from your `requestAnimationFrame` loop.
pub struct ScrollSmoother {
    spring: Spring,
    content_element: HtmlElement,
    scroll_closure: Option<Closure<dyn FnMut(Event)>>,
    attached: bool,
    target_y: f32,
}

impl core::fmt::Debug for ScrollSmoother {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ScrollSmoother")
            .field("attached", &self.attached)
            .field("position", &self.spring.position())
            .field("target_y", &self.target_y)
            .finish()
    }
}

impl ScrollSmoother {
    /// Create a new scroll smoother for the given content element.
    ///
    /// The spring configuration controls the smoothing feel. Use
    /// `SpringConfig::gentle()` for buttery smooth, `SpringConfig::stiff()`
    /// for more responsive.
    pub fn new(content_element: HtmlElement, config: SpringConfig) -> Self {
        Self {
            spring: Spring::new(config),
            content_element,
            scroll_closure: None,
            attached: false,
            target_y: 0.0,
        }
    }

    /// Attach the scroll listener.
    ///
    /// Sets `overflow: hidden` on the content's parent (the wrapper) and
    /// listens for scroll events on the window to feed the spring.
    pub fn attach(&mut self) {
        if self.attached {
            return;
        }

        // Set up the content element for transform-based scrolling
        let _ = self
            .content_element
            .style()
            .set_property("will-change", "transform");

        // Create a shared reference to target_y that the closure can write to.
        // We use a simple approach: store a leaked Box to share state.
        // On detach, we stop reading it, so the leak is bounded.
        let target = std::rc::Rc::new(std::cell::Cell::new(0.0_f32));
        let target_clone = target.clone();

        let closure = Closure::wrap(Box::new(move |_e: Event| {
            if let Some(win) = web_sys::window() {
                let scroll_y = win.scroll_y().unwrap_or(0.0) as f32;
                target_clone.set(scroll_y);
            }
        }) as Box<dyn FnMut(Event)>);

        if let Some(win) = web_sys::window() {
            let _ =
                win.add_event_listener_with_callback("scroll", closure.as_ref().unchecked_ref());
        }

        // Store the Rc so tick() can read it
        self.scroll_closure = Some(closure);
        self.attached = true;

        // We'll read from window.scrollY in tick() instead of the Rc,
        // since we need the latest value each frame anyway.
    }

    /// Detach the scroll listener and restore original styles.
    pub fn detach(&mut self) {
        if !self.attached {
            return;
        }

        if let Some(closure) = self.scroll_closure.take() {
            if let Some(win) = web_sys::window() {
                let _ = win.remove_event_listener_with_callback(
                    "scroll",
                    closure.as_ref().unchecked_ref(),
                );
            }
        }

        let _ = self.content_element.style().remove_property("will-change");
        let _ = self.content_element.style().remove_property("transform");

        self.attached = false;
    }

    /// Tick the spring and apply the smoothed transform.
    ///
    /// Call this from your `requestAnimationFrame` loop. `dt` is seconds
    /// since the last frame.
    pub fn tick(&mut self, dt: f32) {
        if !self.attached {
            return;
        }

        // Read current scroll position
        if let Some(win) = web_sys::window() {
            self.target_y = win.scroll_y().unwrap_or(0.0) as f32;
        }

        self.spring.set_target(self.target_y);
        self.spring.update(dt);

        let y = self.spring.position();
        let _ = self
            .content_element
            .style()
            .set_property("transform", &format!("translateY(-{y}px)"));
    }

    /// Current smoothed scroll position (px).
    pub fn position(&self) -> f32 {
        self.spring.position()
    }

    /// Target (actual native) scroll position (px).
    pub fn target(&self) -> f32 {
        self.target_y
    }

    /// Mutable access to the spring config for live tuning.
    pub fn spring_config_mut(&mut self) -> &mut SpringConfig {
        &mut self.spring.config
    }
}
