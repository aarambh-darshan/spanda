//! Catmull-Rom spline — smooth curve through a sequence of points.
//!
//! A Catmull-Rom spline automatically generates smooth control points so the
//! curve passes **through** every given point (unlike raw Bezier, where control
//! points pull the curve but don't lie on it).
//!
//! The `tension` parameter controls curviness:
//! - `0.0` → straight lines between points
//! - `0.5` → standard Catmull-Rom (default)
//! - `1.0` → maximum curvature
//! - `>1.0` → exaggerated curvature
//!
//! # Example
//!
//! ```rust
//! use spanda::bezier::{CatmullRomSpline, PathEvaluate2D};
//!
//! let spline = CatmullRomSpline::new(vec![
//!     [0.0, 0.0],
//!     [100.0, 50.0],
//!     [200.0, 0.0],
//!     [300.0, 50.0],
//! ]);
//!
//! let mid = spline.evaluate([0.0, 0.0], 0.5);
//! ```

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

// ── PathEvaluate2D ──────────────────────────────────────────────────────────

/// Evaluate a 2D curve at progress `t ∈ [0.0, 1.0]`.
///
/// Takes a `default` value used when the path has no points.
pub trait PathEvaluate2D {
    /// Evaluate position on the path at progress `t`.
    fn evaluate(&self, default: [f32; 2], t: f32) -> [f32; 2];

    /// Evaluate the tangent (direction vector) at progress `t`.
    fn tangent(&self, default: [f32; 2], t: f32) -> [f32; 2];
}

// ── CatmullRomSpline ────────────────────────────────────────────────────────

/// A Catmull-Rom spline through an ordered list of 2D points.
///
/// Converts internally to cubic Bezier segments for evaluation.
/// The `tension` parameter (default 0.5) controls curviness — equivalent
/// to GSAP's `curviness` property divided by 3.
#[derive(Debug, Clone)]
pub struct CatmullRomSpline {
    /// The original knot points.
    points: Vec<[f32; 2]>,
    /// Tension parameter (0.0 = straight, 0.5 = standard, >1.0 = exaggerated).
    tension: f32,
}

impl CatmullRomSpline {
    /// Create a spline through the given points with default tension (0.5).
    pub fn new(points: Vec<[f32; 2]>) -> Self {
        Self {
            points,
            tension: 0.5,
        }
    }

    /// Set the tension / curviness.
    ///
    /// `0.0` = straight lines, `0.5` = standard Catmull-Rom, `>1.0` = exaggerated.
    pub fn tension(mut self, t: f32) -> Self {
        self.tension = t.max(0.0);
        self
    }

    /// Number of knot points.
    pub fn point_count(&self) -> usize {
        self.points.len()
    }

    /// Number of spline segments (one fewer than points, minimum 0).
    pub fn segment_count(&self) -> usize {
        if self.points.len() < 2 {
            0
        } else {
            self.points.len() - 1
        }
    }

    /// Get a reference to the knot points.
    pub fn points(&self) -> &[[f32; 2]] {
        &self.points
    }

    /// Compute the cubic Bezier control points for segment `i` (between
    /// `points[i]` and `points[i+1]`).
    ///
    /// Returns `(cp1, cp2)` — the two control points of the cubic Bezier
    /// segment from `points[i]` to `points[i+1]`.
    fn segment_control_points(&self, i: usize) -> ([f32; 2], [f32; 2]) {
        let n = self.points.len();
        assert!(i + 1 < n);

        let p0 = if i == 0 {
            self.points[0]
        } else {
            self.points[i - 1]
        };
        let p1 = self.points[i];
        let p2 = self.points[i + 1];
        let p3 = if i + 2 < n {
            self.points[i + 2]
        } else {
            self.points[n - 1]
        };

        let alpha = self.tension;

        // Catmull-Rom to cubic Bezier conversion:
        // cp1 = p1 + (p2 - p0) * alpha / 3
        // cp2 = p2 - (p3 - p1) * alpha / 3
        let cp1 = [
            p1[0] + (p2[0] - p0[0]) * alpha / 3.0,
            p1[1] + (p2[1] - p0[1]) * alpha / 3.0,
        ];
        let cp2 = [
            p2[0] - (p3[0] - p1[0]) * alpha / 3.0,
            p2[1] - (p3[1] - p1[1]) * alpha / 3.0,
        ];

        (cp1, cp2)
    }

