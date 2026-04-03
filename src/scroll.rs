//! Scroll-linked animation driver.
//!
//! Instead of driving animations with wall-clock time, a [`ScrollDriver`]
//! maps a **position** (e.g. scroll offset in pixels) to animation progress.
//! Combined with a [`ScrollClock`], this gives you GSAP-style scroll-triggered
//! animations in any Rust target.
//!
//! # Example — basic scroll-linked tween
//!
//! ```rust
//! use spanda::scroll::{ScrollClock, ScrollDriver};
//! use spanda::tween::Tween;
//! use spanda::easing::Easing;
//!
//! // Map scroll range 0..1000 pixels to animation progress 0..1
//! let mut driver = ScrollDriver::new(0.0, 1000.0);
//!
//! driver.add(Tween::new(0.0_f32, 100.0).duration(1.0).build());
//!
//! // Simulate scrolling to 500px (50%)
//! driver.set_position(500.0);
//! assert_eq!(driver.active_count(), 1);
//! ```
//!
//! # Example — scroll clock for manual use
//!
//! ```rust
//! use spanda::scroll::ScrollClock;
//! use spanda::clock::Clock;
//!
//! let mut clock = ScrollClock::new(0.0, 1000.0);
//! clock.set_position(100.0);
//! clock.set_position(250.0);
//! let dt = clock.delta(); // 0.15 (150px / 1000px range)
//! ```

#[cfg(not(feature = "std"))]
#[allow(unused_imports)]
use num_traits::Float as _;

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, vec::Vec};

use crate::clock::Clock;
use crate::driver::AnimationId;
use crate::traits::Update;

// ── ScrollClock ─────────────────────────────────────────────────────────────

/// A [`Clock`] that derives delta-time from scroll position changes.
///
/// Instead of measuring wall time, `ScrollClock` converts position changes
/// (in any unit — pixels, percentage, etc.) into normalised animation progress.
///
/// The `delta()` value represents the *fraction of the scroll range* that
/// was traversed since the last call, which can be fed directly into
/// `Update::update()`.
#[derive(Debug, Clone)]
pub struct ScrollClock {
    /// Start of the scroll range (e.g. 0.0 pixels).
    start: f32,
    /// End of the scroll range (e.g. 1000.0 pixels).
    end: f32,
    /// Current scroll position.
    position: f32,
    /// Accumulated position change since last `delta()` call.
    pending_delta: f32,
}

impl ScrollClock {
    /// Create a new `ScrollClock` mapping position from `start` to `end`.
    ///
    /// The animation progresses from 0.0 to 1.0 as position moves from
    /// `start` to `end`. Scrolling backwards produces negative delta.
    pub fn new(start: f32, end: f32) -> Self {
        Self {
            start,
            end,
            position: start,
            pending_delta: 0.0,
        }
    }

    /// Set the current scroll position.
    ///
    /// The difference from the previous position is accumulated and returned
    /// by the next [`Clock::delta`] call.
    pub fn set_position(&mut self, position: f32) {
        let range = self.end - self.start;
        if range.abs() < 1e-10 {
            return;
        }
        let old_progress = (self.position - self.start) / range;
        let new_progress = (position - self.start) / range;
        self.pending_delta += new_progress - old_progress;
        self.position = position;
    }

    /// Current scroll position.
    pub fn position(&self) -> f32 {
        self.position
    }

    /// Current progress in `0.0..=1.0` (clamped).
    pub fn progress(&self) -> f32 {
        let range = self.end - self.start;
        if range.abs() < 1e-10 {
            return 0.0;
        }
        ((self.position - self.start) / range).clamp(0.0, 1.0)
    }

    /// The start of the scroll range.
    pub fn start(&self) -> f32 {
        self.start
    }

    /// The end of the scroll range.
    pub fn end(&self) -> f32 {
        self.end
    }

    /// Update the scroll range at runtime.
    pub fn set_range(&mut self, start: f32, end: f32) {
        self.start = start;
        self.end = end;
    }
}

impl Clock for ScrollClock {
    /// Returns the accumulated scroll delta as a fraction of the total range.
    ///
    /// For a scroll range of 0..1000, scrolling from 0 to 500 produces a
    /// delta of 0.5. The accumulator is reset after each call.
    fn delta(&mut self) -> f32 {
        let d = self.pending_delta;
        self.pending_delta = 0.0;
        d
    }
}

// ── ScrollDriver ────────────────────────────────────────────────────────────

