//! Bezier paths and motion path interpolation.
//!
//! This module provides quadratic and cubic Bezier curves, plus a
//! [`MotionPath`] type that composes multiple curve segments into a single
//! animatable path.
//!
//! # Example — cubic Bezier
//!
//! ```rust
//! use spanda::path::{BezierPath, PathEvaluate};
//!
//! let curve = BezierPath::cubic(
//!     [0.0, 0.0],   // start
//!     [0.0, 100.0],  // control 1
//!     [100.0, 100.0],// control 2
//!     [100.0, 0.0],  // end
//! );
//!
//! let mid = curve.evaluate(0.5);
//! assert!(mid[0] > 0.0 && mid[1] > 0.0);
//! ```
//!
//! # Example — motion path with multiple segments
//!
//! ```rust
//! use spanda::path::{BezierPath, MotionPath, PathEvaluate};
//!
//! let path = MotionPath::new()
//!     .cubic(
//!         [0.0, 0.0],
//!         [50.0, 100.0],
//!         [100.0, 100.0],
//!         [150.0, 0.0],
//!     )
//!     .line([150.0, 0.0], [200.0, 0.0]);
//!
//! // Evaluate at any progress 0.0..=1.0
//! let point = path.evaluate(0.5);
//! ```

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::traits::Interpolate;

// ── PathEvaluate trait ──────────────────────────────────────────────────────

/// A curve that can be evaluated at any progress `t ∈ [0.0, 1.0]`.
pub trait PathEvaluate<T> {
    /// Evaluate the path at progress `t`.
    ///
    /// `t = 0.0` returns the start, `t = 1.0` returns the end.
    fn evaluate(&self, t: f32) -> T;
}

// ── BezierPath ──────────────────────────────────────────────────────────────

/// A single Bezier curve segment — linear, quadratic, or cubic.
#[derive(Debug, Clone)]
pub enum BezierPath<T: Clone> {
    /// Straight line from `start` to `end`.
    Linear {
        /// Starting point.
        start: T,
        /// Ending point.
        end: T,
    },
    /// Quadratic Bezier with one control point.
    Quadratic {
        /// Starting point.
        start: T,
        /// Control point.
        control: T,
        /// Ending point.
        end: T,
    },
    /// Cubic Bezier with two control points.
    Cubic {
        /// Starting point.
        start: T,
        /// First control point.
        control1: T,
        /// Second control point.
        control2: T,
        /// Ending point.
        end: T,
    },
}

impl<T: Clone> BezierPath<T> {
    /// Create a linear (straight-line) path.
    pub fn linear(start: T, end: T) -> Self {
        Self::Linear { start, end }
    }

    /// Create a quadratic Bezier path.
    pub fn quadratic(start: T, control: T, end: T) -> Self {
        Self::Quadratic {
            start,
            control,
            end,
        }
    }

    /// Create a cubic Bezier path.
    pub fn cubic(start: T, control1: T, control2: T, end: T) -> Self {
        Self::Cubic {
            start,
            control1,
            control2,
            end,
        }
    }
}

impl<T: Interpolate + Clone> PathEvaluate<T> for BezierPath<T> {
    fn evaluate(&self, t: f32) -> T {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Linear { start, end } => start.lerp(end, t),
            Self::Quadratic {
                start,
                control,
                end,
            } => {
                // B(t) = (1-t)²·P0 + 2(1-t)t·P1 + t²·P2
                let a = start.lerp(control, t);
                let b = control.lerp(end, t);
                a.lerp(&b, t)
            }
            Self::Cubic {
                start,
                control1,
                control2,
                end,
            } => {
                // De Casteljau's algorithm for cubic:
                // Level 1
                let a = start.lerp(control1, t);
                let b = control1.lerp(control2, t);
                let c = control2.lerp(end, t);
                // Level 2
                let d = a.lerp(&b, t);
                let e = b.lerp(&c, t);
                // Level 3
                d.lerp(&e, t)
            }
        }
    }
}

// ── MotionPath ──────────────────────────────────────────────────────────────

/// A multi-segment path composed of Bezier curves.
///
/// Each segment has an associated "weight" (defaulting to 1.0) that
/// determines what fraction of the overall `t ∈ [0.0, 1.0]` range it
/// occupies. This allows segments of different visual lengths to
/// receive proportional time.
///
/// When evaluated at `t`, the `MotionPath` selects the active segment
/// and maps `t` into the segment's local range.
pub struct MotionPath<T: Clone> {
    segments: Vec<(BezierPath<T>, f32)>, // (curve, weight)
}

