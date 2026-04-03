//! Multi-stop keyframe animation tracks.
//!
//! A [`KeyframeTrack<T>`] holds a sorted list of `(time, value)` pairs.  At any
//! time `t`, it interpolates between the two surrounding keyframes using the
//! segment's [`Easing`] curve.
//!
//! # Quick start
//!
//! ```rust
//! use spanda::keyframe::{KeyframeTrack, Loop};
//! use spanda::traits::Update;
//!
//! let mut track = KeyframeTrack::new()
//!     .push(0.0, 0.0_f32)
//!     .push(0.5, 1.0)
//!     .push(1.0, 0.0)
//!     .looping(Loop::Forever);
//!
//! track.update(0.25);
//! let value = track.value();
//! assert!(value > Some(0.0) && value < Some(1.0));
//! ```

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::easing::Easing;
use crate::traits::{Animatable, Update};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// ── Loop ─────────────────────────────────────────────────────────────────────

/// How a [`KeyframeTrack`] repeats after reaching the end.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Loop {
    /// Play once and stop at the last keyframe.
    Once,
    /// Play a fixed number of times.
    Times(u32),
    /// Loop forever.
    Forever,
    /// Play forward then backward, repeating.
    PingPong,
}

// ── Keyframe ─────────────────────────────────────────────────────────────────

/// A single keyframe — a value at a specific time with an easing to the next.
#[derive(Clone)]
pub struct Keyframe<T: Animatable> {
    /// Time in seconds from track start.
    pub time: f32,
    /// Value at this keyframe.
    pub value: T,
    /// Easing used from THIS keyframe to the NEXT one.
    pub easing: Easing,
}

impl<T: Animatable + core::fmt::Debug> core::fmt::Debug for Keyframe<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Keyframe")
            .field("time", &self.time)
            .field("value", &self.value)
            .field("easing", &self.easing)
            .finish()
    }
}

// ── KeyframeTrack ────────────────────────────────────────────────────────────

/// A sorted sequence of keyframes that can be evaluated at any time.
pub struct KeyframeTrack<T: Animatable> {
    frames: Vec<Keyframe<T>>,
    elapsed: f32,
    looping: Loop,
    completed: bool,
    loop_count: u32,
}

impl<T: Animatable + core::fmt::Debug> core::fmt::Debug for KeyframeTrack<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("KeyframeTrack")
            .field("frames", &self.frames)
            .field("elapsed", &self.elapsed)
            .field("looping", &self.looping)
            .field("completed", &self.completed)
            .finish()
    }
}

impl<T: Animatable> Default for KeyframeTrack<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Animatable> KeyframeTrack<T> {
    /// Create an empty track.
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
            elapsed: 0.0,
            looping: Loop::Once,
            completed: false,
            loop_count: 0,
        }
    }

    /// Add a keyframe at `time` with the given `value`.
    ///
    /// Uses [`Easing::Linear`] for the segment from this frame to the next.
    /// Frames are kept sorted by time internally.
    pub fn push(mut self, time: f32, value: T) -> Self {
        self.frames.push(Keyframe {
            time,
            value,
            easing: Easing::Linear,
        });
        self.frames
            .sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
        self
    }

    /// Add a keyframe with a specific easing to the next frame.
    pub fn push_with_easing(mut self, time: f32, value: T, easing: Easing) -> Self {
        self.frames.push(Keyframe {
            time,
            value,
            easing,
        });
        self.frames
            .sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
        self
    }

    /// Set the loop mode.
    pub fn looping(mut self, mode: Loop) -> Self {
        self.looping = mode;
        self
    }

    /// Total duration — time of the last keyframe.
    pub fn duration(&self) -> f32 {
        self.frames.last().map_or(0.0, |f| f.time)
    }

    /// Evaluate the track at an arbitrary time `t` (pure, ignores `elapsed`).
    ///
    /// Returns `None` if the track has no keyframes.
    pub fn value_at(&self, t: f32) -> Option<T> {
        if self.frames.is_empty() {
            return None;
        }

        if self.frames.len() == 1 {
            return Some(self.frames[0].value.clone());
        }

        // Clamp to valid range
        let t = t.clamp(0.0, self.duration());

        // Find the segment: last frame where frame.time <= t
        let idx = self.frames.iter().rposition(|f| f.time <= t).unwrap_or(0);

        // If at or past the last frame, return last value
        if idx >= self.frames.len() - 1 {
            return Some(self.frames.last().unwrap().value.clone());
        }

        let a = &self.frames[idx];
        let b = &self.frames[idx + 1];
        let segment_duration = b.time - a.time;

        if segment_duration <= 0.0 {
            return Some(b.value.clone());
        }

        let local_t = ((t - a.time) / segment_duration).clamp(0.0, 1.0);
        let curved_t = a.easing.apply(local_t);
        Some(a.value.lerp(&b.value, curved_t))
    }

    /// Current value based on internal `elapsed` time.
    ///
    /// Returns `None` if the track has no keyframes.
    pub fn value(&self) -> Option<T> {
        let t = self.effective_time();
        self.value_at(t)
    }

    /// Whether the track has finished playing.
    pub fn is_complete(&self) -> bool {
        self.completed
    }

    /// Reset to the beginning.
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
        self.completed = false;
        self.loop_count = 0;
    }

    /// Compute the effective time considering loop mode.
    fn effective_time(&self) -> f32 {
        let dur = self.duration();
        if dur <= 0.0 {
            return 0.0;
        }

        match &self.looping {
            Loop::Once => self.elapsed.clamp(0.0, dur),
            Loop::Times(_) | Loop::Forever => self.elapsed % dur,
            Loop::PingPong => {
                let cycle = 2.0 * dur;
                let cycle_t = self.elapsed % cycle;
                if cycle_t <= dur {
                    cycle_t
                } else {
                    2.0 * dur - cycle_t
                }
            }
        }
    }
}

