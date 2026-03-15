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
//!
//! # Generic springs
//!
//! `Spring` animates a single `f32`.  For multi-dimensional types, use
//! [`SpringN`] which internally manages an array of spring components:
//!
//! ```rust
//! use spanda::spring::{SpringN, SpringConfig};
//! use spanda::traits::Update;
//!
//! let mut spring = SpringN::new(SpringConfig::wobbly(), [0.0_f32, 0.0]);
//! spring.set_target([100.0, 200.0]);
//!
//! for _ in 0..1000 {
//!     spring.update(1.0 / 60.0);
//! }
//!
//! let pos = spring.position();
//! assert!((pos[0] - 100.0).abs() < 1.0);
//! assert!((pos[1] - 200.0).abs() < 1.0);
//! ```

use crate::traits::Update;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
use alloc::vec;

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

// ── Spring (f32) ────────────────────────────────────────────────────────────

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

// ── SpringN<T> — generic multi-dimensional spring ───────────────────────────

/// A multi-dimensional spring that animates any type implementing
/// [`SpringAnimatable`].
///
/// Internally maintains one position+velocity pair per component, using the
/// same semi-implicit Euler integration as [`Spring`].
///
/// # Supported types
///
/// Out of the box: `f32`, `[f32; 2]`, `[f32; 3]`, `[f32; 4]`.
/// Implement [`SpringAnimatable`] on your own types to extend support.
///
/// # Example
///
/// ```rust
/// use spanda::spring::{SpringN, SpringConfig};
/// use spanda::traits::Update;
///
/// // 2D spring: position, target, etc.
/// let mut spring = SpringN::new(SpringConfig::wobbly(), [0.0_f32, 0.0]);
/// spring.set_target([100.0, 200.0]);
///
/// for _ in 0..1000 {
///     spring.update(1.0 / 60.0);
/// }
///
/// let pos = spring.position();
/// assert!((pos[0] - 100.0).abs() < 1.0);
/// assert!((pos[1] - 200.0).abs() < 1.0);
/// ```
pub struct SpringN<T: SpringAnimatable> {
    /// Spring parameters.
    pub config: SpringConfig,
    /// Per-component positions.
    positions: Vec<f32>,
    /// Per-component velocities.
    velocities: Vec<f32>,
    /// Target value.
    target: T,
    /// Current value (reconstructed from components after each step).
    current: T,
    /// Whether the spring has settled.
    settled: bool,
}

impl<T: SpringAnimatable> SpringN<T> {
    /// Create a new multi-dimensional spring at the given initial value.
    pub fn new(config: SpringConfig, initial: T) -> Self {
        let components = initial.to_components();
        let n = components.len();
        let positions = components;
        let velocities = vec![0.0; n];
        Self {
            config,
            positions,
            velocities,
            target: initial.clone(),
            current: initial,
            settled: true,
        }
    }

    /// Set a new target.  The spring immediately begins moving toward it.
    pub fn set_target(&mut self, target: T) {
        self.target = target;
        self.settled = false;
    }

    /// Current position as the animated type.
    pub fn position(&self) -> T {
        self.current.clone()
    }

    /// Current position components as a raw slice.
    pub fn position_components(&self) -> &[f32] {
        &self.positions
    }

    /// Current velocity components as a raw slice.
    pub fn velocity_components(&self) -> &[f32] {
        &self.velocities
    }

    /// Target value.
    pub fn target(&self) -> &T {
        &self.target
    }

    /// Whether the spring has settled to its target within [`SpringConfig::epsilon`].
    pub fn is_settled(&self) -> bool {
        self.settled
    }

    /// Reset all positions and velocities to zero.
    pub fn reset(&mut self) {
        for p in &mut self.positions {
            *p = 0.0;
        }
        for v in &mut self.velocities {
            *v = 0.0;
        }
        self.current = T::from_components(&self.positions);
        self.check_settled();
    }

    /// Semi-implicit Euler step for all components.
    fn step(&mut self, dt: f32) {
        let target_components = self.target.to_components();
        for i in 0..self.positions.len() {
            let displacement = self.positions[i] - target_components[i];
            let acceleration = (-self.config.stiffness * displacement
                - self.config.damping * self.velocities[i])
                / self.config.mass;
            self.velocities[i] += acceleration * dt;
            self.positions[i] += self.velocities[i] * dt;
        }
    }

