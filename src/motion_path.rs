//! Advanced 2D motion path system — GSAP-equivalent path animation.
//!
//! This module provides:
//!
//! - [`PolyPath`] — smooth path through a sequence of `[x, y]` points
//! - [`CompoundPath`] — multi-segment path from Move, Line, Quad, Cubic commands
//! - Arc-length parameterization for constant-speed motion
//! - Tangent angle computation for auto-rotate
//! - Start / end offset for partial path traversal
//!
//! All types output raw `[f32; 2]` values — the caller writes them to SVG,
//! Canvas, Bevy Transform, or any other target.
//!
//! # Example — PolyPath
//!
//! ```rust
//! use spanda::motion_path::PolyPath;
//!
//! let path = PolyPath::from_points(vec![
//!     [0.0, 0.0],
//!     [100.0, 50.0],
//!     [200.0, 0.0],
//! ]);
//! let pos = path.position(0.5);
//! let angle = path.rotation(0.5);
//! ```

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::bezier::{CatmullRomSpline, PathEvaluate2D, tangent_angle};

// ── Arc-length LUT ──────────────────────────────────────────────────────────

/// Number of samples used to build the arc-length lookup table.
const ARC_LEN_SAMPLES: usize = 256;

/// A precomputed lookup table mapping parametric `t` to cumulative arc length,
/// enabling constant-speed traversal of any curve.
#[derive(Debug, Clone)]
struct ArcLengthTable {
    /// `entries[i]` = cumulative arc length at `t = i / (entries.len() - 1)`.
    entries: Vec<f32>,
    /// Total arc length of the curve.
    total_length: f32,
}

impl ArcLengthTable {
    /// Build an arc-length table by sampling the evaluator at `n` points.
    fn build<F: Fn(f32) -> [f32; 2]>(evaluate: F, n: usize) -> Self {
        let mut entries = Vec::with_capacity(n);
        entries.push(0.0);

        let mut prev = evaluate(0.0);
        let mut cumulative = 0.0_f32;

        for i in 1..n {
            let t = i as f32 / (n - 1) as f32;
            let curr = evaluate(t);
            let dx = curr[0] - prev[0];
            let dy = curr[1] - prev[1];
            cumulative += (dx * dx + dy * dy).sqrt();
            entries.push(cumulative);
            prev = curr;
        }

        let total_length = cumulative;
        Self {
            entries,
            total_length,
        }
    }

    /// Map a uniform distance fraction `u ∈ [0, 1]` to the parametric `t`
    /// that produces that fraction of total arc length.
    fn uniform_to_t(&self, u: f32) -> f32 {
        if self.total_length <= 0.0 || self.entries.len() < 2 {
            return u;
        }

        let u = u.clamp(0.0, 1.0);
        let target = u * self.total_length;

        // Binary search for the interval containing target
        let n = self.entries.len();
        let mut lo = 0;
        let mut hi = n - 1;

        while lo < hi - 1 {
            let mid = (lo + hi) / 2;
            if self.entries[mid] < target {
                lo = mid;
            } else {
                hi = mid;
            }
        }

        // Linear interpolation within the interval
        let seg_len = self.entries[hi] - self.entries[lo];
        let frac = if seg_len > 1e-10 {
            (target - self.entries[lo]) / seg_len
        } else {
            0.0
        };

        let t_lo = lo as f32 / (n - 1) as f32;
        let t_hi = hi as f32 / (n - 1) as f32;
        t_lo + frac * (t_hi - t_lo)
    }
}

// ── PolyPath ────────────────────────────────────────────────────────────────

/// A smooth path through a sequence of 2D points.
///
/// Internally uses a [`CatmullRomSpline`] for smooth interpolation and an
/// arc-length lookup table for constant-speed motion.
///
/// Equivalent to GSAP's `path: [{x,y}...]` point array.
///
/// ```rust
/// use spanda::motion_path::PolyPath;
///
/// let path = PolyPath::from_points(vec![
///     [0.0, 0.0],
///     [100.0, 50.0],
///     [200.0, 0.0],
/// ]);
///
/// let start = path.position(0.0);
/// let mid = path.position(0.5);
/// let end = path.position(1.0);
/// ```
pub struct PolyPath {
    spline: CatmullRomSpline,
    arc_table: ArcLengthTable,
    start_offset: f32,
    end_offset: f32,
    rotation_offset_rad: f32,
}