/// A driver that maps scroll position to animation progress.
///
/// This is the scroll-linked equivalent of [`AnimationDriver`](crate::driver::AnimationDriver).
/// Instead of calling `tick(dt)`, you call [`ScrollDriver::set_position`] and
/// the driver converts the scroll movement into animation progress.
///
/// Animations added to a `ScrollDriver` should use a duration of **1.0** —
/// the driver normalises the scroll range to `[0.0, 1.0]`.
pub struct ScrollDriver {
    clock: ScrollClock,
    animations: Vec<(AnimationId, Box<dyn Update>)>,
    next_id: u64,
    /// Previous progress for direction/enter/leave detection.
    prev_progress: f32,
    /// Whether we're currently "inside" the scroll range.
    #[allow(dead_code)]
    inside: bool,
    /// Scroll snap points (progress values to snap to).
    snap_points: Vec<f32>,
    /// Callback fired when scrolling enters the range (progress crosses 0.0 going forward).
    #[cfg(feature = "std")]
    on_enter_cb: Option<Box<dyn FnMut()>>,
    /// Callback fired when scrolling leaves the range (progress crosses 1.0 going forward).
    #[cfg(feature = "std")]
    on_leave_cb: Option<Box<dyn FnMut()>>,
    /// Callback fired when scrolling enters from the back (progress crosses 1.0 going backward).
    #[cfg(feature = "std")]
    on_enter_back_cb: Option<Box<dyn FnMut()>>,
    /// Callback fired when scrolling leaves from the back (progress crosses 0.0 going backward).
    #[cfg(feature = "std")]
    on_leave_back_cb: Option<Box<dyn FnMut()>>,
}

impl ScrollDriver {
    /// Create a new `ScrollDriver` mapping scroll from `start` to `end`.
    pub fn new(start: f32, end: f32) -> Self {
        Self {
            clock: ScrollClock::new(start, end),
            animations: Vec::new(),
            next_id: 0,
            prev_progress: 0.0,
            inside: false,
            snap_points: Vec::new(),
            #[cfg(feature = "std")]
            on_enter_cb: None,
            #[cfg(feature = "std")]
            on_leave_cb: None,
            #[cfg(feature = "std")]
            on_enter_back_cb: None,
            #[cfg(feature = "std")]
            on_leave_back_cb: None,
        }
    }

    /// Add an animation.
    pub fn add<A: Update + 'static>(&mut self, animation: A) -> AnimationId {
        let id = AnimationId::new(self.next_id);
        self.next_id += 1;
        self.animations.push((id, Box::new(animation)));
        id
    }

    /// Set scroll snap points (progress values to snap to).
    ///
    /// When the user stops scrolling near a snap point, the scroll position
    /// will animate to that point.
    pub fn set_snap_points(&mut self, points: Vec<f32>) {
        self.snap_points = points;
    }

    /// Add a single snap point.
    pub fn add_snap_point(&mut self, progress: f32) {
        self.snap_points.push(progress.clamp(0.0, 1.0));
    }

    /// Get the nearest snap point to the current progress.
    ///
    /// Returns `None` if no snap points are configured.
    pub fn nearest_snap_point(&self) -> Option<f32> {
        if self.snap_points.is_empty() {
            return None;
        }
        let current = self.clock.progress();
        self.snap_points
            .iter()
            .min_by(|a, b| {
                let da = (current - **a).abs();
                let db = (current - **b).abs();
                da.partial_cmp(&db).unwrap_or(core::cmp::Ordering::Equal)
            })
            .copied()
    }

    /// Register a callback fired when scrolling enters the range (going forward).
    #[cfg(feature = "std")]
    pub fn on_enter<F: FnMut() + 'static>(&mut self, f: F) -> &mut Self {
        self.on_enter_cb = Some(Box::new(f));
        self
    }

    /// Register a callback fired when scrolling leaves the range (going forward).
    #[cfg(feature = "std")]
    pub fn on_leave<F: FnMut() + 'static>(&mut self, f: F) -> &mut Self {
        self.on_leave_cb = Some(Box::new(f));
        self
    }

    /// Register a callback fired when scrolling enters from the back (going backward).
    #[cfg(feature = "std")]
    pub fn on_enter_back<F: FnMut() + 'static>(&mut self, f: F) -> &mut Self {
        self.on_enter_back_cb = Some(Box::new(f));
        self
    }

    /// Register a callback fired when scrolling leaves from the back (going backward).
    #[cfg(feature = "std")]
    pub fn on_leave_back<F: FnMut() + 'static>(&mut self, f: F) -> &mut Self {
        self.on_leave_back_cb = Some(Box::new(f));
        self
    }

    /// Set the current scroll position and tick all animations.
    ///
    /// This is the primary method — call it whenever the scroll position
    /// changes (e.g. on a scroll event).
    pub fn set_position(&mut self, position: f32) {
        self.clock.set_position(position);
        let dt = self.clock.delta();

        let new_progress = self.clock.progress();
        let old_progress = self.prev_progress;
        #[allow(unused_variables)]
        let going_forward = new_progress > old_progress;

        // Check for enter/leave callbacks
        #[cfg(feature = "std")]
        {
            // Enter from start (0.0 crossed going forward)
            if !self.inside && going_forward && old_progress < 0.01 && new_progress >= 0.01 {
                self.inside = true;
                if let Some(ref mut cb) = self.on_enter_cb {
                    cb();
                }
            }
            // Leave at end (1.0 crossed going forward)
            if self.inside && going_forward && old_progress < 0.99 && new_progress >= 0.99 {
                self.inside = false;
                if let Some(ref mut cb) = self.on_leave_cb {
                    cb();
                }
            }
            // Enter from back (1.0 crossed going backward)
            if !self.inside && !going_forward && old_progress > 0.99 && new_progress <= 0.99 {
                self.inside = true;
                if let Some(ref mut cb) = self.on_enter_back_cb {
                    cb();
                }
            }
            // Leave at start (0.0 crossed going backward)
            if self.inside && !going_forward && old_progress > 0.01 && new_progress <= 0.01 {
                self.inside = false;
                if let Some(ref mut cb) = self.on_leave_back_cb {
                    cb();
                }
            }
        }

        self.prev_progress = new_progress;

        if dt.abs() > 1e-10 {
            self.animations.retain_mut(|(_, anim)| anim.update(dt));
        }
    }

    /// Current scroll progress in `0.0..=1.0`.
    pub fn progress(&self) -> f32 {
        self.clock.progress()
    }

    /// Current scroll position.
    pub fn position(&self) -> f32 {
        self.clock.position()
    }

    /// Cancel a specific animation.
    pub fn cancel(&mut self, id: AnimationId) {
        self.animations.retain(|(aid, _)| *aid != id);
    }

    /// Cancel all animations.
    pub fn cancel_all(&mut self) {
        self.animations.clear();
    }

    /// Number of active animations.
    pub fn active_count(&self) -> usize {
        self.animations.len()
    }

    /// Access the underlying [`ScrollClock`].
    pub fn clock(&self) -> &ScrollClock {
        &self.clock
    }

    /// Mutable access to the underlying [`ScrollClock`].
    pub fn clock_mut(&mut self) -> &mut ScrollClock {
        &mut self.clock
    }
}

