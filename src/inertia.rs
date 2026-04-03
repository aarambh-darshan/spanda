//! Inertia-based deceleration — friction physics without a target.
//!
//! Unlike a [`Spring`](crate::spring::Spring) which pulls toward a target,
//! `Inertia` simply decelerates from an initial velocity until it stops.
//! Useful for scroll momentum, fling gestures, and swipe-to-dismiss.
//!
//! # Example
//!
//! ```rust
//! use spanda::inertia::{Inertia, InertiaConfig};
//! use spanda::traits::Update;
//!
//! let mut inertia = Inertia::new(InertiaConfig::default_flick())
//!     .with_velocity(500.0);
//!
//! // Simulate at 60 fps
//! for _ in 0..300 {
//!     if !inertia.update(1.0 / 60.0) {
//!         break; // settled
//!     }
//! }
//! assert!(inertia.is_settled());
//! assert!(inertia.velocity().abs() < 0.2);
//! ```

use crate::spring::SpringAnimatable;
#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec};

#[cfg(not(feature = "std"))]
#[allow(unused_imports)]
use num_traits::Float as _;

use crate::traits::Update;

/// Configuration for inertia deceleration.
#[derive(Clone, Debug)]
pub struct InertiaConfig {
    /// Friction coefficient (0.0 = no friction, 1.0 = instant stop).
    /// Typical range: 0.02–0.1.
    pub friction: f32,
    /// Velocity threshold below which the animation is considered settled.
    pub epsilon: f32,
}

impl InertiaConfig {
    /// Default flick preset — moderate deceleration.
    pub fn default_flick() -> Self {
        Self {
            friction: 0.05,
            epsilon: 0.1,
        }
    }

    /// Heavy preset — slow deceleration, long coast.
    pub fn heavy() -> Self {
        Self {
            friction: 0.02,
            epsilon: 0.1,
        }
    }

    /// Snappy preset — fast deceleration, quick stop.
    pub fn snappy() -> Self {
        Self {
            friction: 0.1,
            epsilon: 0.1,
        }
    }
}

impl Default for InertiaConfig {
    fn default() -> Self {
        Self::default_flick()
    }
}

/// Single-axis inertia deceleration.
///
/// Call [`Inertia::with_velocity`] to set the initial velocity, then tick
/// with `update(dt)` each frame. Position accumulates as velocity decays.
#[derive(Clone, Debug)]
pub struct Inertia {
    /// Configuration (mutable for live tuning).
    pub config: InertiaConfig,
    velocity: f32,
    position: f32,
    settled: bool,
}

impl Inertia {
    /// Create new inertia with the given config. Starts at position 0, velocity 0.
    pub fn new(config: InertiaConfig) -> Self {
        Self {
            config,
            velocity: 0.0,
            position: 0.0,
            settled: true,
        }
    }

    /// Set initial velocity (builder-style).
    pub fn with_velocity(mut self, velocity: f32) -> Self {
        self.velocity = velocity;
        self.settled = velocity.abs() < self.config.epsilon;
        self
    }

    /// Set initial position (builder-style).
    pub fn with_position(mut self, position: f32) -> Self {
        self.position = position;
        self
    }

