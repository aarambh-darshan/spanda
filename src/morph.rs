//! Shape morphing — lerp between two sets of 2D control points.
//!
//! `MorphPath` animates a smooth transition between two shapes defined as
//! sequences of `[f32; 2]` points. Each point in the source shape is linearly
//! interpolated toward its corresponding point in the target shape.
//!
//! If the two shapes have different point counts, use [`resample`] to
//! normalise them, or let the builder handle it automatically.
//!
//! # Example
//!
//! ```rust
//! use spanda::morph::MorphPath;
//! use spanda::easing::Easing;
//! use spanda::traits::Update;
//!
//! let triangle = vec![[0.0, 0.0], [50.0, 100.0], [100.0, 0.0]];
//! let square   = vec![[0.0, 0.0], [0.0, 100.0], [100.0, 100.0]];
//!
//! let mut morph = MorphPath::new(triangle, square)
//!     .duration(1.0)
//!     .easing(Easing::EaseInOutCubic)
//!     .build();
//!
//! morph.update(0.5);
//! let points = morph.value();
//! assert_eq!(points.len(), 3);
//! ```

use crate::easing::Easing;
use crate::traits::Update;

/// Animated shape morph between two sets of 2D points.
#[derive(Clone, Debug)]
pub struct MorphPath {
    from_points: Vec<[f32; 2]>,
    to_points: Vec<[f32; 2]>,
    duration: f32,
    easing: Easing,
    elapsed: f32,
    completed: bool,
}

/// Builder for [`MorphPath`].
#[derive(Debug)]
pub struct MorphPathBuilder {
    from_points: Vec<[f32; 2]>,
    to_points: Vec<[f32; 2]>,
    duration: f32,
    easing: Easing,
    shape_index: ShapeIndex,
}

impl MorphPath {
    /// Start building a morph from `from` points to `to` points.
    ///
    /// If the two point arrays have different lengths, the shorter one is
    /// automatically resampled to match the longer.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(from: Vec<[f32; 2]>, to: Vec<[f32; 2]>) -> MorphPathBuilder {
        MorphPathBuilder {
            from_points: from,
            to_points: to,
            duration: 1.0,
            easing: Easing::Linear,
            shape_index: ShapeIndex::None,
        }
    }

    /// Current interpolated points at the current progress.
    pub fn value(&self) -> Vec<[f32; 2]> {
        let raw_t = if self.duration > 0.0 {
            (self.elapsed / self.duration).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let t = self.easing.apply(raw_t);

        self.from_points
            .iter()
            .zip(self.to_points.iter())
            .map(|(a, b)| [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t])
            .collect()
    }

    /// Raw progress `0.0..=1.0` (before easing).
    pub fn progress(&self) -> f32 {
        if self.duration > 0.0 {
            (self.elapsed / self.duration).clamp(0.0, 1.0)
        } else {
            1.0
        }
    }

    /// Whether the morph animation has completed.
    pub fn is_complete(&self) -> bool {
        self.completed
    }

    /// Reset the animation to the beginning.
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
        self.completed = false;
    }

    /// Jump to a specific progress value `t` (0.0..=1.0).
    pub fn seek(&mut self, t: f32) {
        self.elapsed = t.clamp(0.0, 1.0) * self.duration;
        self.completed = t >= 1.0;
    }
}

impl Update for MorphPath {
    fn update(&mut self, dt: f32) -> bool {
        if self.completed {
            return false;
        }
        self.elapsed += dt;
        if self.elapsed >= self.duration {
            self.elapsed = self.duration;
            self.completed = true;
        }
        !self.completed
    }
}

impl MorphPathBuilder {
    /// Set animation duration in seconds (default: 1.0).
    pub fn duration(mut self, d: f32) -> Self {
        self.duration = d;
        self
    }

    /// Set the easing curve (default: Linear).
    pub fn easing(mut self, e: Easing) -> Self {
        self.easing = e;
        self
    }

    /// Set the shape index for point correspondence.
    ///
    /// GSAP equivalent: `shapeIndex` property.
    ///
    /// Controls which points in the source shape map to which points in
    /// the target shape. Use `ShapeIndex::Auto` for automatic detection
    /// of the best rotation, or `ShapeIndex::Offset(n)` for manual control.
    pub fn shape_index(mut self, index: ShapeIndex) -> Self {
        self.shape_index = index;
        self
    }

