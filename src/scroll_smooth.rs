//! One-dimensional Lenis-style smooth scroll **core** (no DOM).
//!
//! [`SmoothScroll1D`] keeps a **target** scroll offset and a **current** value that
//! follows it using **frame-rate independent** exponential decay (not a fixed lerp
//! factor per frame).
//!
//! Use this from the `wasm-dom` [`SmoothScroll`](crate::integrations::smooth_scroll::SmoothScroll)
//! integration or any host that applies the resulting [`SmoothScroll1D::current`] to a scroll container.

#[cfg(not(feature = "std"))]
#[allow(unused_imports)]
use num_traits::Float as _;

/// Portable smooth scroll state: `current` eases toward `target` exponentially.
#[derive(Clone, Debug)]
pub struct SmoothScroll1D {
    current: f32,
    target: f32,
    min: f32,
    max: f32,
    /// Strength of easing toward `target` (higher = snappier). Scaled for a 60fps reference.
    lerp_factor: f32,
}

impl SmoothScroll1D {
    /// Create a new state with both `current` and `target` at `initial`, clamped to `[min, max]`.
    pub fn new(initial: f32, min: f32, max: f32, lerp_factor: f32) -> Self {
        let mut s = Self {
            current: initial,
            target: initial,
            min,
            max,
            lerp_factor,
        };
        s.clamp_both();
        s
    }

    /// Smoothed scroll position (what to render / pass to `ScrollDriver`).
    #[inline]
    pub fn current(&self) -> f32 {
        self.current
    }

    /// Target scroll offset (after input and `add_delta`).
    #[inline]
    pub fn target(&self) -> f32 {
        self.target
    }

    /// Exponential smoothing factor (tuning knob).
    #[inline]
    pub fn lerp_factor(&self) -> f32 {
        self.lerp_factor
    }

    /// Set exponential smoothing strength (see struct docs).
    #[inline]
    pub fn set_lerp_factor(&mut self, lerp_factor: f32) {
        self.lerp_factor = lerp_factor;
    }

    /// Scroll range lower bound.
    #[inline]
    pub fn min(&self) -> f32 {
        self.min
    }

    /// Scroll range upper bound.
    #[inline]
    pub fn max(&self) -> f32 {
        self.max
    }

    /// Set scroll limits and clamp `current` / `target`.
    pub fn set_limits(&mut self, min: f32, max: f32) {
        self.min = min;
        self.max = max;
        self.clamp_both();
    }

    /// Set the target offset (clamped). Does not move `current` until [`Self::update`].
    pub fn set_target(&mut self, target: f32) {
        self.target = target.clamp(self.min, self.max);
    }

    /// Add a delta to the target (e.g. wheel or touch step).
    pub fn add_delta(&mut self, delta: f32) {
        self.set_target(self.target + delta);
    }

    /// Advance one timestep: move `current` toward `target` via exponential decay.
    ///
    /// Uses `factor = 1 - exp(-lerp_factor * dt * 60)` so behavior is **frame-rate independent**.
    pub fn update(&mut self, dt: f32) {
        if dt <= 0.0 {
            return;
        }
        let factor = 1.0 - (-self.lerp_factor * dt * 60.0).exp();
        self.current += (self.target - self.current) * factor;
    }

    /// Snap `current` to `target` (e.g. `prefers-reduced-motion`).
    pub fn snap_to_target(&mut self) {
        self.current = self.target;
    }

    /// Align both values to an external scroll position (e.g. browser sync on attach).
    pub fn sync_both(&mut self, scroll_y: f32) {
        let y = scroll_y.clamp(self.min, self.max);
        self.current = y;
        self.target = y;
    }

    fn clamp_both(&mut self) {
        self.target = self.target.clamp(self.min, self.max);
        self.current = self.current.clamp(self.min, self.max);
    }

    /// Whether `current` is close to `target` (for optional early-out in hosts).
    pub fn is_settled(&self, epsilon: f32) -> bool {
        (self.target - self.current).abs() < epsilon
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exponential_converges_to_target() {
        let mut s = SmoothScroll1D::new(0.0, 0.0, 10_000.0, 8.0);
        s.set_target(100.0);
        for _ in 0..200 {
            s.update(1.0 / 60.0);
        }
        assert!((s.current() - 100.0).abs() < 0.5, "got {}", s.current());
    }

    #[test]
    fn large_dt_small_dt_same_end_state() {
        let mut a = SmoothScroll1D::new(0.0, 0.0, 10_000.0, 5.0);
        a.set_target(500.0);
        a.update(1.0 / 30.0);
        a.update(1.0 / 30.0);

        let mut b = SmoothScroll1D::new(0.0, 0.0, 10_000.0, 5.0);
        b.set_target(500.0);
        b.update(2.0 / 30.0);

        assert!((a.current() - b.current()).abs() < 1.0);
    }

    #[test]
    fn add_delta_clamps_target() {
        let mut s = SmoothScroll1D::new(0.0, 0.0, 100.0, 4.0);
        s.add_delta(200.0);
        assert!((s.target() - 100.0).abs() < 1e-5);
    }

    #[test]
    fn snap_to_target() {
        let mut s = SmoothScroll1D::new(0.0, 0.0, 1000.0, 4.0);
        s.set_target(500.0);
        s.update(1.0 / 60.0);
        assert!((s.current() - 500.0).abs() > 1.0);
        s.snap_to_target();
        assert!((s.current() - 500.0).abs() < 1e-5);
    }

    #[test]
    fn sync_both() {
        let mut s = SmoothScroll1D::new(0.0, 0.0, 200.0, 4.0);
        s.set_target(150.0);
        s.sync_both(42.0);
        assert!((s.current() - 42.0).abs() < 1e-5);
        assert!((s.target() - 42.0).abs() < 1e-5);
    }
}
