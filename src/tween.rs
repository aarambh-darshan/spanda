//! Single-value animation — the most common building block.
//!
//! A [`Tween<T>`] smoothly interpolates a value from `start` to `end` over a
//! `duration` in seconds, applying an [`Easing`] curve.
//!
//! # Quick start
//!
//! ```rust
//! use spanda::tween::Tween;
//! use spanda::easing::Easing;
//! use spanda::traits::Update;
//!
//! let mut t = Tween::new(0.0_f32, 100.0)
//!     .duration(1.0)
//!     .easing(Easing::EaseOutCubic)
//!     .build();
//!
//! // Simulate 10 frames of 0.1s each:
//! for _ in 0..10 {
//!     t.update(0.1);
//! }
//!
//! assert!(t.is_complete());
//! assert!((t.value() - 100.0).abs() < 1e-6);
//! ```

use crate::easing::Easing;
use crate::keyframe::Loop;
use crate::traits::{Animatable, Update};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// ── TweenState ───────────────────────────────────────────────────────────────

/// Current phase of a [`Tween`]'s lifecycle.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TweenState {
    /// Inside the delay period — animation has not started yet.
    Waiting,
    /// Actively interpolating between `start` and `end`.
    Running,
    /// Reached the end — `value()` returns `end`.
    Completed,
    /// Manually paused via [`Tween::pause`].
    Paused,
}

// ── Tween ────────────────────────────────────────────────────────────────────

/// A single-value animation from `start` to `end` over `duration` seconds.
///
/// Use [`Tween::new`] to start the builder chain.
#[cfg_attr(feature = "bevy", derive(bevy_ecs::component::Component))]
pub struct Tween<T: Animatable> {
    /// Starting value of the animation.
    pub start: T,
    /// Ending value of the animation.
    pub end: T,
    /// Total animation duration in seconds.
    pub duration: f32,
    /// Easing curve applied to the raw progress.
    pub easing: Easing,
    /// Delay in seconds before animation begins.
    pub delay: f32,
    /// Time scale multiplier applied to dt (default 1.0).
    time_scale: f32,
    /// Loop mode for this tween (default: Loop::Once).
    looping: Loop,
    /// Number of completed loop iterations.
    loop_count: u32,
    /// Direction flag for PingPong (true = forward).
    forward: bool,
    /// Whether on_start has been fired for the current iteration.
    started: bool,
    /// Private: accumulated elapsed time (after delay).
    elapsed: f32,
    /// Private: current state.
    state: TweenState,
    /// Callback fired once when the tween starts running (per iteration).
    #[cfg(all(feature = "std", not(feature = "bevy")))]
    on_start_cb: Option<Box<dyn FnMut()>>,
    /// Callback fired every frame while Running. Receives the current value.
    #[cfg(all(feature = "std", not(feature = "bevy")))]
    on_update_cb: Option<Box<dyn FnMut(T)>>,
    /// Callback fired once when the tween completes (after all loops).
    #[cfg(all(feature = "std", not(feature = "bevy")))]
    on_complete_cb: Option<Box<dyn FnMut()>>,
    /// Value modifier applied after interpolation in `value()`.
    #[cfg(all(feature = "std", not(feature = "bevy")))]
    modifier: Option<Box<dyn Fn(T) -> T>>,
}

impl<T: Animatable> core::fmt::Debug for Tween<T>
where
    T: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Tween")
            .field("start", &self.start)
            .field("end", &self.end)
            .field("duration", &self.duration)
            .field("easing", &self.easing)
            .field("delay", &self.delay)
            .field("time_scale", &self.time_scale)
            .field("looping", &self.looping)
            .field("loop_count", &self.loop_count)
            .field("forward", &self.forward)
            .field("elapsed", &self.elapsed)
            .field("state", &self.state)
            .finish()
    }
}

impl<T: Animatable> Tween<T> {
    /// Begin building a tween from `start` to `end`.
    ///
    /// Returns a [`TweenBuilder`] — call `.duration()`, `.easing()`,
    /// `.delay()`, and `.build()` to finish.
    pub fn new(start: T, end: T) -> TweenBuilder<T> {
        TweenBuilder {
            start,
            end,
            duration: 1.0,
            easing: Easing::Linear,
            delay: 0.0,
            time_scale: 1.0,
            looping: Loop::Once,
        }
    }