impl core::fmt::Debug for PolyPath {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PolyPath")
            .field("points", &self.spline.point_count())
            .field("arc_length", &self.arc_table.total_length)
            .field("start_offset", &self.start_offset)
            .field("end_offset", &self.end_offset)
            .finish()
    }
}

impl PolyPath {
    /// Create a smooth path through the given points.
    pub fn from_points(points: Vec<[f32; 2]>) -> Self {
        let spline = CatmullRomSpline::new(points);
        let arc_table =
            ArcLengthTable::build(|t| spline.evaluate([0.0, 0.0], t), ARC_LEN_SAMPLES);
        Self {
            spline,
            arc_table,
            start_offset: 0.0,
            end_offset: 1.0,
            rotation_offset_rad: 0.0,
        }
    }

    /// Create a path with custom tension (curviness).
    ///
    /// GSAP equivalent: `curviness` property. Standard is `0.5`,
    /// higher values create more exaggerated curves.
    pub fn from_points_with_tension(points: Vec<[f32; 2]>, tension: f32) -> Self {
        let spline = CatmullRomSpline::new(points).tension(tension);
        let arc_table =
            ArcLengthTable::build(|t| spline.evaluate([0.0, 0.0], t), ARC_LEN_SAMPLES);
        Self {
            spline,
            arc_table,
            start_offset: 0.0,
            end_offset: 1.0,
            rotation_offset_rad: 0.0,
        }
    }

    /// Set starting offset — begin at this fraction of the path.
    ///
    /// GSAP equivalent: `start: 0.5` (begin at 50% of the path).
    pub fn start_offset(mut self, offset: f32) -> Self {
        self.start_offset = offset.clamp(0.0, 1.0);
        self
    }

    /// Set ending offset — stop at this fraction of the path.
    ///
    /// GSAP equivalent: `end: 0.8` (stop at 80% of the path).
    pub fn end_offset(mut self, offset: f32) -> Self {
        self.end_offset = offset.clamp(0.0, 1.0);
        self
    }

    /// Set rotation offset in degrees.
    ///
    /// Added to the auto-rotation angle. GSAP equivalent: `autoRotate: 90`.
    pub fn rotation_offset(mut self, degrees: f32) -> Self {
        self.rotation_offset_rad = degrees.to_radians();
        self
    }

    /// Total arc length of the full path.
    pub fn arc_length(&self) -> f32 {
        self.arc_table.total_length
    }

    /// Map user-facing `u ∈ [0, 1]` through start/end offsets and arc-length.
    fn map_u(&self, u: f32) -> f32 {
        let u = u.clamp(0.0, 1.0);
        // Map [0,1] to [start_offset, end_offset]
        let effective = self.start_offset + u * (self.end_offset - self.start_offset);
        // Arc-length reparameterize for constant speed
        self.arc_table.uniform_to_t(effective)
    }

    /// Position on the path at progress `u ∈ [0, 1]`.
    ///
    /// Uses arc-length parameterization for constant speed and respects
    /// start/end offsets.
    pub fn position(&self, u: f32) -> [f32; 2] {
        let t = self.map_u(u);
        self.spline.evaluate([0.0, 0.0], t)
    }

    /// Tangent vector at progress `u ∈ [0, 1]`.
    pub fn tangent(&self, u: f32) -> [f32; 2] {
        let t = self.map_u(u);
        self.spline.tangent([0.0, 0.0], t)
    }

    /// Auto-rotation angle in radians at progress `u`, including the rotation
    /// offset. Use for "auto-rotate along path" effects.
    pub fn rotation(&self, u: f32) -> f32 {
        let tan = self.tangent(u);
        tangent_angle(tan) + self.rotation_offset_rad
    }

    /// Auto-rotation angle in degrees at progress `u`, including offset.
    pub fn rotation_deg(&self, u: f32) -> f32 {
        self.rotation(u).to_degrees()
    }