    /// Check if all components are settled.
    fn check_settled(&mut self) {
        let target_components = self.target.to_components();
        let eps = self.config.epsilon;
        let all_settled = self
            .positions
            .iter()
            .zip(self.velocities.iter())
            .zip(target_components.iter())
            .all(|((p, v), t)| (p - t).abs() < eps && v.abs() < eps);

        if all_settled {
            let tc = self.target.to_components();
            for (i, t) in tc.iter().enumerate() {
                self.positions[i] = *t;
                self.velocities[i] = 0.0;
            }
            self.current = self.target.clone();
            self.settled = true;
        } else {
            self.current = T::from_components(&self.positions);
        }
    }
}

impl<T: SpringAnimatable> Update for SpringN<T> {
    fn update(&mut self, dt: f32) -> bool {
        if self.settled {
            return false;
        }

        let dt = dt.max(0.0);

        // Handle degenerate cases
        if self.config.stiffness <= 0.0 {
            let tc = self.target.to_components();
            for (i, t) in tc.iter().enumerate() {
                self.positions[i] = *t;
                self.velocities[i] = 0.0;
            }
            self.current = self.target.clone();
            self.settled = true;
            return false;
        }

        // Sub-step for stability
        let max_step = 1.0 / 120.0;
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

impl<T: SpringAnimatable> Clone for SpringN<T> {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            positions: self.positions.clone(),
            velocities: self.velocities.clone(),
            target: self.target.clone(),
            current: self.current.clone(),
            settled: self.settled,
        }
    }
}

impl<T: SpringAnimatable + core::fmt::Debug> core::fmt::Debug for SpringN<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SpringN")
            .field("config", &self.config)
            .field("current", &self.current)
            .field("target", &self.target)
            .field("settled", &self.settled)
            .field("components", &self.positions.len())
            .finish()
    }
}

// ── SpringAnimatable trait ──────────────────────────────────────────────────

/// Trait for types that can be decomposed into / reconstructed from `f32`
/// components for multi-dimensional spring simulation.
///
/// Implement this on your own types to use them with [`SpringN`].
pub trait SpringAnimatable: Clone + 'static {
    /// Decompose the value into a flat `Vec<f32>` of components.
    fn to_components(&self) -> Vec<f32>;

    /// Reconstruct the value from a slice of f32 components.
    ///
    /// The slice length must match what [`to_components`](SpringAnimatable::to_components) returns.
    fn from_components(components: &[f32]) -> Self;
}

impl SpringAnimatable for f32 {
    fn to_components(&self) -> Vec<f32> {
        vec![*self]
    }
    fn from_components(c: &[f32]) -> Self {
        c[0]
    }
}

impl SpringAnimatable for [f32; 2] {
    fn to_components(&self) -> Vec<f32> {
        vec![self[0], self[1]]
    }
    fn from_components(c: &[f32]) -> Self {
        [c[0], c[1]]
    }
}

impl SpringAnimatable for [f32; 3] {
    fn to_components(&self) -> Vec<f32> {
        vec![self[0], self[1], self[2]]
    }
    fn from_components(c: &[f32]) -> Self {
        [c[0], c[1], c[2]]
    }
}

