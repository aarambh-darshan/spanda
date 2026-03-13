//! Clock abstractions for decoupling animation from wall time.
//!
//! The [`Clock`] trait lets the animation system work with any time source —
//! real clocks for production, fixed-step clocks for deterministic tests.
//!
//! # Example — deterministic testing with [`MockClock`]
//!
//! ```rust
//! use spanda::clock::{Clock, MockClock};
//!
//! let mut clock = MockClock::new(1.0 / 60.0); // 60 fps
//! let dt = clock.delta();
//! assert!((dt - 1.0 / 60.0).abs() < 1e-6);
//! ```

// ── Clock trait ──────────────────────────────────────────────────────────────

/// Provides a delta-time value for each animation frame.
///
/// Implementors return the number of seconds elapsed since the last call
/// to [`Clock::delta`].
pub trait Clock {
    /// Returns seconds elapsed since the last call to `delta()`.
    fn delta(&mut self) -> f32;
}

// ── WallClock (std only) ─────────────────────────────────────────────────────

/// Real wall-clock time using [`std::time::Instant`].
///
/// Only available with the `std` feature (enabled by default).
#[cfg(feature = "std")]
pub struct WallClock {
    last: std::time::Instant,
}

#[cfg(feature = "std")]
impl WallClock {
    /// Create a new `WallClock` starting from now.
    pub fn new() -> Self {
        Self {
            last: std::time::Instant::now(),
        }
    }
}

#[cfg(feature = "std")]
impl Default for WallClock {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "std")]
impl Clock for WallClock {
    fn delta(&mut self) -> f32 {
        let now = std::time::Instant::now();
        let dt = now.duration_since(self.last).as_secs_f32();
        self.last = now;
        dt
    }
}

#[cfg(feature = "std")]
impl core::fmt::Debug for WallClock {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("WallClock")
            .field("last", &self.last)
            .finish()
    }
}

// ── ManualClock ──────────────────────────────────────────────────────────────

/// A clock where the caller explicitly provides delta-time via [`ManualClock::advance`].
///
/// Useful for game engines that already compute `dt` each frame.
///
/// ```rust
/// use spanda::clock::{Clock, ManualClock};
///
/// let mut clock = ManualClock::new();
/// clock.advance(0.016); // ~60 fps frame
/// let dt = clock.delta();
/// assert!((dt - 0.016).abs() < 1e-6);
/// ```
#[derive(Debug, Clone)]
pub struct ManualClock {
    pending_dt: f32,
}

impl ManualClock {
    /// Create a new `ManualClock` with zero pending time.
    pub fn new() -> Self {
        Self { pending_dt: 0.0 }
    }

    /// Accumulate time.  The next call to [`Clock::delta`] will return
    /// the total accumulated time and reset the accumulator.
    pub fn advance(&mut self, dt: f32) {
        self.pending_dt += dt;
    }
}

impl Default for ManualClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for ManualClock {
    fn delta(&mut self) -> f32 {
        let dt = self.pending_dt;
        self.pending_dt = 0.0;
        dt
    }
}

// ── MockClock ────────────────────────────────────────────────────────────────

/// A clock that returns a fixed time step on every call to [`Clock::delta`].
///
/// Perfect for unit tests — makes animation behaviour 100 % deterministic.
///
/// ```rust
/// use spanda::clock::{Clock, MockClock};
///
/// let mut clock = MockClock::new(0.1); // 10 fps
/// assert!((clock.delta() - 0.1).abs() < 1e-6);
/// assert!((clock.delta() - 0.1).abs() < 1e-6);
/// ```
#[derive(Debug, Clone)]
pub struct MockClock {
    step: f32,
}

impl MockClock {
    /// Create a mock clock returning `step_seconds` on every `delta()` call.
    pub fn new(step_seconds: f32) -> Self {
        Self {
            step: step_seconds,
        }
    }
}

impl Clock for MockClock {
    fn delta(&mut self) -> f32 {
        self.step
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_clock_returns_fixed_dt() {
        let mut clock = MockClock::new(1.0 / 60.0);
        for _ in 0..100 {
            let dt = clock.delta();
            assert!((dt - 1.0 / 60.0).abs() < 1e-7);
        }
    }

    #[test]
    fn manual_clock_accumulates() {
        let mut clock = ManualClock::new();
        clock.advance(0.1);
        clock.advance(0.2);
        let dt = clock.delta();
        assert!((dt - 0.3).abs() < 1e-6);
    }

    #[test]
    fn manual_clock_resets_after_delta() {
        let mut clock = ManualClock::new();
        clock.advance(0.5);
        let _ = clock.delta();
        let dt = clock.delta();
        assert!((dt - 0.0).abs() < 1e-6);
    }

    #[cfg(feature = "std")]
    #[test]
    fn wall_clock_returns_positive() {
        let mut clock = WallClock::new();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let dt = clock.delta();
        assert!(dt > 0.0);
    }
}