    /// Get relative position along the path for a world-space point.
    ///
    /// Returns the progress value `u ∈ [0, 1]` for the point on the path
    /// closest to the given world position. Useful for determining how far
    /// along the path an object is.
    ///
    /// GSAP equivalent: `MotionPathPlugin.getRelativePosition()`
    ///
    /// # Example
    ///
    /// ```rust
    /// use spanda::motion_path::PolyPath;
    ///
    /// let path = PolyPath::from_points(vec![
    ///     [0.0, 0.0],
    ///     [100.0, 0.0],
    ///     [200.0, 0.0],
    /// ]);
    ///
    /// // Point at [100, 0] should be ~50% along the path
    /// let progress = path.get_relative_position([100.0, 0.0]);
    /// assert!((progress - 0.5).abs() < 0.1);
    /// ```
    pub fn get_relative_position(&self, point: [f32; 2]) -> f32 {
        self.get_relative_position_with_precision(point, 100)
    }

    /// Get relative position with custom sample precision.
    ///
    /// Higher sample count gives more accurate results but is slower.
    pub fn get_relative_position_with_precision(&self, point: [f32; 2], samples: usize) -> f32 {
        let samples = samples.max(2);
        let mut best_u = 0.0_f32;
        let mut best_dist_sq = f32::MAX;

        for i in 0..=samples {
            let u = i as f32 / samples as f32;
            let p = self.position(u);
            let dx = p[0] - point[0];
            let dy = p[1] - point[1];
            let dist_sq = dx * dx + dy * dy;

            if dist_sq < best_dist_sq {
                best_dist_sq = dist_sq;
                best_u = u;
            }
        }

        best_u
    }

    /// Get the distance from a point to the closest point on the path.
    ///
    /// Returns a tuple of (progress, distance) where progress is `u ∈ [0, 1]`
    /// and distance is the Euclidean distance.
    pub fn closest_point(&self, point: [f32; 2]) -> (f32, f32) {
        let u = self.get_relative_position(point);
        let p = self.position(u);
        let dx = p[0] - point[0];
        let dy = p[1] - point[1];
        let dist = (dx * dx + dy * dy).sqrt();
        (u, dist)
    }
}

// ── CompoundPath ────────────────────────────────────────────────────────────

/// A path segment command — SVG-style.
#[derive(Debug, Clone)]
pub enum PathCommand {
    /// Move to a point (starts a new subpath).
    MoveTo([f32; 2]),
    /// Straight line to a point.
    LineTo([f32; 2]),
    /// Quadratic Bezier curve (one control point + endpoint).
    QuadTo {
        /// Control point.
        control: [f32; 2],
        /// End point.
        end: [f32; 2],
    },
    /// Cubic Bezier curve (two control points + endpoint).
    CubicTo {
        /// First control point.
        control1: [f32; 2],
        /// Second control point.
        control2: [f32; 2],
        /// End point.
        end: [f32; 2],
    },
    /// Close the path (line back to the last MoveTo point).
    Close,
}

/// A multi-segment 2D path composed of SVG-style commands.
///
/// Like GSAP's compound path handling, supports Move, Line, Quad, and Cubic
/// segments. Provides arc-length parameterized evaluation.
///
/// ```rust
/// use spanda::motion_path::{CompoundPath, PathCommand};
///
/// let path = CompoundPath::new(vec![
///     PathCommand::MoveTo([0.0, 0.0]),
///     PathCommand::CubicTo {
///         control1: [50.0, 100.0],
///         control2: [100.0, 100.0],
///         end: [150.0, 0.0],
///     },
///     PathCommand::LineTo([200.0, 0.0]),
/// ]);
///
/// let pos = path.position(0.5);
/// ```
pub struct CompoundPath {
    segments: Vec<Segment>,
    arc_table: ArcLengthTable,
    start_offset: f32,
    end_offset: f32,
    rotation_offset_rad: f32,
}

