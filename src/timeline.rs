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

// ── At (relative positioning) ───────────────────────────────────────────────

/// Relative placement tokens for [`Timeline::add_at`].
///
/// Instead of manually calculating offsets, use `At` variants to position
/// animations relative to existing timeline entries — GSAP-style.
///
/// # Example
///
/// ```rust
/// use spanda::timeline::{Timeline, At};
/// use spanda::tween::Tween;
///
/// let mut timeline = Timeline::new()
///     .add("fade", Tween::new(0.0_f32, 1.0).duration(0.5).build(), 0.0);
///
/// timeline.add_at("slide", Tween::new(0.0_f32, 100.0).duration(0.8).build(), 0.8, At::Start);
/// timeline.add_at("scale", Tween::new(1.0_f32, 2.0).duration(0.3).build(), 0.3, At::End);
/// timeline.add_at("glow", Tween::new(0.0_f32, 1.0).duration(0.4).build(), 0.4, At::Label("fade"));
/// timeline.add_at("pop", Tween::new(0.0_f32, 1.0).duration(0.2).build(), 0.2, At::Offset(0.1));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum At<'a> {
    /// Place at `t = 0.0` (the absolute start of the timeline).
    Start,
    /// Place after the latest-ending entry in the timeline.
    End,
    /// Place at the same start time as the entry with the given label.
    Label(&'a str),
    /// Place at the given number of seconds *after* the last-added entry ends.
    ///
    /// Positive values add a gap, negative values overlap.
    Offset(f32),
}

// ── TimelineEntry ────────────────────────────────────────────────────────────

/// Type of timeline entry.
#[derive(Debug, Clone, PartialEq)]
enum EntryKind {
    /// Regular animation entry.
    Animation,
    /// Callback entry (fires at a specific time).
    #[cfg(feature = "std")]
    Callback,
    /// Pause point (timeline pauses when reached).
    Pause,
}

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
    /// Entry kind (animation, callback, or pause).
    kind: EntryKind,
    /// Callback function for callback entries.
    #[cfg(feature = "std")]
    callback: Option<Box<dyn FnMut()>>,
}

