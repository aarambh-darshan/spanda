//! WASM integration — `requestAnimationFrame` driver.
//!
//! Activate with `features = ["wasm"]` in your `Cargo.toml`.
//!
//! # Usage
//!
//! ```rust,ignore
//! use spanda::integrations::wasm::RafDriver;
//!
//! let mut driver = RafDriver::new();
//! // Call driver.tick(timestamp_ms) from your rAF callback.
//! ```
//!
//! # Automatic rAF Loop
//!
//! Use [`start_raf_loop`] to start a self-scheduling `requestAnimationFrame`
//! loop that calls your closure each frame:
//!
//! ```rust,ignore
//! use spanda::integrations::wasm::{RafDriver, start_raf_loop};
//!
//! let driver = std::rc::Rc::new(std::cell::RefCell::new(RafDriver::new()));
//! let driver_clone = driver.clone();
//!
//! start_raf_loop(move |timestamp_ms| {
//!     driver_clone.borrow_mut().tick(timestamp_ms);
//! });
//! ```

use wasm_bindgen::prelude::*;

use crate::driver::AnimationDriver;

// ── RafDriver ────────────────────────────────────────────────────────────────

/// A `requestAnimationFrame`-aware animation driver for web/WASM targets.
///
/// Wraps an [`AnimationDriver`] and converts browser timestamps (milliseconds)
/// into delta-time seconds automatically.
///
/// # Typical usage
///
/// ```rust,ignore
/// use wasm_bindgen::prelude::*;
/// use spanda::integrations::wasm::RafDriver;
/// use spanda::tween::Tween;
/// use spanda::easing::Easing;
///
/// #[wasm_bindgen]
/// pub struct App {
///     driver: RafDriver,
/// }
///
/// #[wasm_bindgen]
/// impl App {
///     #[wasm_bindgen(constructor)]
///     pub fn new() -> Self {
///         let mut driver = RafDriver::new();
///         driver.add(
///             Tween::new(0.0_f32, 500.0)
///                 .duration(1.5)
///                 .easing(Easing::EaseOutBounce)
///                 .build(),
///         );
///         Self { driver }
///     }
///
///     pub fn tick(&mut self, timestamp_ms: f64) {
///         self.driver.tick(timestamp_ms);
///     }
/// }
/// ```
#[wasm_bindgen]
pub struct RafDriver {
    inner: AnimationDriver,
    last_timestamp: Option<f64>,
    paused: bool,
    time_scale: f32,
}

#[wasm_bindgen]
impl RafDriver {
    /// Create a new RAF-driven animation driver.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: AnimationDriver::new(),
            last_timestamp: None,
            paused: false,
            time_scale: 1.0,
        }
    }

    /// Tick all animations using a browser `performance.now()` timestamp in
    /// **milliseconds**.
    ///
    /// On the first call, `dt` is assumed to be `0.0`.  Subsequent calls
    /// compute `dt` from the difference between timestamps.
    ///
    /// If the driver is paused, this is a no-op (timestamp tracking still
    /// advances to avoid jumps on resume).
    pub fn tick(&mut self, timestamp_ms: f64) {
        if self.paused {
            self.last_timestamp = Some(timestamp_ms);
            return;
        }

        let dt = match self.last_timestamp {
            Some(last) => ((timestamp_ms - last) / 1000.0) as f32, // ms → seconds
            None => 0.0,
        };
        self.last_timestamp = Some(timestamp_ms);

        // Cap dt to avoid huge jumps after tab switch (> 500ms → clamp)
        let dt = dt.max(0.0).min(0.5) * self.time_scale;
        self.inner.tick(dt);
    }

    /// Number of active animations.
    pub fn active_count(&self) -> usize {
        self.inner.active_count()
    }

    /// Reset the timestamp tracking (e.g. after the tab was hidden).
    pub fn reset_timestamp(&mut self) {
        self.last_timestamp = None;
    }

    /// Pause animation playback.  Timestamps still advance to avoid jumps.
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resume animation playback.
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Whether the driver is currently paused.
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Set the time scale for all animations.
    ///
    /// Values > 1.0 speed up, < 1.0 slow down, 0.0 effectively pauses.
    pub fn set_time_scale(&mut self, scale: f32) {
        self.time_scale = scale.max(0.0);
    }

    /// Get the current time scale.
    pub fn get_time_scale(&self) -> f32 {
        self.time_scale
    }

    /// Handle a page visibility change.
    ///
    /// Call this when the `visibilitychange` event fires.  When the page
    /// becomes hidden, the timestamp is reset so that resuming doesn't cause
    /// a huge time jump.
    pub fn on_visibility_change(&mut self, hidden: bool) {
        if hidden {
            self.last_timestamp = None;
        }
    }
}