    /// Evaluate a single cubic Bezier segment at local `t`.
    fn eval_cubic(p0: [f32; 2], cp1: [f32; 2], cp2: [f32; 2], p3: [f32; 2], t: f32) -> [f32; 2] {
        let inv = 1.0 - t;
        let inv2 = inv * inv;
        let inv3 = inv2 * inv;
        let t2 = t * t;
        let t3 = t2 * t;
        [
            inv3 * p0[0] + 3.0 * inv2 * t * cp1[0] + 3.0 * inv * t2 * cp2[0] + t3 * p3[0],
            inv3 * p0[1] + 3.0 * inv2 * t * cp1[1] + 3.0 * inv * t2 * cp2[1] + t3 * p3[1],
        ]
    }

    /// Derivative of a cubic Bezier segment at local `t`.
    fn eval_cubic_derivative(
        p0: [f32; 2],
        cp1: [f32; 2],
        cp2: [f32; 2],
        p3: [f32; 2],
        t: f32,
    ) -> [f32; 2] {
        let inv = 1.0 - t;
        let inv2 = inv * inv;
        let t2 = t * t;
        [
            3.0 * inv2 * (cp1[0] - p0[0])
                + 6.0 * inv * t * (cp2[0] - cp1[0])
                + 3.0 * t2 * (p3[0] - cp2[0]),
            3.0 * inv2 * (cp1[1] - p0[1])
                + 6.0 * inv * t * (cp2[1] - cp1[1])
                + 3.0 * t2 * (p3[1] - cp2[1]),
        ]
    }

    /// Map global `t ∈ [0, 1]` to `(segment_index, local_t)`.
    fn map_t(&self, t: f32) -> (usize, f32) {
        let seg_count = self.segment_count();
        if seg_count == 0 {
            return (0, 0.0);
        }
        let t = t.clamp(0.0, 1.0);
        let scaled = t * seg_count as f32;
        let idx = (scaled.floor() as usize).min(seg_count - 1);
        let local = scaled - idx as f32;
        (idx, local.clamp(0.0, 1.0))
    }
}

impl PathEvaluate2D for CatmullRomSpline {
    fn evaluate(&self, default: [f32; 2], t: f32) -> [f32; 2] {
        match self.points.len() {
            0 => default,
            1 => self.points[0],
            _ => {
                let (idx, local_t) = self.map_t(t);
                let (cp1, cp2) = self.segment_control_points(idx);
                Self::eval_cubic(self.points[idx], cp1, cp2, self.points[idx + 1], local_t)
            }
        }
    }

    fn tangent(&self, default: [f32; 2], t: f32) -> [f32; 2] {
        match self.points.len() {
            0 | 1 => default,
            _ => {
                let (idx, local_t) = self.map_t(t);
                let (cp1, cp2) = self.segment_control_points(idx);
                Self::eval_cubic_derivative(
                    self.points[idx],
                    cp1,
                    cp2,
                    self.points[idx + 1],
                    local_t,
                )
            }
        }
    }
}

// ── Tangent angle helper ────────────────────────────────────────────────────

/// Compute the rotation angle (in radians) from a tangent vector.
///
/// Returns the angle of the tangent relative to the positive X axis,
/// using `atan2(y, x)`. Equivalent to GSAP's `autoRotate: true`.
pub fn tangent_angle(tangent: [f32; 2]) -> f32 {
    tangent[1].atan2(tangent[0])
}