    /// Build the `MorphPath`. Auto-resamples if point counts differ.
    pub fn build(mut self) -> MorphPath {
        let from_len = self.from_points.len();
        let to_len = self.to_points.len();

        // Resample to match point counts
        if from_len != to_len && from_len > 0 && to_len > 0 {
            let target = from_len.max(to_len);
            if from_len < target {
                self.from_points = resample(&self.from_points, target);
            } else {
                self.to_points = resample(&self.to_points, target);
            }
        }

        // Apply shape index rotation to target points
        let to_points = match self.shape_index {
            ShapeIndex::Auto => {
                let best = ShapeIndex::auto(&self.from_points, &self.to_points);
                best.apply(&self.to_points)
            }
            other => other.apply(&self.to_points),
        };

        MorphPath {
            from_points: self.from_points,
            to_points,
            duration: self.duration,
            easing: self.easing,
            elapsed: 0.0,
            completed: false,
        }
    }
}

/// Resample a polyline to `target_count` evenly-spaced points along its arc length.
///
/// Preserves the first and last endpoints. If `points` has fewer than 2 entries
/// or `target_count` is 0, returns `points` unchanged (or empty).
pub fn resample(points: &[[f32; 2]], target_count: usize) -> Vec<[f32; 2]> {
    if points.len() < 2 || target_count < 2 {
        return if target_count == 1 && !points.is_empty() {
            vec![points[0]]
        } else {
            points.to_vec()
        };
    }

    // Build cumulative arc-length table
    let mut lengths = Vec::with_capacity(points.len());
    lengths.push(0.0_f32);
    for i in 1..points.len() {
        let dx = points[i][0] - points[i - 1][0];
        let dy = points[i][1] - points[i - 1][1];
        let seg_len = (dx * dx + dy * dy).sqrt();
        lengths.push(lengths[i - 1] + seg_len);
    }

    let total_len = *lengths.last().unwrap();
    if total_len < 1e-10 {
        return vec![points[0]; target_count];
    }

    let mut result = Vec::with_capacity(target_count);
    for i in 0..target_count {
        let target_dist = total_len * (i as f32 / (target_count - 1) as f32);

        // Binary search for the segment containing target_dist
        let seg = match lengths.binary_search_by(|l| l.partial_cmp(&target_dist).unwrap()) {
            Ok(idx) => idx.min(points.len() - 2),
            Err(idx) => {
                if idx == 0 {
                    0
                } else {
                    (idx - 1).min(points.len() - 2)
                }
            }
        };

        let seg_start = lengths[seg];
        let seg_end = lengths[seg + 1];
        let seg_len = seg_end - seg_start;

        let local_t = if seg_len > 1e-10 {
            (target_dist - seg_start) / seg_len
        } else {
            0.0
        };

        result.push([
            points[seg][0] + (points[seg + 1][0] - points[seg][0]) * local_t,
            points[seg][1] + (points[seg + 1][1] - points[seg][1]) * local_t,
        ]);
    }

    result
}

// ── ShapeIndex ──────────────────────────────────────────────────────────────

/// Controls point correspondence during shape morphing.
///
/// GSAP equivalent: `shapeIndex` property in MorphSVGPlugin.
///
/// When morphing between two shapes, `ShapeIndex` determines which point
/// in the source shape corresponds to which point in the target shape.
/// This can dramatically affect the visual quality of the morph.
///
/// # Example
///
/// ```rust
/// use spanda::morph::{MorphPath, ShapeIndex};
///
/// let triangle = vec![[0.0, 0.0], [50.0, 100.0], [100.0, 0.0]];
/// let square   = vec![[0.0, 0.0], [0.0, 100.0], [100.0, 100.0], [100.0, 0.0]];
///
/// // Use auto-detection for best point alignment
/// let index = ShapeIndex::auto(&triangle, &square);
///
/// let morph = MorphPath::new(triangle, square)
///     .shape_index(index)
///     .duration(1.0)
///     .build();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShapeIndex {
    /// No rotation - use points as-is (default).
    #[default]
    None,
    /// Rotate target points by this index offset.
    ///
    /// For a shape with N points, `Offset(k)` maps source point 0 to
    /// target point k, source point 1 to target point (k+1) % N, etc.
    Offset(i32),
    /// Automatically find the best rotation to minimize morph distance.
    Auto,
}

impl ShapeIndex {
    /// Compute the best rotation offset for two shapes.
    ///
    /// Finds the rotation that minimizes the total distance between
    /// corresponding points. Both shapes should have the same number
    /// of points (resample first if needed).
    pub fn auto(from: &[[f32; 2]], to: &[[f32; 2]]) -> Self {
        if from.is_empty() || to.is_empty() || from.len() != to.len() {
            return ShapeIndex::None;
        }

        let n = from.len();
        let mut best_offset = 0;
        let mut best_dist = f32::MAX;

        for offset in 0..n {
            let mut total_dist = 0.0_f32;
            for (i, from_pt) in from.iter().enumerate() {
                let to_idx = (i + offset) % n;
                let dx = to[to_idx][0] - from_pt[0];
                let dy = to[to_idx][1] - from_pt[1];
                total_dist += dx * dx + dy * dy;
            }

            if total_dist < best_dist {
                best_dist = total_dist;
                best_offset = offset;
            }
        }

        if best_offset == 0 {
            ShapeIndex::None
        } else {
            ShapeIndex::Offset(best_offset as i32)
        }
    }