    /// Apply a velocity impulse, restarting the animation.
    pub fn kick(&mut self, velocity: f32) {
        self.velocity = velocity;
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

    /// Whether the inertia has settled (velocity below epsilon).
    pub fn is_settled(&self) -> bool {
        self.settled
    }

    /// Reset position and velocity to zero.
    pub fn reset(&mut self) {
        self.position = 0.0;
        self.velocity = 0.0;
        self.settled = true;
    }
}

impl Update for Inertia {
    fn update(&mut self, dt: f32) -> bool {
        if self.settled {
            return false;
        }

        // Frame-rate independent exponential decay:
        // velocity *= (1 - friction)^(dt * 60)
        // This normalises friction to a 60 fps reference.
        let decay = (1.0 - self.config.friction).powf(dt * 60.0);
        self.velocity *= decay;
        self.position += self.velocity * dt;

        if self.velocity.abs() < self.config.epsilon {
            self.velocity = 0.0;
            self.settled = true;
        }

        !self.settled
    }
}

/// Multi-dimensional inertia for any type implementing [`SpringAnimatable`].
///
/// Works the same as [`Inertia`] but decomposes the type into per-axis
/// components and applies friction independently to each.
///
/// # Example
///
/// ```rust
/// use spanda::inertia::{InertiaN, InertiaConfig};
/// use spanda::traits::Update;
///
/// let mut inertia = InertiaN::new(InertiaConfig::default_flick(), [0.0_f32, 0.0])
///     .with_velocity([300.0, -200.0]);
///
/// for _ in 0..300 {
///     if !inertia.update(1.0 / 60.0) { break; }
/// }
/// assert!(inertia.is_settled());
/// ```
#[derive(Clone, Debug)]
pub struct InertiaN<T: SpringAnimatable> {
    /// Configuration (mutable for live tuning).
    pub config: InertiaConfig,
    velocities: Vec<f32>,
    positions: Vec<f32>,
    current: T,
    settled: bool,
}

impl<T: SpringAnimatable> InertiaN<T> {
    /// Create multi-dimensional inertia starting at the given position.
    pub fn new(config: InertiaConfig, initial: T) -> Self {
        let positions = initial.to_components();
        let n = positions.len();
        Self {
            config,
            velocities: vec![0.0; n],
            positions,
            current: initial,
            settled: true,
        }
    }

    /// Set initial velocity decomposed into the same components as position.
    pub fn with_velocity(mut self, velocity: T) -> Self {
        self.velocities = velocity.to_components();
        self.settled = self
            .velocities
            .iter()
            .all(|&v: &f32| v.abs() < self.config.epsilon);
        self
    }

    /// Apply a velocity impulse, restarting the animation.
    pub fn kick(&mut self, velocity: T) {
        self.velocities = velocity.to_components();
        self.settled = false;
    }

    /// Current position as the animated type.
    pub fn position(&self) -> T {
        self.current.clone()
    }

    /// Raw velocity components.
    pub fn velocity_components(&self) -> &[f32] {
        &self.velocities
    }

    /// Whether all axes have settled.
    pub fn is_settled(&self) -> bool {
        self.settled
    }

