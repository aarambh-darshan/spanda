//! Animation driver — manages a collection of active animations.
//!
//! The [`AnimationDriver`] owns multiple animations and ticks them all each
//! frame.  Completed animations are automatically removed.
//!
//! # Example
//!
//! ```rust
//! use spanda::driver::AnimationDriver;
//! use spanda::tween::Tween;
//! use spanda::easing::Easing;
//!
//! let mut driver = AnimationDriver::new();
//!
//! let id = driver.add(
//!     Tween::new(0.0_f32, 1.0).duration(1.0).build()
//! );
//!
//! // Tick all animations:
//! driver.tick(0.5);
//! assert_eq!(driver.active_count(), 1);
//!
//! driver.tick(0.5);
//! assert_eq!(driver.active_count(), 0); // auto-removed
//! ```

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, vec::Vec};

use crate::traits::Update;

#[cfg(feature = "std")]
use std::sync::{Arc, Mutex};

// ── AnimationId ──────────────────────────────────────────────────────────────

/// Opaque handle to a running animation inside an [`AnimationDriver`].
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct AnimationId(u64);

impl AnimationId {
    /// Create an `AnimationId` from a raw counter value.
    ///
    /// This is public so that other drivers (e.g. [`ScrollDriver`](crate::scroll::ScrollDriver))
    /// can mint IDs. End users should not need to call this directly.
    pub fn new(raw: u64) -> Self {
        Self(raw)
    }
}

// ── AnimationDriver ──────────────────────────────────────────────────────────

/// Manages a set of active animations.
///
/// Call [`AnimationDriver::tick`] once per frame.  Completed animations are
/// cleaned up automatically.
pub struct AnimationDriver {
    // We use a trait object so the driver can hold heterogeneous animation types.
    animations: Vec<(AnimationId, Box<dyn Update>)>,
    next_id: u64,
}

impl AnimationDriver {
    /// Create an empty driver.
    pub fn new() -> Self {
        Self {
            animations: Vec::new(),
            next_id: 0,
        }
    }

    /// Add an animation and receive an [`AnimationId`] to cancel it later.
    pub fn add<A: Update + 'static>(&mut self, animation: A) -> AnimationId {
        let id = AnimationId(self.next_id);
        self.next_id += 1;
        self.animations.push((id, Box::new(animation)));
        id
    }

    /// Tick every active animation forward by `dt` seconds.
    ///
    /// Completed animations (where [`Update::update`] returns `false`) are
    /// removed automatically.
    pub fn tick(&mut self, dt: f32) {
        self.animations.retain_mut(|(_, anim)| anim.update(dt));
    }

    /// Cancel a specific animation by its [`AnimationId`].
    pub fn cancel(&mut self, id: AnimationId) {
        self.animations.retain(|(aid, _)| *aid != id);
    }

    /// Cancel all active animations.
    pub fn cancel_all(&mut self) {
        self.animations.clear();
    }

    /// Number of currently active (non-completed) animations.
    pub fn active_count(&self) -> usize {
        self.animations.len()
    }
}

impl Default for AnimationDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Debug for AnimationDriver {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AnimationDriver")
            .field("active_count", &self.animations.len())
            .field("next_id", &self.next_id)
            .finish()
    }
}

// ── AnimationDriverArc (thread-safe, std only) ───────────────────────────────

/// Thread-safe wrapper around [`AnimationDriver`].
///
/// Wraps the driver in `Arc<Mutex<>>` so it can be shared across threads
/// (e.g. audio thread + render thread).
#[cfg(feature = "std")]
#[derive(Clone, Debug)]
pub struct AnimationDriverArc(Arc<Mutex<AnimationDriver>>);

#[cfg(feature = "std")]
impl AnimationDriverArc {
    /// Create a new thread-safe driver.
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(AnimationDriver::new())))
    }

    /// Add an animation.
    pub fn add<A: Update + 'static>(&self, animation: A) -> AnimationId {
        self.0.lock().unwrap().add(animation)
    }

    /// Tick all active animations.
    pub fn tick(&self, dt: f32) {
        self.0.lock().unwrap().tick(dt);
    }

    /// Cancel an animation by ID.
    pub fn cancel(&self, id: AnimationId) {
        self.0.lock().unwrap().cancel(id);
    }

    /// Cancel all animations.
    pub fn cancel_all(&self) {
        self.0.lock().unwrap().cancel_all();
    }

    /// Number of active animations.
    pub fn active_count(&self) -> usize {
        self.0.lock().unwrap().active_count()
    }
}

#[cfg(feature = "std")]
impl Default for AnimationDriverArc {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tween::Tween;

    #[test]
    fn completed_animations_are_removed() {
        let mut driver = AnimationDriver::new();
        driver.add(Tween::new(0.0_f32, 1.0).duration(0.5).build());
        assert_eq!(driver.active_count(), 1);

        driver.tick(0.5);
        assert_eq!(driver.active_count(), 0);
    }

    #[test]
    fn cancel_removes_animation() {
        let mut driver = AnimationDriver::new();
        let id = driver.add(Tween::new(0.0_f32, 1.0).duration(10.0).build());
        assert_eq!(driver.active_count(), 1);

        driver.cancel(id);
        assert_eq!(driver.active_count(), 0);
    }

    #[test]
    fn cancel_all_clears_everything() {
        let mut driver = AnimationDriver::new();
        driver.add(Tween::new(0.0_f32, 1.0).duration(10.0).build());
        driver.add(Tween::new(0.0_f32, 1.0).duration(10.0).build());
        driver.add(Tween::new(0.0_f32, 1.0).duration(10.0).build());
        assert_eq!(driver.active_count(), 3);

        driver.cancel_all();
        assert_eq!(driver.active_count(), 0);
    }

    #[test]
    fn multiple_animations_tick_independently() {
        let mut driver = AnimationDriver::new();
        driver.add(Tween::new(0.0_f32, 1.0).duration(0.5).build());
        driver.add(Tween::new(0.0_f32, 1.0).duration(1.0).build());

        driver.tick(0.5);
        // First one is done, second is still running
        assert_eq!(driver.active_count(), 1);

        driver.tick(0.5);
        assert_eq!(driver.active_count(), 0);
    }

    #[test]
    fn cancel_nonexistent_id_is_noop() {
        let mut driver = AnimationDriver::new();
        driver.add(Tween::new(0.0_f32, 1.0).duration(10.0).build());
        driver.cancel(AnimationId(999));
        assert_eq!(driver.active_count(), 1);
    }
}