    /// Apply the shape index rotation to a set of points.
    ///
    /// Returns a new vector with points rotated according to the index.
    pub fn apply(&self, points: &[[f32; 2]]) -> Vec<[f32; 2]> {
        if points.is_empty() {
            return Vec::new();
        }

        match self {
            ShapeIndex::None => points.to_vec(),
            ShapeIndex::Offset(k) => {
                let n = points.len() as i32;
                let k = k.rem_euclid(n) as usize;
                let mut result = Vec::with_capacity(points.len());
                for i in 0..points.len() {
                    result.push(points[(i + k) % points.len()]);
                }
                result
            }
            ShapeIndex::Auto => {
                // Auto requires both shapes - caller should use ShapeIndex::auto() first
                points.to_vec()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn morph_at_t0_returns_from() {
        let from = vec![[0.0, 0.0], [10.0, 10.0]];
        let to = vec![[100.0, 100.0], [200.0, 200.0]];
        let morph = MorphPath::new(from.clone(), to).duration(1.0).build();
        let val = morph.value();
        assert!((val[0][0] - 0.0).abs() < 1e-6);
        assert!((val[1][1] - 10.0).abs() < 1e-6);
    }

    #[test]
    fn morph_at_t1_returns_to() {
        let from = vec![[0.0, 0.0], [10.0, 10.0]];
        let to = vec![[100.0, 100.0], [200.0, 200.0]];
        let mut morph = MorphPath::new(from, to.clone()).duration(1.0).build();
        morph.update(1.0);
        let val = morph.value();
        assert!((val[0][0] - 100.0).abs() < 1e-6);
        assert!((val[1][1] - 200.0).abs() < 1e-6);
    }

    #[test]
    fn morph_midpoint() {
        let from = vec![[0.0, 0.0]];
        let to = vec![[100.0, 200.0]];
        let mut morph = MorphPath::new(from, to).duration(1.0).build();
        morph.update(0.5);
        let val = morph.value();
        assert!((val[0][0] - 50.0).abs() < 1e-5);
        assert!((val[0][1] - 100.0).abs() < 1e-5);
    }

    #[test]
    fn morph_auto_resample_mismatched_lengths() {
        let from = vec![[0.0, 0.0], [100.0, 0.0]];
        let to = vec![[0.0, 0.0], [50.0, 50.0], [100.0, 0.0]];
        let morph = MorphPath::new(from, to).duration(1.0).build();
        // Both should now have 3 points
        let val = morph.value();
        assert_eq!(val.len(), 3);
    }

    #[test]
    fn morph_update_returns_false_when_done() {
        let from = vec![[0.0, 0.0]];
        let to = vec![[10.0, 10.0]];
        let mut morph = MorphPath::new(from, to).duration(0.5).build();
        assert!(morph.update(0.3));
        assert!(!morph.update(0.3));
        assert!(morph.is_complete());
    }

    #[test]
    fn morph_reset() {
        let from = vec![[0.0, 0.0]];
        let to = vec![[10.0, 10.0]];
        let mut morph = MorphPath::new(from, to).duration(0.5).build();
        morph.update(1.0);
        assert!(morph.is_complete());
        morph.reset();
        assert!(!morph.is_complete());
        assert!((morph.value()[0][0]).abs() < 1e-6);
    }

    #[test]
    fn resample_preserves_endpoints() {
        let pts = vec![[0.0, 0.0], [50.0, 50.0], [100.0, 0.0]];
        let resampled = resample(&pts, 5);
        assert_eq!(resampled.len(), 5);
        assert!((resampled[0][0] - 0.0).abs() < 1e-5);
        assert!((resampled[0][1] - 0.0).abs() < 1e-5);
        assert!((resampled[4][0] - 100.0).abs() < 1e-5);
        assert!((resampled[4][1] - 0.0).abs() < 1e-5);
    }

    #[test]
    fn resample_single_point() {
        let pts = vec![[42.0, 17.0]];
        let resampled = resample(&pts, 1);
        assert_eq!(resampled.len(), 1);
        assert!((resampled[0][0] - 42.0).abs() < 1e-6);
    }
}