impl<T: Clone + core::fmt::Debug> core::fmt::Debug for MotionPath<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MotionPath")
            .field("segments", &self.segments.len())
            .finish()
    }
}

impl<T: Clone> MotionPath<T> {
    /// Create an empty motion path.
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    /// Append a linear (straight-line) segment with weight 1.0.
    pub fn line(mut self, start: T, end: T) -> Self {
        self.segments.push((BezierPath::linear(start, end), 1.0));
        self
    }

    /// Append a quadratic Bezier segment with weight 1.0.
    pub fn quadratic(mut self, start: T, control: T, end: T) -> Self {
        self.segments
            .push((BezierPath::quadratic(start, control, end), 1.0));
        self
    }

    /// Append a cubic Bezier segment with weight 1.0.
    pub fn cubic(mut self, start: T, control1: T, control2: T, end: T) -> Self {
        self.segments
            .push((BezierPath::cubic(start, control1, control2, end), 1.0));
        self
    }

    /// Append a linear segment with a custom weight.
    pub fn line_weighted(mut self, start: T, end: T, weight: f32) -> Self {
        self.segments
            .push((BezierPath::linear(start, end), weight.max(0.0)));
        self
    }

    /// Append a quadratic Bezier segment with a custom weight.
    pub fn quadratic_weighted(
        mut self,
        start: T,
        control: T,
        end: T,
        weight: f32,
    ) -> Self {
        self.segments
            .push((BezierPath::quadratic(start, control, end), weight.max(0.0)));
        self
    }

    /// Append a cubic Bezier segment with a custom weight.
    pub fn cubic_weighted(
        mut self,
        start: T,
        control1: T,
        control2: T,
        end: T,
        weight: f32,
    ) -> Self {
        self.segments.push((
            BezierPath::cubic(start, control1, control2, end),
            weight.max(0.0),
        ));
        self
    }

    /// Append a raw [`BezierPath`] segment with weight 1.0.
    pub fn segment(mut self, path: BezierPath<T>) -> Self {
        self.segments.push((path, 1.0));
        self
    }

    /// Append a raw [`BezierPath`] segment with a custom weight.
    pub fn segment_weighted(mut self, path: BezierPath<T>, weight: f32) -> Self {
        self.segments.push((path, weight.max(0.0)));
        self
    }

    /// Number of segments in this path.
    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }

    /// Total weight (sum of all segment weights).
    fn total_weight(&self) -> f32 {
        self.segments.iter().map(|(_, w)| w).sum()
    }
}

impl<T: Clone> Default for MotionPath<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Interpolate + Clone> PathEvaluate<T> for MotionPath<T> {
    fn evaluate(&self, t: f32) -> T {
        let t = t.clamp(0.0, 1.0);

        if self.segments.is_empty() {
            panic!("MotionPath::evaluate called on empty path");
        }

        if self.segments.len() == 1 {
            return self.segments[0].0.evaluate(t);
        }

        let total = self.total_weight();
        if total <= 0.0 {
            return self.segments[0].0.evaluate(0.0);
        }

        // Map global t to the correct segment
        let target = t * total;
        let mut accumulated = 0.0_f32;

        for (curve, weight) in &self.segments {
            if accumulated + weight >= target || (target - (accumulated + weight)).abs() < 1e-10 {
                // This is the active segment
                let local_t = if *weight <= 0.0 {
                    0.0
                } else {
                    ((target - accumulated) / weight).clamp(0.0, 1.0)
                };
                return curve.evaluate(local_t);
            }
            accumulated += weight;
        }

        // Fallback: evaluate end of last segment
        self.segments.last().unwrap().0.evaluate(1.0)
    }
}

// ── MotionPathTween (convenience) ───────────────────────────────────────────

/// A tween-like animation that moves a value along a [`MotionPath`].
///
/// Implements [`Update`](crate::traits::Update) so it can be used
/// with timelines, drivers, and sequences just like a regular Tween.
///
/// ```rust
/// use spanda::path::{MotionPath, MotionPathTween};
/// use spanda::traits::Update;
/// use spanda::easing::Easing;
///
/// let path = MotionPath::new()
///     .line([0.0_f32, 0.0], [100.0, 0.0])
///     .line([100.0, 0.0], [100.0, 100.0]);
///
/// let mut tween = MotionPathTween::new(path)
///     .duration(2.0)
///     .easing(Easing::EaseInOutCubic);
///
/// tween.update(1.0); // 50% — at corner
/// let pos = tween.value();
/// ```
pub struct MotionPathTween<T: Interpolate + Clone> {
    path: MotionPath<T>,
    duration: f32,
    easing: crate::easing::Easing,
    elapsed: f32,
    completed: bool,
}