impl core::fmt::Debug for TimelineEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TimelineEntry")
            .field("label", &self.label)
            .field("start_at", &self.start_at)
            .field("duration", &self.duration)
            .field("started", &self.started)
            .field("completed", &self.completed)
            .field("kind", &self.kind)
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
    /// Time scale multiplier applied to dt (default 1.0).
    time_scale: f32,
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
            .field("time_scale", &self.time_scale)
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
            time_scale: 1.0,
            #[cfg(feature = "std")]
            on_finish_callbacks: Vec::new(),
        }
    }

    /// Add a labelled animation starting at `start_at` seconds.
    ///
    /// The `duration` parameter specifies the animation's expected duration,
    /// which is used by [`At::End`] and [`At::Offset`] for relative positioning.
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
            duration: 0.0, // filled in by the Sequence builder or add_with_duration()
            started: false,
            completed: false,
            kind: EntryKind::Animation,
            #[cfg(feature = "std")]
            callback: None,
        });
        self
    }

    /// Add a labelled animation with explicit duration for correct relative positioning.
    ///
    /// Use this instead of [`Timeline::add`] when you need [`At::End`] or
    /// [`At::Offset`] to reference this entry's duration.
    pub fn add_with_duration<A: Update + 'static>(
        mut self,
        label: &str,
        animation: A,
        start_at: f32,
        duration: f32,
    ) -> Self {
        self.entries.push(TimelineEntry {
            label: label.to_string(),
            animation: Box::new(animation),
            start_at,
            duration,
            started: false,
            completed: false,
            kind: EntryKind::Animation,
            #[cfg(feature = "std")]
            callback: None,
        });
        self
    }

    /// Add a labelled animation at a position relative to existing entries.
    ///
    /// `duration` is the length of this animation in seconds (needed because
    /// trait objects cannot expose their own duration).
    ///
    /// See [`At`] for available positioning modes.
    pub fn add_at<A: Update + 'static>(
        &mut self,
        label: &str,
        animation: A,
        duration: f32,
        at: At<'_>,
    ) {
        let start_at = match at {
            At::Start => 0.0,
            At::End => {
                // After the latest-ending entry
                self.entries
                    .iter()
                    .map(|e| e.start_at + e.duration)
                    .fold(0.0_f32, f32::max)
            }
            At::Label(target) => {
                // Same start time as the entry with the given label
                self.entries
                    .iter()
                    .find(|e| e.label == target)
                    .map(|e| e.start_at)
                    .unwrap_or(0.0)
            }
            At::Offset(offset) => {
                // Relative to the last-added entry's end.
                // If no entries exist, treat as absolute.
                self.entries
                    .last()
                    .map(|e| e.start_at + e.duration + offset)
                    .unwrap_or(offset.max(0.0))
            }
        };

        self.entries.push(TimelineEntry {
            label: label.to_string(),
            animation: Box::new(animation),
            start_at: start_at.max(0.0),
            duration,
            started: false,
            completed: false,
            kind: EntryKind::Animation,
            #[cfg(feature = "std")]
            callback: None,
        });
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

    /// Set the time scale multiplier at runtime.
    ///
    /// Values > 1.0 speed up, < 1.0 slow down, 0.0 effectively pauses.
    pub fn set_time_scale(&mut self, scale: f32) {
        self.time_scale = scale;
    }

    /// Get the current time scale.
    pub fn time_scale(&self) -> f32 {
        self.time_scale
    }

    /// Total duration including all nested timelines (same as `duration()`).
    ///
    /// For consistency with GSAP's `totalDuration()`.
    pub fn total_duration(&self) -> f32 {
        self.duration()
    }

    /// Total progress from 0.0 to 1.0 (same as `progress()`).
    ///
    /// For consistency with GSAP's `totalProgress()`.
    pub fn total_progress(&self) -> f32 {
        self.progress()
    }

    /// Get the labels of all entries matching a predicate.
    ///
    /// GSAP-style `getTweensOf` equivalent — query animations by label pattern.
    ///
    /// # Example
    ///
    /// ```rust
    /// use spanda::timeline::Timeline;
    /// use spanda::tween::Tween;
    ///
    /// let timeline = Timeline::new()
    ///     .add("fade_in", Tween::new(0.0_f32, 1.0).duration(0.5).build(), 0.0)
    ///     .add("fade_out", Tween::new(1.0_f32, 0.0).duration(0.5).build(), 0.5);
    ///
    /// let fades: Vec<_> = timeline.get_entries_by_label(|l| l.starts_with("fade")).collect();
    /// assert_eq!(fades.len(), 2);
    /// ```
    pub fn get_entries_by_label<'a, F>(&'a self, predicate: F) -> impl Iterator<Item = &'a str>
    where
        F: Fn(&str) -> bool + 'a,
    {
        self.entries
            .iter()
            .filter(move |e| predicate(&e.label))
            .map(|e| e.label.as_str())
    }

    /// Insert a callback at a specific time in the timeline.
    ///
    /// GSAP-style `.call()` — the callback fires when the timeline reaches
    /// the specified time.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut timeline = Timeline::new()
    ///     .add("fade", Tween::new(0.0_f32, 1.0).duration(0.5).build(), 0.0);
    ///
    /// timeline.call(0.25, || println!("Halfway through fade!"));
    /// timeline.play();
    /// ```
    #[cfg(feature = "std")]
    pub fn call<F: FnMut() + 'static>(&mut self, time: f32, callback: F) {
        // Create a no-op animation that immediately completes
        struct NoOp;
        impl Update for NoOp {
            fn update(&mut self, _dt: f32) -> bool {
                false
            }
        }

        self.entries.push(TimelineEntry {
            label: format!("__call_{:.3}", time),
            animation: Box::new(NoOp),
            start_at: time.max(0.0),
            duration: 0.0,
            started: false,
            completed: false,
            kind: EntryKind::Callback,
            callback: Some(Box::new(callback)),
        });
    }

    /// Insert a pause point at a specific time in the timeline.
    ///
    /// GSAP-style `.addPause()` — the timeline pauses when it reaches this time.
    /// Use `timeline.resume()` to continue playback.
    ///
    /// # Example
    ///
    /// ```rust
    /// use spanda::timeline::{Timeline, TimelineState};
    /// use spanda::tween::Tween;
    /// use spanda::traits::Update;
    ///
    /// let mut timeline = Timeline::new()
    ///     .add("fade", Tween::new(0.0_f32, 1.0).duration(1.0).build(), 0.0);
    ///
    /// timeline.add_pause(0.5); // Pause halfway through
    /// timeline.play();
    /// timeline.update(0.6); // Will pause at 0.5
    /// assert_eq!(*timeline.state(), TimelineState::Paused);
    /// ```
    pub fn add_pause(&mut self, time: f32) {
        // Create a no-op animation that immediately completes
        struct NoOp;
        impl Update for NoOp {
            fn update(&mut self, _dt: f32) -> bool {
                false
            }
        }

        self.entries.push(TimelineEntry {
            label: format!("__pause_{:.3}", time),
            animation: Box::new(NoOp),
            start_at: time.max(0.0),
            duration: 0.0,
            started: false,
            completed: false,
            kind: EntryKind::Pause,
            #[cfg(feature = "std")]
            callback: None,
        });
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

        let dt = (dt * self.time_scale).max(0.0);
        self.elapsed += dt;

        let mut all_done = true;
        let mut should_pause = false;

        for entry in &mut self.entries {
            if entry.completed {
                continue;
            }

            // Check if this entry should be active
            if self.elapsed >= entry.start_at {
                match entry.kind {
                    EntryKind::Pause => {
                        if !entry.started {
                            entry.started = true;
                            entry.completed = true;
                            should_pause = true;
                            // Clamp elapsed to the pause point
                            self.elapsed = entry.start_at;
                        }
                    }
                    #[cfg(feature = "std")]
                    EntryKind::Callback => {
                        if !entry.started {
                            entry.started = true;
                            entry.completed = true;
                            // Fire the callback
                            if let Some(ref mut cb) = entry.callback {
                                cb();
                            }
                        }
                    }
                    EntryKind::Animation => {
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
                    }
                }
            } else {
                all_done = false;
            }
        }

        // Handle pause after processing all entries
        if should_pause {
            self.state = TimelineState::Paused;
            return true;
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
            time_scale: 1.0,
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
                kind: EntryKind::Animation,
                #[cfg(feature = "std")]
                callback: None,
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

// ── Stagger utility ─────────────────────────────────────────────────────────

/// Create a [`Timeline`] where each animation starts `stagger_delay` seconds
/// after the previous one.
///
/// This is the equivalent of GSAP's `stagger` property — instead of manually
/// calculating offsets for N animations, just pass them and the spacing.
///
/// Each tuple is `(animation, duration)`.  Duration is needed because trait
/// objects cannot expose their own duration.
///
/// # Example
///
/// ```rust
/// use spanda::timeline::stagger;
/// use spanda::tween::Tween;
/// use spanda::traits::Update;
///
/// let tweens: Vec<_> = (0..5).map(|i| {
///     let end = (i + 1) as f32 * 20.0;
///     (Tween::new(0.0_f32, end).duration(0.5).build(), 0.5)
/// }).collect();
///
/// let mut timeline = stagger(tweens, 0.1);
/// timeline.play();
/// // Animations start at t=0.0, 0.1, 0.2, 0.3, 0.4
/// ```
pub fn stagger<A: Update + 'static>(
    animations: Vec<(A, f32)>,
    stagger_delay: f32,
) -> Timeline {
    let mut timeline = Timeline {
        entries: Vec::new(),
        elapsed: 0.0,
        state: TimelineState::Idle,
        looping: Loop::Once,
        time_scale: 1.0,
        #[cfg(feature = "std")]
        on_finish_callbacks: Vec::new(),
    };

    for (i, (animation, duration)) in animations.into_iter().enumerate() {
        let start_at = i as f32 * stagger_delay;
        let label = format!("stagger_{}", i);
        timeline.entries.push(TimelineEntry {
            label,
            animation: Box::new(animation),
            start_at,
            duration,
            started: false,
            completed: false,
            kind: EntryKind::Animation,
            #[cfg(feature = "std")]
            callback: None,
        });
    }

    timeline
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

    // ── Time scale tests ────────────────────────────────────────────────────

    #[test]
    fn timeline_time_scale_double_speed() {
        let mut tl = Timeline::new()
            .add("t1", Tween::new(0.0_f32, 1.0).duration(1.0).build(), 0.0);
        tl.set_time_scale(2.0);
        tl.play();
        assert!(!tl.update(0.5)); // effective dt = 1.0, should complete
        assert_eq!(*tl.state(), TimelineState::Completed);
    }

    #[test]
    fn timeline_time_scale_half_speed() {
        let mut tl = Timeline::new()
            .add("t1", Tween::new(0.0_f32, 1.0).duration(1.0).build(), 0.0);
        tl.set_time_scale(0.5);
        tl.play();
        tl.update(1.0); // effective dt = 0.5
        assert_eq!(*tl.state(), TimelineState::Playing);
    }

    // ── Stagger tests ───────────────────────────────────────────────────────

    #[test]
    fn stagger_creates_offset_timeline() {
        let tweens: Vec<_> = (0..3).map(|_| {
            (Tween::new(0.0_f32, 1.0).duration(0.5).build(), 0.5)
        }).collect();

        let mut tl = stagger(tweens, 0.2);
        tl.play();

        // At t=0.2, first tween running, second about to start
        tl.update(0.2);
        assert_eq!(*tl.state(), TimelineState::Playing);

        // Total: last starts at 0.4, runs 0.5 = 0.9s
        let mut total = 0.2;
        while tl.update(0.01) {
            total += 0.01;
            if total > 5.0 { panic!("Stagger timeline didn't complete"); }
        }
        assert!(total >= 0.6 && total <= 1.0, "Expected ~0.7-0.9s, got {total}");
    }

    #[test]
    fn stagger_empty_vec_does_not_panic() {
        let tl = stagger::<Tween<f32>>(Vec::new(), 0.1);
        assert!((tl.duration() - 0.0).abs() < 1e-6);
    }

    // ── At (relative positioning) tests ────────────────────────────────────

    #[test]
    fn at_start_places_at_zero() {
        let mut tl = Timeline::new()
            .add("a", Tween::new(0.0_f32, 1.0).duration(0.5).build(), 0.5);

        tl.add_at("b", Tween::new(0.0_f32, 1.0).duration(0.3).build(), 0.3, At::Start);

        // Entry "b" should start at 0.0
        assert!(
            (tl.entries[1].start_at - 0.0).abs() < 1e-6,
            "Expected start_at 0.0, got {}",
            tl.entries[1].start_at
        );
    }

    #[test]
    fn at_end_places_after_last() {
        let mut tl = Timeline::new()
            .add("a", Tween::new(0.0_f32, 1.0).duration(0.5).build(), 0.0);
        // Give entry "a" a known duration for At::End to work
        tl.entries[0].duration = 0.5;

        tl.add_at("b", Tween::new(0.0_f32, 1.0).duration(0.3).build(), 0.3, At::End);

        // Entry "b" should start at 0.5 (end of "a")
        assert!(
            (tl.entries[1].start_at - 0.5).abs() < 1e-6,
            "Expected start_at 0.5, got {}",
            tl.entries[1].start_at
        );
    }

    #[test]
    fn at_label_places_at_same_time() {
        let mut tl = Timeline::new()
            .add("fade", Tween::new(0.0_f32, 1.0).duration(0.5).build(), 0.3);

        tl.add_at(
            "scale",
            Tween::new(1.0_f32, 2.0).duration(0.3).build(),
            0.3,
            At::Label("fade"),
        );

        // "scale" should start at the same time as "fade" (0.3)
        assert!(
            (tl.entries[1].start_at - 0.3).abs() < 1e-6,
            "Expected start_at 0.3, got {}",
            tl.entries[1].start_at
        );
    }

    #[test]
    fn at_offset_places_relative_to_previous() {
        let mut tl = Timeline::new()
            .add("a", Tween::new(0.0_f32, 1.0).duration(0.5).build(), 0.0);
        tl.entries[0].duration = 0.5;

        tl.add_at("b", Tween::new(0.0_f32, 1.0).duration(0.3).build(), 0.3, At::Offset(0.2));

        // "b" should start at 0.0 + 0.5 + 0.2 = 0.7
        assert!(
            (tl.entries[1].start_at - 0.7).abs() < 1e-6,
            "Expected start_at 0.7, got {}",
            tl.entries[1].start_at
        );
    }

    #[test]
    fn at_offset_negative_overlaps() {
        let mut tl = Timeline::new()
            .add("a", Tween::new(0.0_f32, 1.0).duration(1.0).build(), 0.0);
        tl.entries[0].duration = 1.0;

        tl.add_at("b", Tween::new(0.0_f32, 1.0).duration(0.5).build(), 0.5, At::Offset(-0.3));

        // "b" should start at 0.0 + 1.0 - 0.3 = 0.7
        assert!(
            (tl.entries[1].start_at - 0.7).abs() < 1e-6,
            "Expected start_at 0.7, got {}",
            tl.entries[1].start_at
        );
    }

    #[test]
    fn at_label_unknown_falls_back_to_zero() {
        let mut tl = Timeline::new()
            .add("a", Tween::new(0.0_f32, 1.0).duration(0.5).build(), 0.5);

        tl.add_at("b", Tween::new(0.0_f32, 1.0).duration(0.3).build(), 0.3, At::Label("nonexistent"));

        assert!(
            (tl.entries[1].start_at - 0.0).abs() < 1e-6,
            "Expected fallback to 0.0, got {}",
            tl.entries[1].start_at
        );
    }
}