impl core::fmt::Debug for ScrollDriver {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ScrollDriver")
            .field("clock", &self.clock)
            .field("active_count", &self.animations.len())
            .field("next_id", &self.next_id)
            .finish()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tween::Tween;

    #[test]
    fn scroll_clock_basic_delta() {
        let mut clock = ScrollClock::new(0.0, 1000.0);
        clock.set_position(500.0);
        let dt = clock.delta();
        assert!((dt - 0.5).abs() < 1e-6, "Expected 0.5, got {dt}");
    }

    #[test]
    fn scroll_clock_accumulates() {
        let mut clock = ScrollClock::new(0.0, 100.0);
        clock.set_position(25.0);
        clock.set_position(75.0);
        let dt = clock.delta();
        assert!((dt - 0.75).abs() < 1e-6, "Expected 0.75, got {dt}");
    }

    #[test]
    fn scroll_clock_resets_after_delta() {
        let mut clock = ScrollClock::new(0.0, 100.0);
        clock.set_position(50.0);
        let _ = clock.delta();
        let dt = clock.delta();
        assert!((dt - 0.0).abs() < 1e-6);
    }

    #[test]
    fn scroll_clock_backward_produces_negative_delta() {
        let mut clock = ScrollClock::new(0.0, 100.0);
        clock.set_position(80.0);
        let _ = clock.delta(); // consume
        clock.set_position(30.0);
        let dt = clock.delta();
        assert!((dt - (-0.5)).abs() < 1e-6, "Expected -0.5, got {dt}");
    }

    #[test]
    fn scroll_clock_progress() {
        let mut clock = ScrollClock::new(0.0, 200.0);
        clock.set_position(100.0);
        assert!((clock.progress() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn scroll_clock_zero_range_is_safe() {
        let mut clock = ScrollClock::new(100.0, 100.0);
        clock.set_position(100.0);
        let dt = clock.delta();
        assert!((dt - 0.0).abs() < 1e-6);
    }

    #[test]
    fn scroll_driver_ticks_animations() {
        let mut driver = ScrollDriver::new(0.0, 1.0);
        driver.add(Tween::new(0.0_f32, 100.0).duration(1.0).build());
        assert_eq!(driver.active_count(), 1);

        // Scroll to the end — tween should complete
        driver.set_position(1.0);
        assert_eq!(driver.active_count(), 0);
    }

    #[test]
    fn scroll_driver_partial_scroll() {
        let mut driver = ScrollDriver::new(0.0, 100.0);
        driver.add(Tween::new(0.0_f32, 100.0).duration(1.0).build());

        driver.set_position(50.0); // 50% scroll
        assert_eq!(driver.active_count(), 1); // still running
    }

    #[test]
    fn scroll_driver_cancel() {
        let mut driver = ScrollDriver::new(0.0, 100.0);
        let id = driver.add(Tween::new(0.0_f32, 100.0).duration(1.0).build());
        assert_eq!(driver.active_count(), 1);

        driver.cancel(id);
        assert_eq!(driver.active_count(), 0);
    }
}
