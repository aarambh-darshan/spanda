//! Timeline and Sequence — compose multiple animations together.
//!
//! A [`Timeline`] holds labelled animations that play concurrently or at
//! staggered offsets.  [`Sequence`] is convenience sugar that chains
//! animations end-to-end.
//!
//! # Example — concurrent animations
//!
//! ```rust
//! use spanda::timeline::Timeline;
//! use spanda::tween::Tween;
//! use spanda::easing::Easing;
//! use spanda::traits::Update;
//!
//! let mut timeline = Timeline::new()
//!     .add("fade_in",  Tween::new(0.0_f32, 1.0).duration(0.5).build(), 0.0)
//!     .add("slide_up", Tween::new(100.0_f32, 0.0).duration(0.8).build(), 0.0);
//!
//! timeline.play();
//! timeline.update(0.3);
//! ```

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, format, string::String, vec::Vec};

use crate::keyframe::Loop;
use crate::traits::Update;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// ── TimelineState ────────────────────────────────────────────────────────────

/// Current playback state of a [`Timeline`].
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TimelineState {
    /// Not yet started.
    Idle,
    /// Currently playing.
    Playing,
    /// Manually paused.
    Paused,
    /// All entries have completed.
    Completed,
}

// ── TimelineEntry ────────────────────────────────────────────────────────────

/// A single entry in a [`Timeline`].
struct TimelineEntry {
    /// Human-readable label.
    #[allow(dead_code)]
    label: String,
    /// The animation itself (Tween, KeyframeTrack, Spring, etc.).
    animation: Box<dyn Update>,
    /// Seconds from the timeline start when this entry begins playing.
    start_at: f32,
    /// Duration of this entry (used for sequencing/scheduling).
    duration: f32,
    /// Whether this entry has been started.
    started: bool,
    /// Whether this entry has completed.
    completed: bool,
}

impl core::fmt::Debug for TimelineEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TimelineEntry")
            .field("label", &self.label)
            .field("start_at", &self.start_at)
            .field("duration", &self.duration)
            .field("started", &self.started)
            .field("completed", &self.completed)
            .finish()
    }
}

// ── Timeline ─────────────────────────────────────────────────────────────────

/// A collection of animations that play concurrently with per-entry offsets.
///
/// Use [`Timeline::add`] to schedule animations at specific times, or use
/// [`Sequence`] for sequential chaining.
pub struct Timeline {
    entries: Vec<TimelineEntry>,
    elapsed: f32,
    state: TimelineState,
    looping: Loop,
    /// Callbacks that fire when the timeline completes (std only).
    #[cfg(feature = "std")]
    on_finish_callbacks: Vec<Box<dyn FnMut()>>,
}

impl core::fmt::Debug for Timeline {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Timeline")
            .field("entries", &self.entries)
            .field("elapsed", &self.elapsed)
            .field("state", &self.state)
            .field("looping", &self.looping)
            .finish()
    }
}

impl Timeline {
    /// Create an empty timeline.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            elapsed: 0.0,
            state: TimelineState::Idle,
            looping: Loop::Once,
            #[cfg(feature = "std")]
            on_finish_callbacks: Vec::new(),
        }
    }

    /// Add a labelled animation starting at `start_at` seconds.
    pub fn add<A: Update + 'static>(
        mut self,
        label: &str,
        animation: A,
        start_at: f32,
    ) -> Self {
        // We estimate duration by the animation — for tweens we can't directly
        // read it here because the trait is erased, so the caller can set it.
        self.entries.push(TimelineEntry {
            label: label.to_string(),
            animation: Box::new(animation),
            start_at,
            duration: 0.0, // filled in by the Sequence builder
            started: false,
            completed: false,
        });
        self
    }

    /// Set the loop mode.
    pub fn looping(mut self, mode: Loop) -> Self {
        self.looping = mode;
        self
    }

    /// Start playing the timeline.
    pub fn play(&mut self) {
        self.state = TimelineState::Playing;
    }

    /// Pause the timeline.
    pub fn pause(&mut self) {
        if self.state == TimelineState::Playing {
            self.state = TimelineState::Paused;
        }
    }

    /// Resume after pause.
    pub fn resume(&mut self) {
        if self.state == TimelineState::Paused {
            self.state = TimelineState::Playing;
        }
    }

    /// Jump to a specific time.
    pub fn seek(&mut self, t: f32) {
        self.elapsed = t.max(0.0);
        // Reset all entries
        for entry in &mut self.entries {
            entry.started = false;
            entry.completed = false;
        }
    }

    /// Reset to the beginning.
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
        self.state = TimelineState::Idle;
        for entry in &mut self.entries {
            entry.started = false;
            entry.completed = false;
        }
    }

    /// Total duration from first entry start to last entry end.
    pub fn duration(&self) -> f32 {
        self.entries
            .iter()
            .map(|e| e.start_at + e.duration)
            .fold(0.0_f32, f32::max)
    }

    /// Progress from 0.0 to 1.0.
    pub fn progress(&self) -> f32 {
        let dur = self.duration();
        if dur <= 0.0 {
            return 1.0;
        }
        (self.elapsed / dur).clamp(0.0, 1.0)
    }

    /// Current state.
    pub fn state(&self) -> &TimelineState {
        &self.state
    }

    /// Register a callback to fire when the timeline completes.
    #[cfg(feature = "std")]
    pub fn on_finish<F: FnMut() + 'static>(&mut self, callback: F) {
        self.on_finish_callbacks.push(Box::new(callback));
    }
}