/// Internal segment representation (start + end are resolved).
#[derive(Debug, Clone)]
enum Segment {
    Line {
        start: [f32; 2],
        end: [f32; 2],
    },
    Quad {
        start: [f32; 2],
        control: [f32; 2],
        end: [f32; 2],
    },
    Cubic {
        start: [f32; 2],
        control1: [f32; 2],
        control2: [f32; 2],
        end: [f32; 2],
    },
}

impl Segment {
    fn evaluate(&self, t: f32) -> [f32; 2] {
        match self {
            Segment::Line { start, end } => [
                start[0] + (end[0] - start[0]) * t,
                start[1] + (end[1] - start[1]) * t,
            ],
            Segment::Quad {
                start,
                control,
                end,
            } => {
                let inv = 1.0 - t;
                let inv2 = inv * inv;
                let t2 = t * t;
                [
                    inv2 * start[0] + 2.0 * inv * t * control[0] + t2 * end[0],
                    inv2 * start[1] + 2.0 * inv * t * control[1] + t2 * end[1],
                ]
            }
            Segment::Cubic {
                start,
                control1,
                control2,
                end,
            } => {
                let inv = 1.0 - t;
                let inv2 = inv * inv;
                let inv3 = inv2 * inv;
                let t2 = t * t;
                let t3 = t2 * t;
                [
                    inv3 * start[0]
                        + 3.0 * inv2 * t * control1[0]
                        + 3.0 * inv * t2 * control2[0]
                        + t3 * end[0],
                    inv3 * start[1]
                        + 3.0 * inv2 * t * control1[1]
                        + 3.0 * inv * t2 * control2[1]
                        + t3 * end[1],
                ]
            }
        }
    }

    fn derivative(&self, t: f32) -> [f32; 2] {
        match self {
            Segment::Line { start, end } => [end[0] - start[0], end[1] - start[1]],
            Segment::Quad {
                start,
                control,
                end,
            } => {
                let inv = 1.0 - t;
                [
                    2.0 * inv * (control[0] - start[0]) + 2.0 * t * (end[0] - control[0]),
                    2.0 * inv * (control[1] - start[1]) + 2.0 * t * (end[1] - control[1]),
                ]
            }
            Segment::Cubic {
                start,
                control1,
                control2,
                end,
            } => {
                let inv = 1.0 - t;
                let inv2 = inv * inv;
                let t2 = t * t;
                [
                    3.0 * inv2 * (control1[0] - start[0])
                        + 6.0 * inv * t * (control2[0] - control1[0])
                        + 3.0 * t2 * (end[0] - control2[0]),
                    3.0 * inv2 * (control1[1] - start[1])
                        + 6.0 * inv * t * (control2[1] - control1[1])
                        + 3.0 * t2 * (end[1] - control2[1]),
                ]
            }
        }
    }
}

impl CompoundPath {
    /// Create a compound path from a list of commands.
    pub fn new(commands: Vec<PathCommand>) -> Self {
        let segments = Self::resolve_commands(&commands);
        let seg_clone = segments.clone();
        let seg_count = seg_clone.len();

        let arc_table = ArcLengthTable::build(
            |t| Self::eval_segments(&seg_clone, seg_count, t),
            ARC_LEN_SAMPLES,
        );

        Self {
            segments,
            arc_table,
            start_offset: 0.0,
            end_offset: 1.0,
            rotation_offset_rad: 0.0,
        }
    }

    /// Set starting offset.
    pub fn start_offset(mut self, offset: f32) -> Self {
        self.start_offset = offset.clamp(0.0, 1.0);
        self
    }

    /// Set ending offset.
    pub fn end_offset(mut self, offset: f32) -> Self {
        self.end_offset = offset.clamp(0.0, 1.0);
        self
    }

    /// Set rotation offset in degrees.
    pub fn rotation_offset(mut self, degrees: f32) -> Self {
        self.rotation_offset_rad = degrees.to_radians();
        self
    }

    /// Total arc length.
    pub fn arc_length(&self) -> f32 {
        self.arc_table.total_length
    }

