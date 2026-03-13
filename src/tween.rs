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
    /// Private: accumulated elapsed time (after delay).
    elapsed: f32,
    /// Private: current state.
    state: TweenState,
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
        }
    }

    /// Current interpolated value.
    ///
    /// Applies the easing curve to the raw progress and lerps between
    /// `start` and `end`.
    pub fn value(&self) -> T {
        if self.duration <= 0.0 {
            return self.end.clone();
        }
        let raw_t = (self.elapsed / self.duration).clamp(0.0, 1.0);
        let curved_t = self.easing.apply(raw_t);
        self.start.lerp(&self.end, curved_t)
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
}

impl<T: Animatable> Update for Tween<T> {
    fn update(&mut self, dt: f32) -> bool {
        let dt = dt.max(0.0); // no backward time

        match self.state {
            TweenState::Completed | TweenState::Paused => return !self.is_complete(),
            TweenState::Waiting => {
                self.delay -= dt;
                if self.delay <= 0.0 {
                    // Carry over any leftover time into elapsed
                    let leftover = -self.delay;
                    self.delay = 0.0;
                    self.state = TweenState::Running;
                    self.elapsed += leftover;
                } else {
                    return true; // still waiting
                }
            }
            TweenState::Running => {
                self.elapsed += dt;
            }
        }

        // Clamp elapsed to duration
        if self.elapsed >= self.duration {
            self.elapsed = self.duration;
            self.state = TweenState::Completed;
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
            elapsed: 0.0,
            state,
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
            .finish()
    }
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
}