impl Default for Timeline {
    fn default() -> Self {
        Self::new()
    }
}

impl Update for Timeline {
    fn update(&mut self, dt: f32) -> bool {
        if self.state != TimelineState::Playing {
            return self.state != TimelineState::Completed;
        }

        let dt = dt.max(0.0);
        self.elapsed += dt;

        let mut all_done = true;

        for entry in &mut self.entries {
            if entry.completed {
                continue;
            }

            // Check if this entry should be active
            if self.elapsed >= entry.start_at {
                // Compute the effective dt for this entry.
                // If the entry just started this frame, only give it the leftover
                // time after its start_at, not the full frame dt.
                let entry_dt = if !entry.started {
                    entry.started = true;
                    // Time that has elapsed since this entry's start_at
                    (self.elapsed - entry.start_at).min(dt)
                } else {
                    dt
                };

                let still_running = entry.animation.update(entry_dt);
                if !still_running {
                    entry.completed = true;
                } else {
                    all_done = false;
                }
            } else {
                all_done = false;
            }
        }

        if all_done && !self.entries.is_empty() {
            self.state = TimelineState::Completed;

            #[cfg(feature = "std")]
            {
                for cb in &mut self.on_finish_callbacks {
                    cb();
                }
            }

            return false;
        }

        true
    }
}

// ── Sequence ─────────────────────────────────────────────────────────────────

/// Sugar for building sequential (end-to-end) animations.
///
/// Each animation starts when the previous one ends.  Use [`Sequence::gap`]
/// to insert pauses between steps.
///
/// # Example
///
/// ```rust
/// use spanda::timeline::Sequence;
/// use spanda::tween::Tween;
/// use spanda::easing::Easing;
///
/// let mut seq = Sequence::new()
///     .then(Tween::new(0.0_f32, 100.0).duration(0.3).build(), 0.3)
///     .gap(0.1)
///     .then(Tween::new(1.0_f32, 0.0).duration(0.2).build(), 0.2);
///
/// let mut timeline = seq.build();
/// timeline.play();
/// ```
pub struct Sequence {
    entries: Vec<(String, Box<dyn Update>, f32, f32)>, // (label, anim, start_at, duration)
    cursor: f32,
    label_counter: u32,
    looping: Loop,
}

impl Sequence {
    /// Create an empty sequence.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            cursor: 0.0,
            label_counter: 0,
            looping: Loop::Once,
        }
    }

    /// Append an animation.  It starts when the previous animation ends.
    ///
    /// `duration` is the length of this animation in seconds (needed because
    /// trait objects cannot expose their duration).
    pub fn then<A: Update + 'static>(mut self, animation: A, duration: f32) -> Self {
        let label = format!("seq_{}", self.label_counter);
        self.label_counter += 1;
        let start_at = self.cursor;
        self.entries
            .push((label, Box::new(animation), start_at, duration));
        self.cursor += duration;
        self
    }

    /// Insert a gap (pause) in seconds.
    pub fn gap(mut self, seconds: f32) -> Self {
        self.cursor += seconds;
        self
    }

    /// Set the loop mode for the resulting timeline.
    pub fn looping(mut self, mode: Loop) -> Self {
        self.looping = mode;
        self
    }

    /// Build the final [`Timeline`].
    pub fn build(self) -> Timeline {
        let mut timeline = Timeline {
            entries: Vec::new(),
            elapsed: 0.0,
            state: TimelineState::Idle,
            looping: self.looping,
            #[cfg(feature = "std")]
            on_finish_callbacks: Vec::new(),
        };

        for (label, animation, start_at, duration) in self.entries {
            timeline.entries.push(TimelineEntry {
                label,
                animation,
                start_at,
                duration,
                started: false,
                completed: false,
            });
        }

        timeline
    }
}