impl SpringAnimatable for [f32; 4] {
    fn to_components(&self) -> Vec<f32> {
        vec![self[0], self[1], self[2], self[3]]
    }
    fn from_components(c: &[f32]) -> Self {
        [c[0], c[1], c[2], c[3]]
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Spring (f32) tests ───────────────────────────────────────────────

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
        assert!(spring.is_settled());
        assert!(!spring.update(0.016));
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

    // ── SpringN (multi-dimensional) tests ────────────────────────────────

    #[test]
    fn spring_n_f32_settles() {
        let mut spring = SpringN::new(SpringConfig::default(), 0.0_f32);
        spring.set_target(100.0);

        for _ in 0..1000 {
            spring.update(1.0 / 60.0);
        }

        assert!(
            (spring.position() - 100.0).abs() < 0.01,
            "SpringN<f32> did not settle: {}",
            spring.position()
        );
        assert!(spring.is_settled());
    }

    #[test]
    fn spring_n_vec2_settles() {
        let mut spring = SpringN::new(SpringConfig::default(), [0.0_f32, 0.0]);
        spring.set_target([100.0, 200.0]);

        for _ in 0..1000 {
            spring.update(1.0 / 60.0);
        }

        let pos = spring.position();
        assert!(
            (pos[0] - 100.0).abs() < 0.1 && (pos[1] - 200.0).abs() < 0.1,
            "SpringN<[f32;2]> did not settle: {:?}",
            pos
        );
        assert!(spring.is_settled());
    }

    #[test]
    fn spring_n_vec3_settles() {
        let mut spring = SpringN::new(SpringConfig::stiff(), [0.0_f32, 0.0, 0.0]);
        spring.set_target([50.0, 100.0, 150.0]);

        for _ in 0..1000 {
            spring.update(1.0 / 60.0);
        }

        let pos = spring.position();
        assert!((pos[0] - 50.0).abs() < 0.1, "x: {}", pos[0]);
        assert!((pos[1] - 100.0).abs() < 0.1, "y: {}", pos[1]);
        assert!((pos[2] - 150.0).abs() < 0.1, "z: {}", pos[2]);
        assert!(spring.is_settled());
    }

    #[test]
    fn spring_n_vec4_settles() {
        let mut spring = SpringN::new(SpringConfig::gentle(), [0.0_f32; 4]);
        spring.set_target([1.0, 0.5, 0.0, 0.8]);

        for _ in 0..2000 {
            spring.update(1.0 / 60.0);
        }

        let pos = spring.position();
        assert!((pos[0] - 1.0).abs() < 0.01);
        assert!((pos[1] - 0.5).abs() < 0.01);
        assert!((pos[2] - 0.0).abs() < 0.01);
        assert!((pos[3] - 0.8).abs() < 0.01);
        assert!(spring.is_settled());
    }

    #[test]
    fn spring_n_retarget_mid_flight() {
        let mut spring = SpringN::new(SpringConfig::wobbly(), [0.0_f32, 0.0]);
        spring.set_target([100.0, 100.0]);

        for _ in 0..30 {
            spring.update(1.0 / 60.0);
        }

        assert!(!spring.is_settled());

        spring.set_target([200.0, 0.0]);

        for _ in 0..2000 {
            spring.update(1.0 / 60.0);
        }

        let pos = spring.position();
        assert!((pos[0] - 200.0).abs() < 0.1);
        assert!((pos[1] - 0.0).abs() < 0.1);
        assert!(spring.is_settled());
    }

    #[test]
    fn spring_n_zero_stiffness_snaps() {
        let mut spring = SpringN::new(
            SpringConfig {
                stiffness: 0.0,
                damping: 10.0,
                mass: 1.0,
                epsilon: 0.001,
            },
            [0.0_f32, 0.0],
        );
        spring.set_target([42.0, 99.0]);
        spring.update(0.016);

        let pos = spring.position();
        assert!((pos[0] - 42.0).abs() < 1e-6);
        assert!((pos[1] - 99.0).abs() < 1e-6);
        assert!(spring.is_settled());
    }

    #[test]
    fn spring_n_reset() {
        let mut spring = SpringN::new(SpringConfig::default(), [50.0_f32, 50.0]);
        spring.set_target([100.0, 100.0]);
        spring.update(0.1);
        spring.reset();

        let pos = spring.position();
        assert!((pos[0] - 0.0).abs() < 1e-6);
        assert!((pos[1] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn spring_n_clone_and_debug() {
        let spring = SpringN::new(SpringConfig::default(), [1.0_f32, 2.0]);
        let _cloned = spring.clone();
        let _debug = format!("{:?}", spring);
    }

    #[test]
    fn spring_n_wobbly_overshoots_in_2d() {
        let mut spring = SpringN::new(SpringConfig::wobbly(), [0.0_f32, 0.0]);
        spring.set_target([100.0, 100.0]);

        let mut max_x = 0.0_f32;
        let mut max_y = 0.0_f32;
        for _ in 0..500 {
            spring.update(1.0 / 60.0);
            let pos = spring.position();
            max_x = max_x.max(pos[0]);
            max_y = max_y.max(pos[1]);
        }

        assert!(max_x > 100.0, "2D wobbly should overshoot x: {max_x}");
        assert!(max_y > 100.0, "2D wobbly should overshoot y: {max_y}");
    }
}