    /// Number of resolved segments.
    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }

    /// Resolve commands into segments, tracking the current position.
    fn resolve_commands(commands: &[PathCommand]) -> Vec<Segment> {
        let mut segments = Vec::new();
        let mut current = [0.0_f32, 0.0];
        let mut subpath_start = current;

        for cmd in commands {
            match cmd {
                PathCommand::MoveTo(p) => {
                    current = *p;
                    subpath_start = *p;
                }
                PathCommand::LineTo(end) => {
                    segments.push(Segment::Line {
                        start: current,
                        end: *end,
                    });
                    current = *end;
                }
                PathCommand::QuadTo { control, end } => {
                    segments.push(Segment::Quad {
                        start: current,
                        control: *control,
                        end: *end,
                    });
                    current = *end;
                }
                PathCommand::CubicTo {
                    control1,
                    control2,
                    end,
                } => {
                    segments.push(Segment::Cubic {
                        start: current,
                        control1: *control1,
                        control2: *control2,
                        end: *end,
                    });
                    current = *end;
                }
                PathCommand::Close => {
                    if (current[0] - subpath_start[0]).abs() > 1e-10
                        || (current[1] - subpath_start[1]).abs() > 1e-10
                    {
                        segments.push(Segment::Line {
                            start: current,
                            end: subpath_start,
                        });
                        current = subpath_start;
                    }
                }
            }
        }

        segments
    }

    /// Evaluate segments at global t ∈ [0, 1].
    fn eval_segments(segments: &[Segment], seg_count: usize, t: f32) -> [f32; 2] {
        if seg_count == 0 {
            return [0.0, 0.0];
        }
        let t = t.clamp(0.0, 1.0);
        let scaled = t * seg_count as f32;
        let idx = (scaled.floor() as usize).min(seg_count - 1);
        let local = (scaled - idx as f32).clamp(0.0, 1.0);
        segments[idx].evaluate(local)
    }

    /// Map user-facing u through offsets and arc-length.
    fn map_u(&self, u: f32) -> f32 {
        let u = u.clamp(0.0, 1.0);
        let effective = self.start_offset + u * (self.end_offset - self.start_offset);
        self.arc_table.uniform_to_t(effective)
    }

    /// Position at progress `u ∈ [0, 1]` with arc-length parameterization.
    pub fn position(&self, u: f32) -> [f32; 2] {
        let t = self.map_u(u);
        Self::eval_segments(&self.segments, self.segments.len(), t)
    }

    /// Tangent vector at progress `u`.
    pub fn tangent(&self, u: f32) -> [f32; 2] {
        let t = self.map_u(u);
        let seg_count = self.segments.len();
        if seg_count == 0 {
            return [1.0, 0.0];
        }
        let t_clamped = t.clamp(0.0, 1.0);
        let scaled = t_clamped * seg_count as f32;
        let idx = (scaled.floor() as usize).min(seg_count - 1);
        let local = (scaled - idx as f32).clamp(0.0, 1.0);
        self.segments[idx].derivative(local)
    }

    /// Auto-rotation angle in radians at progress `u`, including offset.
    pub fn rotation(&self, u: f32) -> f32 {
        let tan = self.tangent(u);
        tangent_angle(tan) + self.rotation_offset_rad
    }

    /// Auto-rotation angle in degrees at progress `u`.
    pub fn rotation_deg(&self, u: f32) -> f32 {
        self.rotation(u).to_degrees()
    }

    /// Get relative position along the path for a world-space point.
    ///
    /// Returns the progress value `u ∈ [0, 1]` for the point on the path
    /// closest to the given world position.
    ///
    /// GSAP equivalent: `MotionPathPlugin.getRelativePosition()`
    pub fn get_relative_position(&self, point: [f32; 2]) -> f32 {
        self.get_relative_position_with_precision(point, 100)
    }

    /// Get relative position with custom sample precision.
    pub fn get_relative_position_with_precision(&self, point: [f32; 2], samples: usize) -> f32 {
        let samples = samples.max(2);
        let mut best_u = 0.0_f32;
        let mut best_dist_sq = f32::MAX;

        for i in 0..=samples {
            let u = i as f32 / samples as f32;
            let p = self.position(u);
            let dx = p[0] - point[0];
            let dy = p[1] - point[1];
            let dist_sq = dx * dx + dy * dy;

            if dist_sq < best_dist_sq {
                best_dist_sq = dist_sq;
                best_u = u;
            }
        }

        best_u
    }

    /// Get the distance from a point to the closest point on the path.
    pub fn closest_point(&self, point: [f32; 2]) -> (f32, f32) {
        let u = self.get_relative_position(point);
        let p = self.position(u);
        let dx = p[0] - point[0];
        let dy = p[1] - point[1];
        let dist = (dx * dx + dy * dy).sqrt();
        (u, dist)
    }
}