impl Default for Sequence {
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Debug for Sequence {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Sequence")
            .field("cursor", &self.cursor)
            .field("entries_count", &self.entries.len())
            .finish()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tween::Tween;

    #[test]
    fn timeline_plays_to_completion() {
        let mut tl = Timeline::new()
            .add("t1", Tween::new(0.0_f32, 1.0).duration(0.5).build(), 0.0);
        tl.play();

        assert!(tl.update(0.3));
        assert!(!tl.update(0.3));
        assert_eq!(*tl.state(), TimelineState::Completed);
    }

    #[test]
    fn timeline_concurrent_entries() {
        let mut tl = Timeline::new()
            .add("a", Tween::new(0.0_f32, 1.0).duration(0.5).build(), 0.0)
            .add("b", Tween::new(0.0_f32, 1.0).duration(1.0).build(), 0.0);
        tl.play();

        tl.update(0.5); // 'a' done, 'b' still running
        assert_eq!(*tl.state(), TimelineState::Playing);

        tl.update(0.5); // 'b' done
        assert_eq!(*tl.state(), TimelineState::Completed);
    }

    #[test]
    fn timeline_staggered_start() {
        let mut tl = Timeline::new()
            .add("a", Tween::new(0.0_f32, 1.0).duration(0.5).build(), 0.0)
            .add("b", Tween::new(0.0_f32, 1.0).duration(0.5).build(), 0.5);
        tl.play();

        tl.update(0.5); // 'a' done, 'b' starts
        assert_eq!(*tl.state(), TimelineState::Playing);

        tl.update(0.5); // 'b' done
        assert_eq!(*tl.state(), TimelineState::Completed);
    }

    #[test]
    fn timeline_pause_and_resume() {
        let mut tl = Timeline::new()
            .add("a", Tween::new(0.0_f32, 1.0).duration(1.0).build(), 0.0);
        tl.play();

        tl.update(0.3);
        tl.pause();
        assert_eq!(*tl.state(), TimelineState::Paused);

        tl.update(0.5); // should not advance
        assert_eq!(*tl.state(), TimelineState::Paused);

        tl.resume();
        assert_eq!(*tl.state(), TimelineState::Playing);
    }

    #[test]
    fn timeline_reset() {
        let mut tl = Timeline::new()
            .add("a", Tween::new(0.0_f32, 1.0).duration(0.5).build(), 0.0);
        tl.play();
        tl.update(0.5);
        assert_eq!(*tl.state(), TimelineState::Completed);

        tl.reset();
        assert_eq!(*tl.state(), TimelineState::Idle);
    }

    #[test]
    fn sequence_chains_animations() {
        let seq = Sequence::new()
            .then(Tween::new(0.0_f32, 1.0).duration(0.5).build(), 0.5)
            .gap(0.1)
            .then(Tween::new(0.0_f32, 1.0).duration(0.3).build(), 0.3);

        let mut tl = seq.build();
        tl.play();

        tl.update(0.5); // first tween done
        assert_eq!(*tl.state(), TimelineState::Playing);

        tl.update(0.1); // gap
        tl.update(0.3); // second tween
        assert_eq!(*tl.state(), TimelineState::Completed);
    }

    #[test]
    fn empty_timeline_does_not_panic() {
        let mut tl = Timeline::new();
        tl.play();
        tl.update(1.0);
    }

    #[cfg(feature = "std")]
    #[test]
    fn on_finish_callback_fires() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;

        let fired = Arc::new(AtomicBool::new(false));
        let fired_clone = fired.clone();

        let mut tl = Timeline::new()
            .add("a", Tween::new(0.0_f32, 1.0).duration(0.5).build(), 0.0);
        tl.on_finish(move || {
            fired_clone.store(true, Ordering::SeqCst);
        });
        tl.play();

        tl.update(0.5);
        assert!(fired.load(Ordering::SeqCst));
    }
}