    /// GSAP-style `from_to` — explicit starting and ending values.
    ///
    /// Semantically identical to [`Tween::new`], provided for GSAP-familiar
    /// naming.
    pub fn from_to(from: T, to: T) -> TweenBuilder<T> {
        Self::new(from, to)
    }

    /// GSAP-style `from` — animate *from* a starting value *to* the given end.
    ///
    /// Semantically identical to [`Tween::new`], but communicates intent: the
    /// first argument is the "source" state, the second is the "destination".
    pub fn from(start: T, end: T) -> TweenBuilder<T> {
        Self::new(start, end)
    }

    /// Current interpolated value.
    ///
    /// Applies the easing curve to the raw progress and lerps between
    /// `start` and `end`.  If a modifier is set, it is applied after
    /// interpolation.
    pub fn value(&self) -> T {
        let val = if self.duration <= 0.0 {
            self.end.clone()
        } else {
            let raw_t = (self.elapsed / self.duration).clamp(0.0, 1.0);
            let curved_t = self.easing.apply(raw_t);
            self.start.lerp(&self.end, curved_t)
        };
        #[cfg(all(feature = "std", not(feature = "bevy")))]
        {
            if let Some(ref m) = self.modifier {
                return m(val);
            }
        }
        val
    }

    /// Raw progress in `0.0..=1.0` (before easing is applied).
    pub fn progress(&self) -> f32 {
        if self.duration <= 0.0 {
            return 1.0;
        }
        (self.elapsed / self.duration).clamp(0.0, 1.0)
    }

    /// `true` once the animation has finished.
    pub fn is_complete(&self) -> bool {
        self.state == TweenState::Completed
    }

    /// Reset the tween to the beginning.
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
        self.loop_count = 0;
        self.forward = true;
        self.started = false;
        self.state = if self.delay > 0.0 {
            TweenState::Waiting
        } else {
            TweenState::Running
        };
    }

    /// Jump to a specific progress value `t ∈ [0.0, 1.0]`.
    pub fn seek(&mut self, t: f32) {
        let t = t.clamp(0.0, 1.0);
        self.elapsed = t * self.duration;
        if t >= 1.0 {
            self.state = TweenState::Completed;
        } else {
            self.state = TweenState::Running;
        }
    }

    /// Swap `start` and `end`, then reset to the beginning.
    pub fn reverse(&mut self) {
        core::mem::swap(&mut self.start, &mut self.end);
        self.reset();
    }

    /// Pause the tween (freezes `elapsed`).
    pub fn pause(&mut self) {
        if self.state == TweenState::Running || self.state == TweenState::Waiting {
            self.state = TweenState::Paused;
        }
    }

    /// Resume a paused tween.
    pub fn resume(&mut self) {
        if self.state == TweenState::Paused {
            self.state = if self.elapsed > 0.0 || self.delay <= 0.0 {
                TweenState::Running
            } else {
                TweenState::Waiting
            };
        }
    }

    /// Returns the current [`TweenState`].
    pub fn state(&self) -> &TweenState {
        &self.state
    }

    /// Set the time scale at runtime.
    ///
    /// Values > 1.0 speed up, < 1.0 slow down, 0.0 effectively pauses.
    pub fn set_time_scale(&mut self, scale: f32) {
        self.time_scale = scale;
    }

    /// Get the current time scale.
    pub fn time_scale(&self) -> f32 {
        self.time_scale
    }

    /// Get the current loop mode.
    pub fn loop_mode(&self) -> &Loop {
        &self.looping
    }

    /// Register a callback that fires once when the tween starts running.
    ///
    /// Fires once per loop iteration for looping tweens.
    #[cfg(all(feature = "std", not(feature = "bevy")))]
    pub fn on_start<F: FnMut() + 'static>(&mut self, f: F) -> &mut Self {
        self.on_start_cb = Some(Box::new(f));
        self
    }

    /// Register a callback that fires every frame with the current value.
    ///
    /// This is the primary bridge for reactive frameworks like Leptos:
    /// ```rust,ignore
    /// tween.on_update(move |val: f32| set_signal.set(val));
    /// ```
    #[cfg(all(feature = "std", not(feature = "bevy")))]
    pub fn on_update<F: FnMut(T) + 'static>(&mut self, f: F) -> &mut Self {
        self.on_update_cb = Some(Box::new(f));
        self
    }

    /// Register a callback that fires once when the tween completes.
    #[cfg(all(feature = "std", not(feature = "bevy")))]
    pub fn on_complete<F: FnMut() + 'static>(&mut self, f: F) -> &mut Self {
        self.on_complete_cb = Some(Box::new(f));
        self
    }

    /// Set a value modifier that transforms the interpolated value before
    /// it is returned by [`Tween::value`].
    ///
    /// ```rust,ignore
    /// tween.set_modifier(|v| (v / 10.0).round() * 10.0); // snap to nearest 10
    /// ```
    #[cfg(all(feature = "std", not(feature = "bevy")))]
    pub fn set_modifier<F: Fn(T) -> T + 'static>(&mut self, f: F) -> &mut Self {
        self.modifier = Some(Box::new(f));
        self
    }
}

