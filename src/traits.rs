//! Core traits that power the entire spanda animation system.
//!
//! The central idea is simple: any type that can be *linearly interpolated*
//! can be animated.  Implement [`Interpolate`] on your type and you get
//! tweening, keyframes, and timeline support for free.
//!
//! # The trait hierarchy
//!
//! ```text
//!  Interpolate          — lerp between two values given t ∈ [0.0, 1.0]
//!      └── Animatable   — Interpolate + Clone + 'static (object-safe bound)
//!              └── (blanket impls for f32, f64, [f32;2], [f32;3], [f32;4])
//!
//!  Update               — advance an animation by a delta-time step
//! ```

#[cfg(not(feature = "std"))]
#[allow(unused_imports)]
use num_traits::Float as _;

// ── Interpolate ──────────────────────────────────────────────────────────────

/// Linear interpolation between two values.
///
/// `t` is a normalised progress value in **[0.0, 1.0]**.
/// Implementations *may* extrapolate outside that range but are not required to.
///
/// # Example — custom 2-D point
/// ```
/// use spanda::traits::Interpolate;
///
/// #[derive(Clone)]
/// struct Point { x: f32, y: f32 }
///
/// impl Interpolate for Point {
///     fn lerp(&self, other: &Self, t: f32) -> Self {
///         Point {
///             x: self.x + (other.x - self.x) * t,
///             y: self.y + (other.y - self.y) * t,
///         }
///     }
/// }
/// ```
pub trait Interpolate: Sized {
    /// Return the value that is `t` of the way from `self` to `other`.
    fn lerp(&self, other: &Self, t: f32) -> Self;
}

// ── Animatable ───────────────────────────────────────────────────────────────

/// A value that can be animated — combines [`Interpolate`] with the bounds
/// needed to store animations generically (`Clone + 'static`).
///
/// You never need to implement this manually; it is provided automatically for
/// every type that implements `Interpolate + Clone + 'static`.
pub trait Animatable: Interpolate + Clone + 'static {}

impl<T: Interpolate + Clone + 'static> Animatable for T {}

// ── Update ───────────────────────────────────────────────────────────────────

/// Advance an animation by `dt` seconds.
///
/// Implemented by [`crate::tween::Tween`], [`crate::timeline::Timeline`],
/// and the Bevy / WASM drivers.
pub trait Update {
    /// Step the animation forward by `dt` seconds.
    ///
    /// Returns `true` while the animation is still running, `false` once it
    /// has completed (so a driver can clean it up automatically).
    fn update(&mut self, dt: f32) -> bool;
}

#[cfg(not(feature = "std"))]
use alloc::boxed::Box;

// Blanket impl for boxed updates
impl<T: Update + ?Sized> Update for Box<T> {
    fn update(&mut self, dt: f32) -> bool {
        (**self).update(dt)
    }
}

// ── Blanket Interpolate impls ─────────────────────────────────────────────────

impl Interpolate for f32 {
    #[inline]
    fn lerp(&self, other: &Self, t: f32) -> Self {
        self + (other - self) * t
    }
}

impl Interpolate for f64 {
    #[inline]
    fn lerp(&self, other: &Self, t: f32) -> Self {
        // f64 keeps its own precision; cast t to f64 internally
        self + (other - self) * (t as f64)
    }
}

/// 2-D vector: `[x, y]`
impl Interpolate for [f32; 2] {
    #[inline]
    fn lerp(&self, other: &Self, t: f32) -> Self {
        [self[0].lerp(&other[0], t), self[1].lerp(&other[1], t)]
    }
}

/// 3-D vector: `[x, y, z]`
impl Interpolate for [f32; 3] {
    #[inline]
    fn lerp(&self, other: &Self, t: f32) -> Self {
        [
            self[0].lerp(&other[0], t),
            self[1].lerp(&other[1], t),
            self[2].lerp(&other[2], t),
        ]
    }
}

/// RGBA colour: `[r, g, b, a]` — all channels in `0.0..=1.0`
impl Interpolate for [f32; 4] {
    #[inline]
    fn lerp(&self, other: &Self, t: f32) -> Self {
        [
            self[0].lerp(&other[0], t),
            self[1].lerp(&other[1], t),
            self[2].lerp(&other[2], t),
            self[3].lerp(&other[3], t),
        ]
    }
}

/// Scalar integer — rounds to nearest after interpolation.
impl Interpolate for i32 {
    #[inline]
    fn lerp(&self, other: &Self, t: f32) -> Self {
        (*self as f32).lerp(&(*other as f32), t).round() as i32
    }
}

// ── f64's lerp needs a special note ──────────────────────────────────────────
// The blanket impl above calls `self + (other - self) * (t as f64)`.
// This is intentional: the `Interpolate` contract uses `f32` for `t` across
// the whole crate so callers don't have to match precision to the value type.

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f32_lerp_midpoint() {
        let a = 0.0_f32;
        let b = 10.0_f32;
        assert!((a.lerp(&b, 0.5) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn f32_lerp_endpoints() {
        let a = 3.0_f32;
        let b = 7.0_f32;
        assert!((a.lerp(&b, 0.0) - 3.0).abs() < 1e-6);
        assert!((a.lerp(&b, 1.0) - 7.0).abs() < 1e-6);
    }

    #[test]
    fn vec2_lerp() {
        let a = [0.0_f32, 0.0];
        let b = [4.0_f32, 8.0];
        let mid = a.lerp(&b, 0.5);
        assert!((mid[0] - 2.0).abs() < 1e-6);
        assert!((mid[1] - 4.0).abs() < 1e-6);
    }

    #[test]
    fn rgba_lerp_alpha() {
        let transparent = [1.0_f32, 0.0, 0.0, 0.0];
        let opaque = [1.0_f32, 0.0, 0.0, 1.0];
        let half = transparent.lerp(&opaque, 0.5);
        assert!((half[3] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn i32_lerp_rounds() {
        assert_eq!(0_i32.lerp(&10, 0.34), 3);
        assert_eq!(0_i32.lerp(&10, 0.35), 4); // rounds to nearest
    }

    #[test]
    fn animatable_is_auto_impl() {
        fn needs_animatable<T: Animatable>(_: T) {}
        needs_animatable(1.0_f32);
        needs_animatable([0.0_f32; 4]);
    }
}