impl RafDriver {
    /// Add an animation (Rust-side only — not exposed to JS).
    ///
    /// Use this from Rust code to add tweens, springs, etc.
    pub fn add<A: crate::traits::Update + 'static>(
        &mut self,
        animation: A,
    ) -> crate::driver::AnimationId {
        self.inner.add(animation)
    }

    /// Cancel an animation by ID.
    pub fn cancel(&mut self, id: crate::driver::AnimationId) {
        self.inner.cancel(id);
    }

    /// Cancel all animations.
    pub fn cancel_all(&mut self) {
        self.inner.cancel_all();
    }

    /// Get a reference to the inner `AnimationDriver`.
    pub fn driver(&self) -> &AnimationDriver {
        &self.inner
    }

    /// Get a mutable reference to the inner `AnimationDriver`.
    pub fn driver_mut(&mut self) -> &mut AnimationDriver {
        &mut self.inner
    }
}

impl Default for RafDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Debug for RafDriver {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RafDriver")
            .field("active_count", &self.inner.active_count())
            .field("last_timestamp", &self.last_timestamp)
            .field("paused", &self.paused)
            .field("time_scale", &self.time_scale)
            .finish()
    }
}

// ── start_raf_loop ──────────────────────────────────────────────────────────

/// Start a self-scheduling `requestAnimationFrame` loop.
///
/// The provided closure is called every frame with the high-resolution
/// timestamp (in milliseconds). The loop continues until the closure is
/// dropped.
///
/// # Example
///
/// ```rust,ignore
/// use spanda::integrations::wasm::start_raf_loop;
/// use std::rc::Rc;
/// use std::cell::RefCell;
///
/// let driver = Rc::new(RefCell::new(RafDriver::new()));
/// let d = driver.clone();
///
/// start_raf_loop(move |ts| {
///     d.borrow_mut().tick(ts);
/// });
/// ```
pub fn start_raf_loop(mut callback: impl FnMut(f64) + 'static) {
    use std::cell::RefCell;
    use std::rc::Rc;

    let f: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    // Use Closure::wrap (not Closure::new) — produces a 'static Closure
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move |timestamp: f64| {
        callback(timestamp);

        // Schedule next frame
        if let Some(ref closure) = *f.borrow() {
            let _ = web_sys_request_animation_frame(closure);
        }
    }) as Box<dyn FnMut(f64)>));

    // Kick off the first frame — scope the borrow so Ref is dropped before g
    {
        let guard = g.borrow();
        if let Some(ref closure) = *guard {
            let _ = web_sys_request_animation_frame(closure);
        }
    }
}

/// Call `window.requestAnimationFrame`.
fn web_sys_request_animation_frame(closure: &Closure<dyn FnMut(f64)>) -> Result<i32, JsValue> {
    js_sys::Reflect::get(&js_sys::global(), &"requestAnimationFrame".into())
        .and_then(|raf| {
            let raf = js_sys::Function::from(raf);
            raf.call1(&JsValue::NULL, closure.as_ref().unchecked_ref())
        })
        .map(|v| v.as_f64().unwrap_or(0.0) as i32)
}