impl<T: Animatable> Update for Tween<T> {
    fn update(&mut self, dt: f32) -> bool {
        let dt = (dt * self.time_scale).max(0.0);

        match self.state {
            TweenState::Completed | TweenState::Paused => return !self.is_complete(),
            TweenState::Waiting => {
                self.delay -= dt;
                if self.delay <= 0.0 {
                    let leftover = -self.delay;
                    self.delay = 0.0;
                    self.state = TweenState::Running;
                    self.started = false;
                    self.elapsed += leftover;
                } else {
                    return true; // still waiting
                }
            }
            TweenState::Running => {
                self.elapsed += dt;
            }
        }

        // Fire on_start callback once per loop iteration
        if !self.started {
            self.started = true;
            #[cfg(all(feature = "std", not(feature = "bevy")))]
            {
                if let Some(ref mut cb) = self.on_start_cb {
                    cb();
                }
            }
        }

        // Check loop completion
        if self.elapsed >= self.duration {
            match &self.looping {
                Loop::Once => {
                    self.elapsed = self.duration;
                    self.state = TweenState::Completed;
                }
                Loop::Times(n) => {
                    self.loop_count += 1;
                    if self.loop_count >= *n {
                        self.elapsed = self.duration;
                        self.state = TweenState::Completed;
                    } else {
                        let leftover = self.elapsed - self.duration;
                        self.elapsed = leftover;
                        self.started = false;
                    }
                }
                Loop::Forever => {
                    let leftover = self.elapsed - self.duration;
                    self.elapsed = leftover;
                    self.loop_count += 1;
                    self.started = false;
                }
                Loop::PingPong => {
                    let leftover = self.elapsed - self.duration;
                    self.elapsed = leftover;
                    self.loop_count += 1;
                    self.forward = !self.forward;
                    core::mem::swap(&mut self.start, &mut self.end);
                    self.started = false;
                }
            }
        }

        // Fire on_update callback with current value
        #[cfg(all(feature = "std", not(feature = "bevy")))]
        {
            if self.state == TweenState::Running && self.on_update_cb.is_some() {
                let val = self.value();
                if let Some(ref mut cb) = self.on_update_cb {
                    cb(val);
                }
            }
        }

        // Fire on_complete callback
        if self.state == TweenState::Completed {
            #[cfg(all(feature = "std", not(feature = "bevy")))]
            {
                if let Some(ref mut cb) = self.on_complete_cb {
                    cb();
                }
            }
        }

        !self.is_complete()
    }
}

// ── TweenBuilder ─────────────────────────────────────────────────────────────

/// Builder for [`Tween`].  Created via [`Tween::new`].
pub struct TweenBuilder<T: Animatable> {
    start: T,
    end: T,
    duration: f32,
    easing: Easing,
    delay: f32,
    time_scale: f32,
    looping: Loop,
}

impl<T: Animatable> TweenBuilder<T> {
    /// Set the animation duration in seconds (default: 1.0).
    pub fn duration(mut self, d: f32) -> Self {
        self.duration = d.max(0.0);
        self
    }

    /// Set the easing curve (default: [`Easing::Linear`]).
    pub fn easing(mut self, e: Easing) -> Self {
        self.easing = e;
        self
    }

    /// Set a delay in seconds before the animation starts (default: 0.0).
    pub fn delay(mut self, d: f32) -> Self {
        self.delay = d.max(0.0);
        self
    }

    /// Set the time scale multiplier (default: 1.0).
    ///
    /// Values > 1.0 speed up, < 1.0 slow down, 0.0 effectively pauses.
    pub fn time_scale(mut self, scale: f32) -> Self {
        self.time_scale = scale;
        self
    }