impl<T: Interpolate + Clone + core::fmt::Debug> core::fmt::Debug for MotionPathTween<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MotionPathTween")
            .field("path", &self.path)
            .field("duration", &self.duration)
            .field("easing", &self.easing)
            .field("elapsed", &self.elapsed)
            .field("completed", &self.completed)
            .finish()
    }
}

impl<T: Interpolate + Clone> MotionPathTween<T> {
    /// Create a new tween along the given path.
    pub fn new(path: MotionPath<T>) -> Self {
        Self {
            path,
            duration: 1.0,
            easing: crate::easing::Easing::Linear,
            elapsed: 0.0,
            completed: false,
        }
    }

    /// Set the duration in seconds (default: 1.0).
    pub fn duration(mut self, d: f32) -> Self {
        self.duration = d.max(0.0);
        self
    }

    /// Set the easing curve (default: Linear).
    pub fn easing(mut self, e: crate::easing::Easing) -> Self {
        self.easing = e;
        self
    }

    /// Current position on the path.
    pub fn value(&self) -> T {
        let raw_t = if self.duration <= 0.0 {
            1.0
        } else {
            (self.elapsed / self.duration).clamp(0.0, 1.0)
        };
        let curved_t = self.easing.apply(raw_t);
        self.path.evaluate(curved_t)
    }

    /// Raw progress in `0.0..=1.0`.
    pub fn progress(&self) -> f32 {
        if self.duration <= 0.0 {
            1.0
        } else {
            (self.elapsed / self.duration).clamp(0.0, 1.0)
        }
    }

    /// Whether the animation has completed.
    pub fn is_complete(&self) -> bool {
        self.completed
    }

    /// Reset to the beginning.
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
        self.completed = false;
    }
}