impl core::fmt::Debug for CompoundPath {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CompoundPath")
            .field("segments", &self.segments.len())
            .field("arc_length", &self.arc_table.total_length)
            .field("start_offset", &self.start_offset)
            .field("end_offset", &self.end_offset)
            .finish()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── PolyPath tests ─────────────────────────────────────────────────────

    #[test]
    fn polypath_basic_endpoints() {
        let path = PolyPath::from_points(vec![
            [0.0, 0.0],
            [100.0, 0.0],
            [200.0, 0.0],
        ]);

        let start = path.position(0.0);
        let end = path.position(1.0);
        assert!((start[0]).abs() < 1.0, "Expected x~0, got {}", start[0]);
        assert!((end[0] - 200.0).abs() < 1.0, "Expected x~200, got {}", end[0]);
    }

    #[test]
    fn polypath_constant_speed() {
        // Straight horizontal path
        let path = PolyPath::from_points(vec![
            [0.0, 0.0],
            [100.0, 0.0],
            [200.0, 0.0],
        ]);

        // Arc-length parameterized: u=0.25 should give x~50
        let quarter = path.position(0.25);
        assert!(
            (quarter[0] - 50.0).abs() < 5.0,
            "Expected x~50 at u=0.25, got {}",
            quarter[0]
        );
    }

    #[test]
    fn polypath_start_offset() {
        let path = PolyPath::from_points(vec![
            [0.0, 0.0],
            [100.0, 0.0],
        ]).start_offset(0.5);

        // u=0 should start at 50% of the path
        let start = path.position(0.0);
        assert!(
            (start[0] - 50.0).abs() < 5.0,
            "Expected x~50 with start_offset=0.5, got {}",
            start[0]
        );
    }

    #[test]
    fn polypath_end_offset() {
        let path = PolyPath::from_points(vec![
            [0.0, 0.0],
            [100.0, 0.0],
        ]).end_offset(0.5);

        // u=1.0 should be at 50% of the path
        let end = path.position(1.0);
        assert!(
            (end[0] - 50.0).abs() < 5.0,
            "Expected x~50 with end_offset=0.5, got {}",
            end[0]
        );
    }

    #[test]
    fn polypath_rotation() {
        let path = PolyPath::from_points(vec![
            [0.0, 0.0],
            [100.0, 0.0],
        ]);

        // Horizontal path should give ~0 degree rotation
        let rot = path.rotation_deg(0.5);
        assert!(rot.abs() < 5.0, "Expected ~0 degrees, got {rot}");
    }

    #[test]
    fn polypath_rotation_offset() {
        let path = PolyPath::from_points(vec![
            [0.0, 0.0],
            [100.0, 0.0],
        ]).rotation_offset(90.0);

        // Horizontal path + 90° offset = ~90°
        let rot = path.rotation_deg(0.5);
        assert!((rot - 90.0).abs() < 5.0, "Expected ~90 degrees, got {rot}");
    }

    #[test]
    fn polypath_with_tension() {
        let normal = PolyPath::from_points(vec![
            [0.0, 0.0],
            [50.0, 100.0],
            [100.0, 0.0],
        ]);

        let high = PolyPath::from_points_with_tension(
            vec![
                [0.0, 0.0],
                [50.0, 100.0],
                [100.0, 0.0],
            ],
            1.5,
        );

        // Different tension should produce different positions
        let p1 = normal.position(0.25);
        let p2 = high.position(0.25);
        // They may differ in y due to different control points
        let diff = ((p1[0] - p2[0]).powi(2) + (p1[1] - p2[1]).powi(2)).sqrt();
        assert!(diff > 0.1, "Different tensions should produce different paths");
    }