    /// Set the loop mode (default: [`Loop::Once`]).
    pub fn looping(mut self, mode: Loop) -> Self {
        self.looping = mode;
        self
    }

    /// Consume the builder and produce a [`Tween`].
    pub fn build(self) -> Tween<T> {
        let state = if self.delay > 0.0 {
            TweenState::Waiting
        } else {
            TweenState::Running
        };
        Tween {
            start: self.start,
            end: self.end,
            duration: self.duration,
            easing: self.easing,
            delay: self.delay,
            time_scale: self.time_scale,
            looping: self.looping,
            loop_count: 0,
            forward: true,
            started: false,
            elapsed: 0.0,
            state,
            #[cfg(all(feature = "std", not(feature = "bevy")))]
            on_start_cb: None,
            #[cfg(all(feature = "std", not(feature = "bevy")))]
            on_update_cb: None,
            #[cfg(all(feature = "std", not(feature = "bevy")))]
            on_complete_cb: None,
            #[cfg(all(feature = "std", not(feature = "bevy")))]
            modifier: None,
        }
    }
}

impl<T: Animatable + core::fmt::Debug> core::fmt::Debug for TweenBuilder<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TweenBuilder")
            .field("start", &self.start)
            .field("end", &self.end)
            .field("duration", &self.duration)
            .field("delay", &self.delay)
            .field("time_scale", &self.time_scale)
            .field("looping", &self.looping)
            .finish()
    }
}

// ── Snapping utilities ──────────────────────────────────────────────────────

/// Create a modifier that snaps `f32` values to the nearest multiple of `grid`.
///
/// ```rust
/// use spanda::tween::snap_to;
///
/// let snapper = snap_to(10.0);
/// assert!((snapper(23.7) - 20.0).abs() < 1e-4);
/// assert!((snapper(25.0) - 30.0).abs() < 1e-4);
/// ```
pub fn snap_to(grid: f32) -> impl Fn(f32) -> f32 {
    move |v: f32| {
        if grid <= 0.0 {
            return v;
        }
        (v / grid).round() * grid
    }
}