impl<T: Animatable> Update for KeyframeTrack<T> {
    fn update(&mut self, dt: f32) -> bool {
        if self.completed {
            return false;
        }

        let dt = dt.max(0.0);
        self.elapsed += dt;

        let dur = self.duration();
        if dur <= 0.0 {
            self.completed = true;
            return false;
        }

        match &self.looping {
            Loop::Once => {
                if self.elapsed >= dur {
                    self.elapsed = dur;
                    self.completed = true;
                }
            }
            Loop::Times(n) => {
                let loops_done = (self.elapsed / dur).floor() as u32;
                if loops_done >= *n {
                    self.elapsed = dur * (*n as f32);
                    self.completed = true;
                }
            }
            Loop::Forever | Loop::PingPong => {
                // Never completes
            }
        }

        !self.completed
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_frame_returns_its_value() {
        let track = KeyframeTrack::new().push(0.0, 42.0_f32);
        assert!((track.value_at(0.0).unwrap() - 42.0).abs() < 1e-6);
        assert!((track.value_at(999.0).unwrap() - 42.0).abs() < 1e-6);
    }

    #[test]
    fn empty_track_returns_none() {
        let track = KeyframeTrack::<f32>::new();
        assert!(track.value_at(0.0).is_none());
        assert!(track.value().is_none());
    }

    #[test]
    fn two_frames_interpolate() {
        let track = KeyframeTrack::new().push(0.0, 0.0_f32).push(1.0, 100.0);
        assert!((track.value_at(0.5).unwrap() - 50.0).abs() < 1e-4);
    }

    #[test]
    fn three_frames_two_segments() {
        let track = KeyframeTrack::new()
            .push(0.0, 0.0_f32)
            .push(1.0, 100.0)
            .push(2.0, 0.0);
        assert!((track.value_at(0.5).unwrap() - 50.0).abs() < 1e-4);
        assert!((track.value_at(1.5).unwrap() - 50.0).abs() < 1e-4);
    }

    #[test]
    fn loop_once_completes() {
        let mut track = KeyframeTrack::new()
            .push(0.0, 0.0_f32)
            .push(1.0, 100.0)
            .looping(Loop::Once);

        assert!(track.update(0.5));
        assert!(!track.is_complete());
        assert!(!track.update(0.5));
        assert!(track.is_complete());
    }

    #[test]
    fn loop_forever_never_completes() {
        let mut track = KeyframeTrack::new()
            .push(0.0, 0.0_f32)
            .push(1.0, 100.0)
            .looping(Loop::Forever);

        for _ in 0..100 {
            assert!(track.update(0.5));
        }
        assert!(!track.is_complete());
    }

    #[test]
    fn ping_pong_reverses() {
        let track = KeyframeTrack::new()
            .push(0.0, 0.0_f32)
            .push(1.0, 100.0)
            .looping(Loop::PingPong);

        // At t=1.5 in ping-pong: cycle = 2.0, cycle_t = 1.5, backward → t = 0.5
        assert!((track.value_at(0.5).unwrap() - 50.0).abs() < 1e-4);
    }

    #[test]
    fn loop_times_completes_after_n() {
        let mut track = KeyframeTrack::new()
            .push(0.0, 0.0_f32)
            .push(1.0, 100.0)
            .looping(Loop::Times(2));

        assert!(track.update(1.0)); // first loop done
        assert!(!track.update(1.0)); // second loop done
        assert!(track.is_complete());
    }

    #[test]
    fn out_of_bounds_clamps() {
        let track = KeyframeTrack::new().push(0.0, 0.0_f32).push(1.0, 100.0);
        assert!((track.value_at(-5.0).unwrap() - 0.0).abs() < 1e-6);
        assert!((track.value_at(99.0).unwrap() - 100.0).abs() < 1e-6);
    }

    #[test]
    fn with_easing() {
        let track = KeyframeTrack::new()
            .push_with_easing(0.0, 0.0_f32, Easing::EaseInQuad)
            .push(1.0, 100.0);
        // EaseInQuad at t=0.5 → 0.25, so value ≈ 25.0
        assert!((track.value_at(0.5).unwrap() - 25.0).abs() < 1e-4);
    }

    #[test]
    fn update_advances_value() {
        let mut track = KeyframeTrack::new().push(0.0, 0.0_f32).push(1.0, 100.0);
        track.update(0.5);
        assert!((track.value().unwrap() - 50.0).abs() < 1e-4);
    }
}