    // ── CompoundPath tests ─────────────────────────────────────────────────

    #[test]
    fn compound_path_line() {
        let path = CompoundPath::new(vec![
            PathCommand::MoveTo([0.0, 0.0]),
            PathCommand::LineTo([100.0, 0.0]),
        ]);

        let start = path.position(0.0);
        let end = path.position(1.0);
        assert!((start[0]).abs() < 1.0);
        assert!((end[0] - 100.0).abs() < 1.0);
    }

    #[test]
    fn compound_path_cubic() {
        let path = CompoundPath::new(vec![
            PathCommand::MoveTo([0.0, 0.0]),
            PathCommand::CubicTo {
                control1: [50.0, 100.0],
                control2: [100.0, 100.0],
                end: [150.0, 0.0],
            },
        ]);

        let start = path.position(0.0);
        let end = path.position(1.0);
        assert!((start[0]).abs() < 1.0);
        assert!((end[0] - 150.0).abs() < 1.0);

        // Midpoint should be above y=0
        let mid = path.position(0.5);
        assert!(mid[1] > 10.0, "Expected y > 10 at midpoint, got {}", mid[1]);
    }

    #[test]
    fn compound_path_multi_segment() {
        let path = CompoundPath::new(vec![
            PathCommand::MoveTo([0.0, 0.0]),
            PathCommand::LineTo([100.0, 0.0]),
            PathCommand::LineTo([100.0, 100.0]),
        ]);

        let end = path.position(1.0);
        assert!(
            (end[0] - 100.0).abs() < 1.0 && (end[1] - 100.0).abs() < 1.0,
            "Expected (100, 100), got ({}, {})",
            end[0],
            end[1]
        );
    }

    #[test]
    fn compound_path_close() {
        let path = CompoundPath::new(vec![
            PathCommand::MoveTo([0.0, 0.0]),
            PathCommand::LineTo([100.0, 0.0]),
            PathCommand::LineTo([100.0, 100.0]),
            PathCommand::Close,
        ]);

        // Close should add a line back to (0, 0)
        assert_eq!(path.segment_count(), 3);
        let end = path.position(1.0);
        assert!((end[0]).abs() < 1.0 && (end[1]).abs() < 1.0);
    }

    #[test]
    fn compound_path_start_end_offset() {
        let path = CompoundPath::new(vec![
            PathCommand::MoveTo([0.0, 0.0]),
            PathCommand::LineTo([200.0, 0.0]),
        ])
        .start_offset(0.25)
        .end_offset(0.75);

        let start = path.position(0.0);
        let end = path.position(1.0);

        assert!(
            (start[0] - 50.0).abs() < 5.0,
            "Expected x~50 at start, got {}",
            start[0]
        );
        assert!(
            (end[0] - 150.0).abs() < 5.0,
            "Expected x~150 at end, got {}",
            end[0]
        );
    }

    #[test]
    fn compound_path_tangent() {
        let path = CompoundPath::new(vec![
            PathCommand::MoveTo([0.0, 0.0]),
            PathCommand::LineTo([100.0, 0.0]),
        ]);

        let tan = path.tangent(0.5);
        assert!(tan[0] > 0.0, "Expected positive x tangent");
        assert!((tan[1]).abs() < 1e-4, "Expected zero y tangent");
    }

    #[test]
    fn compound_path_rotation() {
        let path = CompoundPath::new(vec![
            PathCommand::MoveTo([0.0, 0.0]),
            PathCommand::LineTo([0.0, 100.0]),
        ]);

        let rot = path.rotation_deg(0.5);
        assert!((rot - 90.0).abs() < 1.0, "Expected ~90deg for upward path, got {rot}");
    }

    #[test]
    fn compound_path_empty() {
        let path = CompoundPath::new(vec![]);
        let pos = path.position(0.5);
        assert!((pos[0]).abs() < 1e-4);
        assert!((pos[1]).abs() < 1e-4);
    }
}
