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
}

#[wasm_bindgen]
impl RafDriver {
    /// Create a new RAF-driven animation driver.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: AnimationDriver::new(),
            last_timestamp: None,
        }
    }

    /// Tick all animations using a browser `performance.now()` timestamp in
    /// **milliseconds**.
    ///
    /// On the first call, `dt` is assumed to be `0.0`.  Subsequent calls
    /// compute `dt` from the difference between timestamps.
    pub fn tick(&mut self, timestamp_ms: f64) {
        let dt = match self.last_timestamp {
            Some(last) => ((timestamp_ms - last) / 1000.0) as f32, // ms → seconds
            None => 0.0,
        };
        self.last_timestamp = Some(timestamp_ms);
        self.inner.tick(dt.max(0.0));
    }

    /// Number of active animations.
    pub fn active_count(&self) -> usize {
        self.inner.active_count()
    }

    /// Reset the timestamp tracking (e.g. after the tab was hidden).
    pub fn reset_timestamp(&mut self) {
        self.last_timestamp = None;
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
            .finish()
    }
}
