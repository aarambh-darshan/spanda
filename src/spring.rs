//! Physics-based spring animation — a damped harmonic oscillator.
//!
//! Unlike easing functions, a spring has no fixed duration.  It settles when
//! both displacement and velocity drop below [`SpringConfig::epsilon`].
//!
//! # Quick start
//!
//! ```rust
//! use spanda::spring::{Spring, SpringConfig};
//! use spanda::traits::Update;
//!
//! let mut spring = Spring::new(SpringConfig::wobbly());
//! spring.set_target(200.0);
//!
//! for _ in 0..1000 {
//!     spring.update(1.0 / 60.0);
//! }
//!
//! assert!((spring.position() - 200.0).abs() < 1.0);
//! ```

use crate::traits::Update;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// ── SpringConfig ─────────────────────────────────────────────────────────────

/// Parameters for a damped harmonic oscillator.
///
/// Use one of the built-in presets or construct your own.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SpringConfig {
    /// "Tightness" — higher means faster oscillation (default: 100.0).
    pub stiffness: f32,
    /// Resistance — higher means less bounce (default: 10.0).
    pub damping: f32,
    /// Inertia — higher means slower start (default: 1.0).
    pub mass: f32,
    /// Threshold below which the spring is considered settled (default: 0.001).
    pub epsilon: f32,
}

impl SpringConfig {
    /// A gentle, slow spring.
    ///
    /// `stiffness: 60, damping: 14`
    pub fn gentle() -> Self {
        Self {
            stiffness: 60.0,
            damping: 14.0,
            mass: 1.0,
            epsilon: 0.001,
        }
    }

    /// A wobbly, bouncy spring.
    ///
    /// `stiffness: 180, damping: 12`
    pub fn wobbly() -> Self {
        Self {
            stiffness: 180.0,
            damping: 12.0,
            mass: 1.0,
            epsilon: 0.001,
        }
    }

    /// A stiff, fast spring with minimal bounce.
    ///
    /// `stiffness: 210, damping: 20`
    pub fn stiff() -> Self {
        Self {
            stiffness: 210.0,
            damping: 20.0,
            mass: 1.0,
            epsilon: 0.001,
        }
    }

    /// A slow, relaxed spring.
    ///
    /// `stiffness: 37, damping: 14`
    pub fn slow() -> Self {
        Self {
            stiffness: 37.0,
            damping: 14.0,
            mass: 1.0,
            epsilon: 0.001,
        }
    }
}

impl Default for SpringConfig {
    fn default() -> Self {
        Self {
            stiffness: 100.0,
            damping: 10.0,
            mass: 1.0,
            epsilon: 0.001,
        }
    }
}

// ── Spring ───────────────────────────────────────────────────────────────────

/// A single-axis damped harmonic oscillator.
///
/// Call [`Spring::set_target`] to change the destination.  Each call to
/// [`Update::update`] integrates the physics one time-step forward.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "bevy", derive(bevy_ecs::component::Component))]
pub struct Spring {
    /// Spring parameters.
    pub config: SpringConfig,
    /// Current position.
    position: f32,
    /// Current velocity.
    velocity: f32,
    /// Target position the spring is moving toward.
    target: f32,
    /// Whether the spring is settled (displacement and velocity < epsilon).
    settled: bool,
}

impl Spring {
    /// Create a new spring starting at position 0 with the given config.
    pub fn new(config: SpringConfig) -> Self {
        Self {
            config,
            position: 0.0,
            velocity: 0.0,
            target: 0.0,
            settled: true,
        }
    }

    /// Create a spring starting at a specific position.
    pub fn with_position(mut self, position: f32) -> Self {
        self.position = position;
        self
    }

    /// Set a new target.  The spring immediately begins moving toward it.
    pub fn set_target(&mut self, target: f32) {
        self.target = target;
        self.settled = false;
    }

    /// Current position.
    pub fn position(&self) -> f32 {
        self.position
    }

    /// Current velocity.
    pub fn velocity(&self) -> f32 {
        self.velocity
    }

    /// Target value.
    pub fn target(&self) -> f32 {
        self.target
    }

    /// Whether the spring has settled to its target within [`SpringConfig::epsilon`].
    pub fn is_settled(&self) -> bool {
        self.settled
    }

    /// Reset position and velocity to zero.
    pub fn reset(&mut self) {
        self.position = 0.0;
        self.velocity = 0.0;
        self.settled = self.target.abs() < self.config.epsilon;
    }

    /// Semi-implicit Euler integration step.
    fn step(&mut self, dt: f32) {
        let displacement = self.position - self.target;
        let acceleration = (-self.config.stiffness * displacement
            - self.config.damping * self.velocity)
            / self.config.mass;
        self.velocity += acceleration * dt;
        self.position += self.velocity * dt;
    }

