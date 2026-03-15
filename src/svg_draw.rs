//! DrawSVG helper — animate `stroke-dashoffset` for SVG path drawing effects.
//!
//! These are thin convenience constructors that return a [`TweenBuilder<f32>`].
//! You chain `.duration()`, `.easing()`, `.build()` as usual, then read
//! `.value()` each frame and apply it as the element's `stroke-dashoffset`.
//!
//! # Example
//!
//! ```rust
//! use spanda::svg_draw::draw_on;
//! use spanda::easing::Easing;
//! use spanda::traits::Update;
//!
//! // Total path length (from getTotalLength() or CompoundPath::arc_length())
//! let path_length = 320.0;
//!
//! let mut tween = draw_on(path_length)
//!     .duration(1.5)
//!     .easing(Easing::EaseInOutCubic)
//!     .build();
//!
//! // Set stroke-dasharray to path_length on your SVG element.
//! // Each frame, apply tween.value() as stroke-dashoffset.
//! tween.update(0.75);
//! let offset = tween.value();
//! assert!(offset < path_length);
//! ```

use crate::tween::{Tween, TweenBuilder};

/// Create a tween that animates from `path_length` → `0.0` (draw on effect).
///
/// Set `stroke-dasharray` to `path_length` on your SVG element, then apply
/// the tween's `value()` as `stroke-dashoffset` each frame.
pub fn draw_on(path_length: f32) -> TweenBuilder<f32> {
    Tween::new(path_length, 0.0)
}

/// Create a tween that animates from `0.0` → `path_length` (erase / draw-off).
///
/// Reverse of [`draw_on`] — the path progressively disappears.
pub fn draw_on_reverse(path_length: f32) -> TweenBuilder<f32> {
    Tween::new(0.0, path_length)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::easing::Easing;
    use crate::traits::Update;

    #[test]
    fn draw_on_starts_at_length() {
        let tween = draw_on(100.0).duration(1.0).build();
        assert!((tween.value() - 100.0).abs() < 1e-6);
    }

    #[test]
    fn draw_on_ends_at_zero() {
        let mut tween = draw_on(100.0).duration(1.0).build();
        tween.update(1.0);
        assert!((tween.value()).abs() < 1e-6);
    }

    #[test]
    fn draw_on_reverse_starts_at_zero() {
        let tween = draw_on_reverse(100.0).duration(1.0).build();
        assert!((tween.value()).abs() < 1e-6);
    }

    #[test]
    fn draw_on_reverse_ends_at_length() {
        let mut tween = draw_on_reverse(100.0).duration(1.0).build();
        tween.update(1.0);
        assert!((tween.value() - 100.0).abs() < 1e-6);
    }

    #[test]
    fn draw_on_with_easing() {
        let mut tween = draw_on(200.0)
            .duration(2.0)
            .easing(Easing::EaseOutCubic)
            .build();
        tween.update(1.0); // halfway through duration
        let val = tween.value();
        // EaseOutCubic at t=0.5 > 0.5, so offset should be < 100
        assert!(val < 100.0, "Expected < 100, got {val}");
    }
}