impl<T: Interpolate + Clone + 'static> crate::traits::Update for MotionPathTween<T> {
    fn update(&mut self, dt: f32) -> bool {
        if self.completed {
            return false;
        }

        let dt = dt.max(0.0);
        self.elapsed += dt;

        if self.elapsed >= self.duration {
            self.elapsed = self.duration;
            self.completed = true;
            return false;
        }

        true
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── BezierPath tests ───────────────────────────────────────────────────

    #[test]
    fn linear_bezier_endpoints() {
        let path = BezierPath::linear([0.0_f32, 0.0], [100.0, 100.0]);
        let start = path.evaluate(0.0);
        let end = path.evaluate(1.0);
        assert!((start[0]).abs() < 1e-6);
        assert!((start[1]).abs() < 1e-6);
        assert!((end[0] - 100.0).abs() < 1e-6);
        assert!((end[1] - 100.0).abs() < 1e-6);
    }

    #[test]
    fn linear_bezier_midpoint() {
        let path = BezierPath::linear([0.0_f32, 0.0], [100.0, 200.0]);
        let mid = path.evaluate(0.5);
        assert!((mid[0] - 50.0).abs() < 1e-4);
        assert!((mid[1] - 100.0).abs() < 1e-4);
    }

    #[test]
    fn quadratic_bezier_endpoints() {
        let path = BezierPath::quadratic([0.0_f32, 0.0], [50.0, 100.0], [100.0, 0.0]);
        let start = path.evaluate(0.0);
        let end = path.evaluate(1.0);
        assert!((start[0]).abs() < 1e-6);
        assert!((end[0] - 100.0).abs() < 1e-6);
        assert!((end[1]).abs() < 1e-6);
    }

    #[test]
    fn quadratic_bezier_peaks_above() {
        // Control point at y=100, start and end at y=0
        let path = BezierPath::quadratic([0.0_f32, 0.0], [50.0, 100.0], [100.0, 0.0]);
        let mid = path.evaluate(0.5);
        // Quadratic midpoint: (1-t)²·0 + 2(1-t)t·100 + t²·0 = 2·0.25·100 = 50
        assert!(mid[1] > 40.0, "Expected y > 40 at midpoint, got {}", mid[1]);
    }

    #[test]
    fn cubic_bezier_endpoints() {
        let path = BezierPath::cubic(
            [0.0_f32, 0.0],
            [33.0, 100.0],
            [66.0, 100.0],
            [100.0, 0.0],
        );
        let start = path.evaluate(0.0);
        let end = path.evaluate(1.0);
        assert!((start[0]).abs() < 1e-6);
        assert!((end[0] - 100.0).abs() < 1e-6);
    }

    #[test]
    fn cubic_bezier_s_curve() {
        // S-curve: both control points above the line
        let path = BezierPath::cubic(
            [0.0_f32, 0.0],
            [0.0, 100.0],
            [100.0, 100.0],
            [100.0, 0.0],
        );
        let mid = path.evaluate(0.5);
        // At t=0.5, the de Casteljau gives y ≈ 75
        assert!(mid[1] > 50.0, "Expected y > 50 at midpoint, got {}", mid[1]);
    }

    #[test]
    fn bezier_clamps_t() {
        let path = BezierPath::linear(0.0_f32, 100.0);
        assert!((path.evaluate(-1.0) - 0.0).abs() < 1e-6);
        assert!((path.evaluate(2.0) - 100.0).abs() < 1e-6);
    }

    // ── MotionPath tests ───────────────────────────────────────────────────

    #[test]
    fn motion_path_single_segment() {
        let path = MotionPath::new().line(0.0_f32, 100.0);
        assert!((path.evaluate(0.0) - 0.0).abs() < 1e-6);
        assert!((path.evaluate(1.0) - 100.0).abs() < 1e-6);
        assert!((path.evaluate(0.5) - 50.0).abs() < 1e-4);
    }

    #[test]
    fn motion_path_two_segments_equal_weight() {
        let path = MotionPath::new()
            .line(0.0_f32, 100.0)
            .line(100.0, 200.0);

        // t=0.0 → start of first segment
        assert!((path.evaluate(0.0) - 0.0).abs() < 1e-6);
        // t=0.5 → end of first / start of second
        assert!((path.evaluate(0.5) - 100.0).abs() < 1e-4);
        // t=1.0 → end of second segment
        assert!((path.evaluate(1.0) - 200.0).abs() < 1e-4);
        // t=0.25 → midpoint of first segment
        assert!((path.evaluate(0.25) - 50.0).abs() < 1e-4);
    }

    #[test]
    fn motion_path_weighted_segments() {
        // First segment weight 3, second weight 1 → first gets 75% of t range
        let path = MotionPath::new()
            .line_weighted(0.0_f32, 300.0, 3.0)
            .line_weighted(300.0, 400.0, 1.0);

        // At t=0.75 → end of first segment
        assert!(
            (path.evaluate(0.75) - 300.0).abs() < 1e-3,
            "Expected 300, got {}",
            path.evaluate(0.75)
        );
    }

    #[test]
    fn motion_path_with_bezier() {
        let path = MotionPath::new().cubic(
            [0.0_f32, 0.0],
            [0.0, 100.0],
            [100.0, 100.0],
            [100.0, 0.0],
        );

        let start = path.evaluate(0.0);
        let end = path.evaluate(1.0);
        assert!((start[0]).abs() < 1e-6);
        assert!((end[0] - 100.0).abs() < 1e-6);
    }

    #[test]
    #[should_panic(expected = "empty path")]
    fn motion_path_empty_panics() {
        let path = MotionPath::<f32>::new();
        let _ = path.evaluate(0.5);
    }

    // ── MotionPathTween tests ──────────────────────────────────────────────

    #[test]
    fn motion_path_tween_basic() {
        use crate::traits::Update;

        let path = MotionPath::new().line([0.0_f32, 0.0], [100.0, 100.0]);

        let mut tween = MotionPathTween::new(path).duration(1.0);
        assert!(!tween.is_complete());

        tween.update(0.5);
        let val = tween.value();
        assert!((val[0] - 50.0).abs() < 1e-4);
        assert!((val[1] - 50.0).abs() < 1e-4);

        tween.update(0.5);
        assert!(tween.is_complete());
        let final_val = tween.value();
        assert!((final_val[0] - 100.0).abs() < 1e-4);
    }

    #[test]
    fn motion_path_tween_with_easing() {
        use crate::easing::Easing;
        use crate::traits::Update;

        let path = MotionPath::new().line(0.0_f32, 100.0);

        let mut tween = MotionPathTween::new(path)
            .duration(1.0)
            .easing(Easing::EaseInQuad);

        tween.update(0.5);
        // EaseInQuad at t=0.5 → 0.25
        assert!(
            (tween.value() - 25.0).abs() < 1e-4,
            "Expected ~25, got {}",
            tween.value()
        );
    }

    #[test]
    fn motion_path_tween_reset() {
        use crate::traits::Update;

        let path = MotionPath::new().line(0.0_f32, 100.0);
        let mut tween = MotionPathTween::new(path).duration(1.0);

        tween.update(1.0);
        assert!(tween.is_complete());

        tween.reset();
        assert!(!tween.is_complete());
        assert!((tween.value() - 0.0).abs() < 1e-6);
    }
}