    /// Check if the spring has settled.
    fn check_settled(&mut self) {
        let displacement = (self.position - self.target).abs();
        let vel = self.velocity.abs();
        if displacement < self.config.epsilon && vel < self.config.epsilon {
            self.position = self.target;
            self.velocity = 0.0;
            self.settled = true;
        }
    }
}

impl Update for Spring {
    fn update(&mut self, dt: f32) -> bool {
        if self.settled {
            return false;
        }

        let dt = dt.max(0.0);

        // Handle degenerate cases
        if self.config.stiffness <= 0.0 {
            self.position = self.target;
            self.velocity = 0.0;
            self.settled = true;
            return false;
        }

        // Sub-step for large dt to maintain stability
        let max_step = 1.0 / 120.0; // 120 Hz minimum
        let mut remaining = dt;
        while remaining > 0.0 {
            let step_dt = remaining.min(max_step);
            self.step(step_dt);
            remaining -= step_dt;
        }

        self.check_settled();
        !self.settled
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spring_settles_to_target() {
        let mut spring = Spring::new(SpringConfig::default());
        spring.set_target(100.0);

        for _ in 0..1000 {
            spring.update(1.0 / 60.0);
        }

        assert!(
            (spring.position() - 100.0).abs() < 0.01,
            "Spring did not settle to target: pos={}",
            spring.position()
        );
        assert!(spring.is_settled());
    }

    #[test]
    fn stiff_spring_settles_faster() {
        let mut gentle = Spring::new(SpringConfig::gentle());
        let mut stiff = Spring::new(SpringConfig::stiff());

        gentle.set_target(100.0);
        stiff.set_target(100.0);

        let mut gentle_frames = 0;
        let mut stiff_frames = 0;

        for i in 0..5000 {
            if !gentle.is_settled() {
                gentle.update(1.0 / 60.0);
                gentle_frames = i;
            }
            if !stiff.is_settled() {
                stiff.update(1.0 / 60.0);
                stiff_frames = i;
            }
        }

        assert!(
            stiff_frames < gentle_frames,
            "Stiff ({stiff_frames}) should settle before gentle ({gentle_frames})"
        );
    }

    #[test]
    fn wobbly_spring_overshoots() {
        let mut spring = Spring::new(SpringConfig::wobbly());
        spring.set_target(100.0);

        let mut max_pos = 0.0_f32;
        for _ in 0..500 {
            spring.update(1.0 / 60.0);
            max_pos = max_pos.max(spring.position());
        }

        // Wobbly springs should overshoot the target
        assert!(
            max_pos > 100.0,
            "Wobbly spring should overshoot: max_pos={max_pos}"
        );
    }

    #[test]
    fn spring_zero_stiffness_snaps_to_target() {
        let mut spring = Spring::new(SpringConfig {
            stiffness: 0.0,
            damping: 10.0,
            mass: 1.0,
            epsilon: 0.001,
        });
        spring.set_target(42.0);
        spring.update(0.016);

        assert!((spring.position() - 42.0).abs() < 1e-6);
        assert!(spring.is_settled());
    }

    #[test]
    fn spring_negative_dt_treated_as_zero() {
        let mut spring = Spring::new(SpringConfig::default());
        spring.set_target(100.0);
        let pos_before = spring.position();
        spring.update(-0.1);
        assert!((spring.position() - pos_before).abs() < 1e-6);
    }

    #[test]
    fn spring_already_at_target_is_settled() {
        let mut spring = Spring::new(SpringConfig::default());
        // Target is 0, position is 0 — already settled
        assert!(spring.is_settled());
        assert!(!spring.update(0.016)); // returns false
    }

    #[test]
    fn spring_with_position_builder() {
        let spring = Spring::new(SpringConfig::default()).with_position(50.0);
        assert!((spring.position() - 50.0).abs() < 1e-6);
    }

    #[test]
    fn spring_presets_have_expected_values() {
        let g = SpringConfig::gentle();
        assert!((g.stiffness - 60.0).abs() < 1e-6);
        assert!((g.damping - 14.0).abs() < 1e-6);

        let w = SpringConfig::wobbly();
        assert!((w.stiffness - 180.0).abs() < 1e-6);
        assert!((w.damping - 12.0).abs() < 1e-6);

        let s = SpringConfig::stiff();
        assert!((s.stiffness - 210.0).abs() < 1e-6);
        assert!((s.damping - 20.0).abs() < 1e-6);

        let sl = SpringConfig::slow();
        assert!((sl.stiffness - 37.0).abs() < 1e-6);
        assert!((sl.damping - 14.0).abs() < 1e-6);
    }
}