/// Compute the rotation angle in degrees from a tangent vector.
pub fn tangent_angle_deg(tangent: [f32; 2]) -> f32 {
    tangent_angle(tangent).to_degrees()
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catmull_rom_endpoints() {
        let spline = CatmullRomSpline::new(vec![
            [0.0, 0.0],
            [100.0, 50.0],
            [200.0, 0.0],
        ]);

        let start = spline.evaluate([0.0, 0.0], 0.0);
        let end = spline.evaluate([0.0, 0.0], 1.0);

        assert!((start[0]).abs() < 1e-4, "Expected start x~0, got {}", start[0]);
        assert!((start[1]).abs() < 1e-4, "Expected start y~0, got {}", start[1]);
        assert!((end[0] - 200.0).abs() < 1e-4, "Expected end x~200, got {}", end[0]);
        assert!((end[1]).abs() < 1e-4, "Expected end y~0, got {}", end[1]);
    }

    #[test]
    fn catmull_rom_passes_through_midpoint() {
        let spline = CatmullRomSpline::new(vec![
            [0.0, 0.0],
            [100.0, 100.0],
            [200.0, 0.0],
        ]);

        // At t=0.5, the spline should pass through [100, 100] (the middle knot)
        let mid = spline.evaluate([0.0, 0.0], 0.5);
        assert!(
            (mid[0] - 100.0).abs() < 1e-3,
            "Expected x~100 at t=0.5, got {}",
            mid[0]
        );
        assert!(
            (mid[1] - 100.0).abs() < 1e-3,
            "Expected y~100 at t=0.5, got {}",
            mid[1]
        );
    }

    #[test]
    fn catmull_rom_four_points() {
        let spline = CatmullRomSpline::new(vec![
            [0.0, 0.0],
            [100.0, 50.0],
            [200.0, 0.0],
            [300.0, 50.0],
        ]);

        let start = spline.evaluate([0.0, 0.0], 0.0);
        let end = spline.evaluate([0.0, 0.0], 1.0);
        assert!((start[0]).abs() < 1e-4);
        assert!((end[0] - 300.0).abs() < 1e-4);

        // At the knot subdivisions
        let p1 = spline.evaluate([0.0, 0.0], 1.0 / 3.0);
        assert!(
            (p1[0] - 100.0).abs() < 1e-3,
            "Expected x~100 at t=1/3, got {}",
            p1[0]
        );
    }

    #[test]
    fn catmull_rom_tension_zero_is_straight() {
        let spline = CatmullRomSpline::new(vec![
            [0.0, 0.0],
            [100.0, 100.0],
            [200.0, 0.0],
        ]).tension(0.0);

        // With tension=0, segments should be straight lines
        // At t=0.25, should be at midpoint of first segment: [50, 50]
        let quarter = spline.evaluate([0.0, 0.0], 0.25);
        assert!(
            (quarter[0] - 50.0).abs() < 1e-3,
            "Expected x~50, got {}",
            quarter[0]
        );
        assert!(
            (quarter[1] - 50.0).abs() < 1e-3,
            "Expected y~50, got {}",
            quarter[1]
        );
    }

    #[test]
    fn catmull_rom_high_tension_overshoots() {
        let normal = CatmullRomSpline::new(vec![
            [0.0, 0.0],
            [100.0, 100.0],
            [200.0, 0.0],
        ]).tension(0.5);

        let high = CatmullRomSpline::new(vec![
            [0.0, 0.0],
            [100.0, 100.0],
            [200.0, 0.0],
        ]).tension(1.5);

        // With higher tension, the curve should deviate more from straight lines
        let normal_pt = normal.evaluate([0.0, 0.0], 0.25);
        let high_pt = high.evaluate([0.0, 0.0], 0.25);

        // They shouldn't be identical (tension changes the control points)
        let diff = (normal_pt[1] - high_pt[1]).abs();
        assert!(diff > 0.1, "High tension should differ, got diff={diff}");
    }

    #[test]
    fn catmull_rom_single_point() {
        let spline = CatmullRomSpline::new(vec![[42.0, 17.0]]);
        let val = spline.evaluate([0.0, 0.0], 0.5);
        assert!((val[0] - 42.0).abs() < 1e-6);
        assert!((val[1] - 17.0).abs() < 1e-6);
    }

    #[test]
    fn catmull_rom_empty() {
        let spline = CatmullRomSpline::new(vec![]);
        let val = spline.evaluate([99.0, 99.0], 0.5);
        assert!((val[0] - 99.0).abs() < 1e-6); // returns default
    }

    #[test]
    fn tangent_at_midpoint() {
        let spline = CatmullRomSpline::new(vec![
            [0.0, 0.0],
            [100.0, 0.0],
            [200.0, 0.0],
        ]);

        let tan = spline.tangent([0.0, 0.0], 0.5);
        // On a straight horizontal path, tangent should be roughly [+, 0]
        assert!(tan[0] > 0.0, "Expected positive x tangent, got {}", tan[0]);
        assert!((tan[1]).abs() < 1.0, "Expected y tangent ~0, got {}", tan[1]);
    }

    #[test]
    fn tangent_angle_horizontal() {
        let angle = tangent_angle([1.0, 0.0]);
        assert!((angle).abs() < 1e-6, "Expected 0 radians, got {angle}");
    }

    #[test]
    fn tangent_angle_vertical() {
        let angle = tangent_angle_deg([0.0, 1.0]);
        assert!((angle - 90.0).abs() < 1e-4, "Expected 90 degrees, got {angle}");
    }
}