/// Create a modifier that rounds `f32` values to a given number of decimal
/// places.
///
/// ```rust
/// use spanda::tween::round_to;
///
/// let rounder = round_to(1);
/// assert!((rounder(3.14159) - 3.1).abs() < 1e-4);
/// ```
pub fn round_to(decimals: u32) -> impl Fn(f32) -> f32 {
    let factor = 10.0_f32.powi(decimals as i32);
    move |v: f32| (v * factor).round() / factor
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tween_starts_at_start_value() {
        let t = Tween::new(0.0_f32, 100.0).duration(1.0).build();
        assert!((t.value() - 0.0).abs() < 1e-6);
    }

    #[test]
    fn tween_ends_at_end_value() {
        let mut t = Tween::new(0.0_f32, 100.0).duration(1.0).build();
        t.update(1.0);
        assert!((t.value() - 100.0).abs() < 1e-6);
    }

    #[test]
    fn tween_is_complete_after_full_duration() {
        let mut t = Tween::new(0.0_f32, 100.0).duration(0.5).build();
        assert!(!t.is_complete());
        t.update(0.5);
        assert!(t.is_complete());
    }

    #[test]
    fn tween_delay_is_respected() {
        let mut t = Tween::new(0.0_f32, 100.0)
            .duration(1.0)
            .delay(0.5)
            .build();
        assert_eq!(t.state, TweenState::Waiting);

        // Advance less than the delay
        t.update(0.3);
        assert_eq!(t.state, TweenState::Waiting);
        assert!((t.value() - 0.0).abs() < 1e-6);

        // Advance past the delay — leftover goes into elapsed
        t.update(0.3); // total 0.6, delay was 0.5 → 0.1 into animation
        assert_eq!(t.state, TweenState::Running);
        assert!(t.value() > 0.0);
    }

    #[test]
    fn tween_reverse_swaps_values() {
        let mut t = Tween::new(0.0_f32, 100.0).duration(1.0).build();
        t.update(1.0);
        assert!((t.value() - 100.0).abs() < 1e-6);

        t.reverse();
        assert!(!t.is_complete());
        assert!((t.value() - 100.0).abs() < 1e-6); // start is now 100
        t.update(1.0);
        assert!((t.value() - 0.0).abs() < 1e-6); // end is now 0
    }

    #[test]
    fn tween_seek_jumps_to_correct_value() {
        let mut t = Tween::new(0.0_f32, 100.0).duration(1.0).build();
        t.seek(0.5);
        assert!((t.value() - 50.0).abs() < 1e-6);
    }

    #[test]
    fn tween_does_not_overshoot_on_large_dt() {
        let mut t = Tween::new(0.0_f32, 100.0).duration(1.0).build();
        t.update(999.0);
        assert!(t.is_complete());
        assert!((t.value() - 100.0).abs() < 1e-6);
    }

    #[test]
    fn tween_zero_duration_immediately_complete() {
        let mut t = Tween::new(0.0_f32, 42.0).duration(0.0).build();
        t.update(0.0);
        assert!(t.is_complete());
        assert!((t.value() - 42.0).abs() < 1e-6);
    }

    #[test]
    fn tween_with_easing() {
        let mut t = Tween::new(0.0_f32, 100.0)
            .duration(1.0)
            .easing(Easing::EaseInQuad)
            .build();
        t.update(0.5);
        // EaseInQuad at t=0.5 → 0.25, so value ≈ 25.0
        assert!((t.value() - 25.0).abs() < 1e-4);
    }

    #[test]
    fn tween_pause_and_resume() {
        let mut t = Tween::new(0.0_f32, 100.0).duration(1.0).build();
        t.update(0.3);
        let val_before_pause = t.value();
        t.pause();
        t.update(0.5); // should not advance
        assert!((t.value() - val_before_pause).abs() < 1e-6);
        t.resume();
        t.update(0.2);
        assert!(t.value() > val_before_pause);
    }

    #[test]
    fn tween_negative_dt_treated_as_zero() {
        let mut t = Tween::new(0.0_f32, 100.0).duration(1.0).build();
        t.update(0.5);
        let v = t.value();
        t.update(-0.3);
        assert!((t.value() - v).abs() < 1e-6);
    }

    #[test]
    fn tween_vec2() {
        let mut t = Tween::new([0.0_f32, 0.0], [100.0, 200.0])
            .duration(1.0)
            .build();
        t.update(0.5);
        let v = t.value();
        assert!((v[0] - 50.0).abs() < 1e-4);
        assert!((v[1] - 100.0).abs() < 1e-4);
    }

    // ── Time scale tests ────────────────────────────────────────────────────

    #[test]
    fn tween_time_scale_double_speed() {
        let mut t = Tween::new(0.0_f32, 100.0)
            .duration(1.0)
            .time_scale(2.0)
            .build();
        t.update(0.5); // effective dt = 1.0
        assert!(t.is_complete());
        assert!((t.value() - 100.0).abs() < 1e-6);
    }

    #[test]
    fn tween_time_scale_half_speed() {
        let mut t = Tween::new(0.0_f32, 100.0)
            .duration(1.0)
            .time_scale(0.5)
            .build();
        t.update(1.0); // effective dt = 0.5
        assert!(!t.is_complete());
        assert!((t.value() - 50.0).abs() < 1e-4);
    }

    #[test]
    fn tween_time_scale_zero_pauses() {
        let mut t = Tween::new(0.0_f32, 100.0)
            .duration(1.0)
            .time_scale(0.0)
            .build();
        t.update(10.0);
        assert!(!t.is_complete());
        assert!((t.value() - 0.0).abs() < 1e-6);
    }

    #[test]
    fn tween_set_time_scale_runtime() {
        let mut t = Tween::new(0.0_f32, 100.0).duration(1.0).build();
        t.set_time_scale(0.5);
        assert!((t.time_scale() - 0.5).abs() < 1e-6);
    }

    // ── Loop tests ──────────────────────────────────────────────────────────

    #[test]
    fn tween_loop_forever_never_completes() {
        let mut t = Tween::new(0.0_f32, 100.0)
            .duration(1.0)
            .looping(Loop::Forever)
            .build();
        for _ in 0..100 {
            assert!(t.update(0.5));
        }
        assert!(!t.is_complete());
    }

    #[test]
    fn tween_loop_times_completes_after_n() {
        let mut t = Tween::new(0.0_f32, 100.0)
            .duration(1.0)
            .looping(Loop::Times(3))
            .build();
        assert!(t.update(1.0));  // loop 1
        assert!(t.update(1.0));  // loop 2
        assert!(!t.update(1.0)); // loop 3, done
        assert!(t.is_complete());
    }

    #[test]
    fn tween_ping_pong_reverses_direction() {
        let mut t = Tween::new(0.0_f32, 100.0)
            .duration(1.0)
            .looping(Loop::PingPong)
            .build();
        t.update(1.0); // forward complete, swaps to reverse
        t.update(0.5); // halfway back
        // After ping-pong: start is now 100, end is now 0
        // At elapsed 0.5 of 1.0 duration, linear value should be ~50.0
        assert!((t.value() - 50.0).abs() < 1e-4);
    }

    #[test]
    fn tween_loop_once_is_default() {
        let t = Tween::new(0.0_f32, 100.0).duration(1.0).build();
        assert_eq!(*t.loop_mode(), Loop::Once);
    }

    // ── from / from_to tests ────────────────────────────────────────────────

    #[test]
    fn tween_from_to_is_alias_for_new() {
        let a = Tween::new(0.0_f32, 100.0).duration(1.0).build();
        let b = Tween::from_to(0.0_f32, 100.0).duration(1.0).build();
        assert!((a.value() - b.value()).abs() < 1e-6);
    }

    #[test]
    fn tween_from_is_alias_for_new() {
        let a = Tween::new(0.0_f32, 100.0).duration(1.0).build();
        let b = Tween::from(0.0_f32, 100.0).duration(1.0).build();
        assert!((a.value() - b.value()).abs() < 1e-6);
    }

    // ── Callback tests ──────────────────────────────────────────────────────

    #[cfg(all(feature = "std", not(feature = "bevy")))]
    #[test]
    fn on_start_fires_once() {
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;

        let count = Arc::new(AtomicU32::new(0));
        let count_clone = count.clone();

        let mut t = Tween::new(0.0_f32, 100.0).duration(1.0).build();
        t.on_start(move || { count_clone.fetch_add(1, Ordering::SeqCst); });

        t.update(0.1);
        t.update(0.1);
        t.update(0.8);

        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[cfg(all(feature = "std", not(feature = "bevy")))]
    #[test]
    fn on_update_receives_value() {
        use std::cell::RefCell;
        use std::rc::Rc;

        let values = Rc::new(RefCell::new(Vec::new()));
        let values_clone = values.clone();

        let mut t = Tween::new(0.0_f32, 100.0).duration(1.0).build();
        t.on_update(move |val| { values_clone.borrow_mut().push(val); });

        t.update(0.5);

        let recorded = values.borrow();
        assert!(!recorded.is_empty());
        assert!(recorded[0] > 0.0);
    }

    #[cfg(all(feature = "std", not(feature = "bevy")))]
    #[test]
    fn on_complete_fires_once() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;

        let fired = Arc::new(AtomicBool::new(false));
        let fired_clone = fired.clone();

        let mut t = Tween::new(0.0_f32, 100.0).duration(0.5).build();
        t.on_complete(move || { fired_clone.store(true, Ordering::SeqCst); });

        t.update(0.5);
        assert!(fired.load(Ordering::SeqCst));
    }

    // ── Modifier / snapping tests ───────────────────────────────────────────

    #[test]
    fn snap_to_rounds_correctly() {
        let snap = snap_to(10.0);
        assert!((snap(23.7) - 20.0).abs() < 1e-4);
        assert!((snap(25.0) - 30.0).abs() < 1e-4);
        assert!((snap(0.0) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn snap_to_zero_grid_is_noop() {
        let snap = snap_to(0.0);
        assert!((snap(42.7) - 42.7).abs() < 1e-6);
    }

    #[test]
    fn round_to_decimals() {
        let round = round_to(1);
        assert!((round(3.14159) - 3.1).abs() < 1e-4);
    }

    #[cfg(all(feature = "std", not(feature = "bevy")))]
    #[test]
    fn modifier_snaps_tween_value() {
        let mut t = Tween::new(0.0_f32, 100.0).duration(1.0).build();
        t.set_modifier(snap_to(25.0));
        t.update(0.3); // raw value ~30.0, snaps to 25.0 or 50.0
        let v = t.value();
        assert!(
            (v % 25.0).abs() < 1e-4,
            "Value {v} not snapped to multiple of 25"
        );
    }
}