    /// Reset everything to zero.
    pub fn reset(&mut self, initial: T) {
        self.positions = initial.to_components();
        self.velocities = vec![0.0; self.positions.len()];
        self.current = initial;
        self.settled = true;
    }
}

impl<T: SpringAnimatable> Update for InertiaN<T> {
    fn update(&mut self, dt: f32) -> bool {
        if self.settled {
            return false;
        }

        let decay = (1.0 - self.config.friction).powf(dt * 60.0);
        let mut all_settled = true;

        for i in 0..self.velocities.len() {
            self.velocities[i] *= decay;
            self.positions[i] += self.velocities[i] * dt;

            if self.velocities[i].abs() >= self.config.epsilon {
                all_settled = false;
            } else {
                self.velocities[i] = 0.0;
            }
        }

        self.current = T::from_components(&self.positions);

        if all_settled {
            self.settled = true;
        }

        !self.settled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inertia_decelerates_to_zero() {
        let mut inertia = Inertia::new(InertiaConfig::default_flick()).with_velocity(500.0);

        for _ in 0..600 {
            if !inertia.update(1.0 / 60.0) {
                break;
            }
        }
        assert!(inertia.is_settled());
        assert!(inertia.velocity().abs() < 0.2);
    }

    #[test]
    fn inertia_position_increases_for_positive_velocity() {
        let mut inertia = Inertia::new(InertiaConfig::default_flick()).with_velocity(100.0);

        let prev_pos = inertia.position();
        inertia.update(1.0 / 60.0);
        assert!(inertia.position() > prev_pos);
    }

    #[test]
    fn inertia_zero_velocity_is_settled() {
        let inertia = Inertia::new(InertiaConfig::default_flick()).with_velocity(0.0);
        assert!(inertia.is_settled());
    }

    #[test]
    fn inertia_kick_restarts() {
        let mut inertia = Inertia::new(InertiaConfig::default_flick()).with_velocity(100.0);

        // Let it settle
        for _ in 0..600 {
            if !inertia.update(1.0 / 60.0) {
                break;
            }
        }
        assert!(inertia.is_settled());

        // Kick it
        inertia.kick(200.0);
        assert!(!inertia.is_settled());
        assert!(inertia.update(1.0 / 60.0));
    }

    #[test]
    fn inertia_snappy_stops_faster_than_heavy() {
        let mut snappy = Inertia::new(InertiaConfig::snappy())
            .with_velocity(500.0)
            .with_position(0.0);
        let mut heavy = Inertia::new(InertiaConfig::heavy())
            .with_velocity(500.0)
            .with_position(0.0);

        let mut snappy_frames = 0u32;
        for _ in 0..10000 {
            snappy_frames += 1;
            if !snappy.update(1.0 / 60.0) {
                break;
            }
        }

        let mut heavy_frames = 0u32;
        for _ in 0..10000 {
            heavy_frames += 1;
            if !heavy.update(1.0 / 60.0) {
                break;
            }
        }

        assert!(
            snappy_frames < heavy_frames,
            "snappy ({snappy_frames}) should stop before heavy ({heavy_frames})"
        );
    }

    #[test]
    fn inertia_n_2d_decelerates() {
        let mut inertia = InertiaN::new(InertiaConfig::default_flick(), [0.0_f32, 0.0])
            .with_velocity([300.0, -200.0]);

        for _ in 0..600 {
            if !inertia.update(1.0 / 60.0) {
                break;
            }
        }
        assert!(inertia.is_settled());
        let vel = inertia.velocity_components();
        assert!(vel[0].abs() < 0.2);
        assert!(vel[1].abs() < 0.2);
    }

    #[test]
    fn inertia_n_position_changes() {
        let mut inertia = InertiaN::new(InertiaConfig::default_flick(), [0.0_f32, 0.0])
            .with_velocity([100.0, 0.0]);

        inertia.update(1.0 / 60.0);
        let pos = inertia.position();
        assert!(pos[0] > 0.0, "x should have moved: {:?}", pos);
        assert!((pos[1]).abs() < 1e-6, "y should be ~0: {:?}", pos);
    }

    #[test]
    fn inertia_reset_works() {
        let mut inertia = Inertia::new(InertiaConfig::default_flick())
            .with_velocity(100.0)
            .with_position(50.0);
        inertia.update(0.1);
        inertia.reset();
        assert!(inertia.is_settled());
        assert!((inertia.position()).abs() < 1e-6);
        assert!((inertia.velocity()).abs() < 1e-6);
    }

    #[test]
    fn inertia_frame_rate_independence() {
        // Two inertia instances: one ticked at 60fps, one at 120fps
        // should end up at roughly the same position
        let mut a = Inertia::new(InertiaConfig::default_flick()).with_velocity(500.0);
        let mut b = Inertia::new(InertiaConfig::default_flick()).with_velocity(500.0);

        // 1 second at 60 fps
        for _ in 0..60 {
            a.update(1.0 / 60.0);
        }
        // 1 second at 120 fps
        for _ in 0..120 {
            b.update(1.0 / 120.0);
        }

        let diff = (a.position() - b.position()).abs();
        assert!(
            diff < 5.0,
            "Frame rate independence: 60fps pos={}, 120fps pos={}, diff={}",
            a.position(),
            b.position(),
            diff
        );
    }
}
