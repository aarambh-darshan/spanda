//! Colour interpolation for the [`palette`](https://crates.io/crates/palette) crate.
//!
//! Activate with `features = ["palette"]` in your `Cargo.toml`.
//!
//! # Direct interpolation
//!
//! All supported palette types implement [`Interpolate`], giving you
//! component-wise lerp that works with [`Tween<T>`](crate::tween::Tween),
//! [`KeyframeTrack<T>`](crate::keyframe::KeyframeTrack), and every other
//! animation primitive:
//!
//! ```rust,ignore
//! use palette::Srgba;
//! use spanda::{Tween, Easing};
//! use spanda::traits::Update;
//!
//! let mut tween = Tween::new(
//!     Srgba::new(1.0_f32, 0.0, 0.0, 1.0),
//!     Srgba::new(0.0_f32, 0.0, 1.0, 1.0),
//! ).duration(1.0).easing(Easing::EaseInOutCubic).build();
//! tween.update(0.5);
//! let colour = tween.value();
//! ```
//!
//! # Colour-space-aware interpolation
//!
//! Interpolating in sRGB produces dull, dark midpoints.  For perceptually
//! smooth gradients, wrap your colours in [`InLab`] or [`InOklch`]:
//!
//! ```rust,ignore
//! use palette::Srgba;
//! use spanda::colour::InLab;
//! use spanda::{Tween, Easing};
//!
//! let start = InLab(Srgba::new(1.0, 0.0, 0.0, 1.0)); // red
//! let end   = InLab(Srgba::new(0.0, 0.0, 1.0, 1.0)); // blue
//!
//! let mut tween = Tween::new(start, end)
//!     .duration(1.0)
//!     .easing(Easing::EaseInOutCubic)
//!     .build();
//! ```

use crate::spring::SpringAnimatable;
use crate::traits::Interpolate;

use palette::{FromColor, Hsla, Lab, LinSrgb, LinSrgba, Oklch, Srgb, Srgba};

#[cfg(not(feature = "std"))]
use alloc::vec;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

// ── Hue helper ──────────────────────────────────────────────────────────────

/// Shortest-arc hue interpolation.  Both `a` and `b` are in degrees.
#[inline]
fn lerp_hue(a: f32, b: f32, t: f32) -> f32 {
    let mut diff = b - a;
    if diff > 180.0 {
        diff -= 360.0;
    }
    if diff < -180.0 {
        diff += 360.0;
    }
    let result = a + diff * t;
    ((result % 360.0) + 360.0) % 360.0
}

/// Clamp sRGB channels to [0, 1] after colour-space conversions.
#[inline]
fn clamp_srgba(c: Srgba) -> Srgba {
    Srgba::new(
        c.red.clamp(0.0, 1.0),
        c.green.clamp(0.0, 1.0),
        c.blue.clamp(0.0, 1.0),
        c.alpha.clamp(0.0, 1.0),
    )
}

// ── Direct Interpolate impls ────────────────────────────────────────────────

impl Interpolate for Srgba {
    #[inline]
    fn lerp(&self, other: &Self, t: f32) -> Self {
        Srgba::new(
            self.red + (other.red - self.red) * t,
            self.green + (other.green - self.green) * t,
            self.blue + (other.blue - self.blue) * t,
            self.alpha + (other.alpha - self.alpha) * t,
        )
    }
}

impl Interpolate for Srgb {
    #[inline]
    fn lerp(&self, other: &Self, t: f32) -> Self {
        Srgb::new(
            self.red + (other.red - self.red) * t,
            self.green + (other.green - self.green) * t,
            self.blue + (other.blue - self.blue) * t,
        )
    }
}

impl Interpolate for LinSrgba {
    #[inline]
    fn lerp(&self, other: &Self, t: f32) -> Self {
        LinSrgba::new(
            self.red + (other.red - self.red) * t,
            self.green + (other.green - self.green) * t,
            self.blue + (other.blue - self.blue) * t,
            self.alpha + (other.alpha - self.alpha) * t,
        )
    }
}

impl Interpolate for LinSrgb {
    #[inline]
    fn lerp(&self, other: &Self, t: f32) -> Self {
        LinSrgb::new(
            self.red + (other.red - self.red) * t,
            self.green + (other.green - self.green) * t,
            self.blue + (other.blue - self.blue) * t,
        )
    }
}

impl Interpolate for Lab {
    #[inline]
    fn lerp(&self, other: &Self, t: f32) -> Self {
        Lab::new(
            self.l + (other.l - self.l) * t,
            self.a + (other.a - self.a) * t,
            self.b + (other.b - self.b) * t,
        )
    }
}

impl Interpolate for palette::Laba {
    #[inline]
    fn lerp(&self, other: &Self, t: f32) -> Self {
        palette::Laba::new(
            self.color.l + (other.color.l - self.color.l) * t,
            self.color.a + (other.color.a - self.color.a) * t,
            self.color.b + (other.color.b - self.color.b) * t,
            self.alpha + (other.alpha - self.alpha) * t,
        )
    }
}

impl Interpolate for Oklch {
    #[inline]
    fn lerp(&self, other: &Self, t: f32) -> Self {
        Oklch::new(
            self.l + (other.l - self.l) * t,
            self.chroma + (other.chroma - self.chroma) * t,
            lerp_hue(
                self.hue.into_positive_degrees(),
                other.hue.into_positive_degrees(),
                t,
            ),
        )
    }
}

impl Interpolate for palette::Oklcha {
    #[inline]
    fn lerp(&self, other: &Self, t: f32) -> Self {
        palette::Oklcha::new(
            self.color.l + (other.color.l - self.color.l) * t,
            self.color.chroma + (other.color.chroma - self.color.chroma) * t,
            lerp_hue(
                self.color.hue.into_positive_degrees(),
                other.color.hue.into_positive_degrees(),
                t,
            ),
            self.alpha + (other.alpha - self.alpha) * t,
        )
    }
}

impl Interpolate for Hsla {
    #[inline]
    fn lerp(&self, other: &Self, t: f32) -> Self {
        Hsla::new(
            lerp_hue(
                self.color.hue.into_positive_degrees(),
                other.color.hue.into_positive_degrees(),
                t,
            ),
            self.color.saturation + (other.color.saturation - self.color.saturation) * t,
            self.color.lightness + (other.color.lightness - self.color.lightness) * t,
            self.alpha + (other.alpha - self.alpha) * t,
        )
    }
}

// ── Colour-space-aware wrappers ─────────────────────────────────────────────

/// Wrapper that stores an sRGB colour but interpolates in **CIE L\*a\*b\***
/// space.
///
/// Use with `Tween<InLab>` for perceptually smooth colour transitions that
/// avoid the dull/dark midpoints of naive sRGB lerp.
///
/// Access the inner `Srgba` with `.0`.
#[derive(Clone, Debug, PartialEq)]
pub struct InLab(pub Srgba);

impl Interpolate for InLab {
    #[inline]
    fn lerp(&self, other: &Self, t: f32) -> Self {
        let a: palette::Laba = palette::Laba::from_color(self.0);
        let b: palette::Laba = palette::Laba::from_color(other.0);
        let result = palette::Laba::new(
            a.color.l + (b.color.l - a.color.l) * t,
            a.color.a + (b.color.a - a.color.a) * t,
            a.color.b + (b.color.b - a.color.b) * t,
            a.alpha + (b.alpha - a.alpha) * t,
        );
        InLab(clamp_srgba(Srgba::from_color(result)))
    }
}

/// Wrapper that stores an sRGB colour but interpolates in **OKLCh** space.
///
/// Produces smooth, vibrant gradients with natural hue rotation.  The hue
/// channel uses shortest-arc interpolation automatically.
///
/// Access the inner `Srgba` with `.0`.
#[derive(Clone, Debug, PartialEq)]
pub struct InOklch(pub Srgba);

impl Interpolate for InOklch {
    #[inline]
    fn lerp(&self, other: &Self, t: f32) -> Self {
        let a: palette::Oklcha = palette::Oklcha::from_color(self.0);
        let b: palette::Oklcha = palette::Oklcha::from_color(other.0);
        let result = palette::Oklcha::new(
            a.color.l + (b.color.l - a.color.l) * t,
            a.color.chroma + (b.color.chroma - a.color.chroma) * t,
            lerp_hue(
                a.color.hue.into_positive_degrees(),
                b.color.hue.into_positive_degrees(),
                t,
            ),
            a.alpha + (b.alpha - a.alpha) * t,
        );
        InOklch(clamp_srgba(Srgba::from_color(result)))
    }
}

/// Wrapper that stores an sRGB colour but interpolates in **linear RGB**
/// space.
///
/// Produces physically correct blending without gamma-curve artifacts.
///
/// Access the inner `Srgba` with `.0`.
#[derive(Clone, Debug, PartialEq)]
pub struct InLinear(pub Srgba);

impl Interpolate for InLinear {
    #[inline]
    fn lerp(&self, other: &Self, t: f32) -> Self {
        let a: LinSrgba = LinSrgba::from_color(self.0);
        let b: LinSrgba = LinSrgba::from_color(other.0);
        let result = LinSrgba::new(
            a.red + (b.red - a.red) * t,
            a.green + (b.green - a.green) * t,
            a.blue + (b.blue - a.blue) * t,
            a.alpha + (b.alpha - a.alpha) * t,
        );
        InLinear(clamp_srgba(Srgba::from_color(result)))
    }
}

// ── Convenience free functions ──────────────────────────────────────────────

/// Interpolate two sRGB colours in CIE L\*a\*b\* space.
///
/// Produces perceptually smooth gradients.  For animations, prefer
/// `Tween<InLab>`.
pub fn lerp_in_lab(a: Srgba, b: Srgba, t: f32) -> Srgba {
    InLab(a).lerp(&InLab(b), t).0
}

/// Interpolate two sRGB colours in OKLCh space.
///
/// Produces smooth, vibrant gradients with natural hue rotation.
pub fn lerp_in_oklch(a: Srgba, b: Srgba, t: f32) -> Srgba {
    InOklch(a).lerp(&InOklch(b), t).0
}

/// Interpolate two sRGB colours in linear RGB space.
///
/// Produces physically correct blending.
pub fn lerp_in_linear(a: Srgba, b: Srgba, t: f32) -> Srgba {
    InLinear(a).lerp(&InLinear(b), t).0
}

// ── SpringAnimatable impls ──────────────────────────────────────────────────

impl SpringAnimatable for Srgba {
    fn to_components(&self) -> Vec<f32> {
        vec![self.red, self.green, self.blue, self.alpha]
    }
    fn from_components(c: &[f32]) -> Self {
        Srgba::new(c[0], c[1], c[2], c[3])
    }
}

impl SpringAnimatable for Srgb {
    fn to_components(&self) -> Vec<f32> {
        vec![self.red, self.green, self.blue]
    }
    fn from_components(c: &[f32]) -> Self {
        Srgb::new(c[0], c[1], c[2])
    }
}

impl SpringAnimatable for LinSrgba {
    fn to_components(&self) -> Vec<f32> {
        vec![self.red, self.green, self.blue, self.alpha]
    }
    fn from_components(c: &[f32]) -> Self {
        LinSrgba::new(c[0], c[1], c[2], c[3])
    }
}

impl SpringAnimatable for LinSrgb {
    fn to_components(&self) -> Vec<f32> {
        vec![self.red, self.green, self.blue]
    }
    fn from_components(c: &[f32]) -> Self {
        LinSrgb::new(c[0], c[1], c[2])
    }
}

impl SpringAnimatable for Lab {
    fn to_components(&self) -> Vec<f32> {
        vec![self.l, self.a, self.b]
    }
    fn from_components(c: &[f32]) -> Self {
        Lab::new(c[0], c[1], c[2])
    }
}

impl SpringAnimatable for palette::Laba {
    fn to_components(&self) -> Vec<f32> {
        vec![self.color.l, self.color.a, self.color.b, self.alpha]
    }
    fn from_components(c: &[f32]) -> Self {
        palette::Laba::new(c[0], c[1], c[2], c[3])
    }
}

impl SpringAnimatable for InLab {
    fn to_components(&self) -> Vec<f32> {
        vec![self.0.red, self.0.green, self.0.blue, self.0.alpha]
    }
    fn from_components(c: &[f32]) -> Self {
        InLab(Srgba::new(c[0], c[1], c[2], c[3]))
    }
}

impl SpringAnimatable for InOklch {
    fn to_components(&self) -> Vec<f32> {
        vec![self.0.red, self.0.green, self.0.blue, self.0.alpha]
    }
    fn from_components(c: &[f32]) -> Self {
        InOklch(Srgba::new(c[0], c[1], c[2], c[3]))
    }
}

impl SpringAnimatable for InLinear {
    fn to_components(&self) -> Vec<f32> {
        vec![self.0.red, self.0.green, self.0.blue, self.0.alpha]
    }
    fn from_components(c: &[f32]) -> Self {
        InLinear(Srgba::new(c[0], c[1], c[2], c[3]))
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Direct Interpolate ───────────────────────────────────────────

    #[test]
    fn srgba_lerp_endpoints() {
        let red = Srgba::new(1.0, 0.0, 0.0, 1.0);
        let blue = Srgba::new(0.0, 0.0, 1.0, 1.0);
        let start = red.lerp(&blue, 0.0);
        assert!((start.red - 1.0).abs() < 1e-6);
        assert!((start.blue - 0.0).abs() < 1e-6);
        let end = red.lerp(&blue, 1.0);
        assert!((end.red - 0.0).abs() < 1e-6);
        assert!((end.blue - 1.0).abs() < 1e-6);
    }

    #[test]
    fn srgba_lerp_midpoint() {
        let black = Srgba::new(0.0, 0.0, 0.0, 1.0);
        let white = Srgba::new(1.0, 1.0, 1.0, 1.0);
        let mid = black.lerp(&white, 0.5);
        assert!((mid.red - 0.5).abs() < 1e-6);
        assert!((mid.green - 0.5).abs() < 1e-6);
        assert!((mid.blue - 0.5).abs() < 1e-6);
    }

    #[test]
    fn srgba_alpha_interpolation() {
        let transparent = Srgba::new(1.0, 0.0, 0.0, 0.0);
        let opaque = Srgba::new(1.0, 0.0, 0.0, 1.0);
        let mid = transparent.lerp(&opaque, 0.5);
        assert!((mid.alpha - 0.5).abs() < 1e-6);
    }

    #[test]
    fn srgb_lerp_midpoint() {
        let a = Srgb::new(0.0_f32, 0.0, 0.0);
        let b = Srgb::new(1.0, 1.0, 1.0);
        let mid = a.lerp(&b, 0.5);
        assert!((mid.red - 0.5).abs() < 1e-6);
    }

    #[test]
    fn lab_lerp_midpoint() {
        let a = Lab::new(0.0_f32, -50.0, -50.0);
        let b = Lab::new(100.0, 50.0, 50.0);
        let mid = a.lerp(&b, 0.5);
        assert!((mid.l - 50.0).abs() < 1e-4);
        assert!((mid.a - 0.0).abs() < 1e-4);
    }

    // ── Hue interpolation ───────────────────────────────────────────

    #[test]
    fn lerp_hue_shortest_arc() {
        // 350 → 10 should go through 0, not through 180
        let result = lerp_hue(350.0, 10.0, 0.5);
        assert!(
            (result - 0.0).abs() < 1e-4 || (result - 360.0).abs() < 1e-4,
            "Expected ~0 or ~360, got {result}"
        );
    }

    #[test]
    fn lerp_hue_normal() {
        let result = lerp_hue(0.0, 90.0, 0.5);
        assert!((result - 45.0).abs() < 1e-4);
    }

    #[test]
    fn lerp_hue_wrap_backward() {
        // 10 → 350 should go backward through 0
        let result = lerp_hue(10.0, 350.0, 0.5);
        assert!(
            (result - 0.0).abs() < 1e-4 || (result - 360.0).abs() < 1e-4,
            "Expected ~0 or ~360, got {result}"
        );
    }

    // ── InLab wrapper ───────────────────────────────────────────────

    #[test]
    fn in_lab_midpoint_differs_from_srgb() {
        let red = Srgba::new(1.0, 0.0, 0.0, 1.0);
        let cyan = Srgba::new(0.0, 1.0, 1.0, 1.0);
        let srgb_mid = red.lerp(&cyan, 0.5);
        let lab_mid = lerp_in_lab(red, cyan, 0.5);
        let diff = (srgb_mid.red - lab_mid.red).abs()
            + (srgb_mid.green - lab_mid.green).abs()
            + (srgb_mid.blue - lab_mid.blue).abs();
        assert!(diff > 0.01, "Lab midpoint should differ from sRGB: diff={diff}");
    }

    #[test]
    fn in_lab_endpoints_preserved() {
        let red = Srgba::new(1.0_f32, 0.0, 0.0, 1.0);
        let blue = Srgba::new(0.0, 0.0, 1.0, 1.0);
        let start = InLab(red).lerp(&InLab(blue), 0.0);
        assert!((start.0.red - 1.0).abs() < 1e-3);
        let end = InLab(red).lerp(&InLab(blue), 1.0);
        assert!((end.0.blue - 1.0).abs() < 1e-3);
    }

    // ── InOklch wrapper ─────────────────────────────────────────────

    #[test]
    fn in_oklch_midpoint_differs_from_srgb() {
        let red = Srgba::new(1.0, 0.0, 0.0, 1.0);
        let blue = Srgba::new(0.0, 0.0, 1.0, 1.0);
        let srgb_mid = red.lerp(&blue, 0.5);
        let oklch_mid = lerp_in_oklch(red, blue, 0.5);
        let diff = (srgb_mid.red - oklch_mid.red).abs()
            + (srgb_mid.green - oklch_mid.green).abs()
            + (srgb_mid.blue - oklch_mid.blue).abs();
        assert!(
            diff > 0.01,
            "Oklch midpoint should differ from sRGB: diff={diff}"
        );
    }

    // ── InLinear wrapper ────────────────────────────────────────────

    #[test]
    fn in_linear_endpoints_preserved() {
        let a = Srgba::new(1.0, 0.0, 0.0, 1.0);
        let b = Srgba::new(0.0, 1.0, 0.0, 1.0);
        let start = InLinear(a).lerp(&InLinear(b), 0.0);
        assert!((start.0.red - 1.0).abs() < 1e-3);
        let end = InLinear(a).lerp(&InLinear(b), 1.0);
        assert!((end.0.green - 1.0).abs() < 1e-3);
    }

    // ── Animatable blanket ──────────────────────────────────────────

    #[test]
    fn srgba_is_animatable() {
        fn needs_animatable<T: crate::traits::Animatable>(_: T) {}
        needs_animatable(Srgba::new(1.0_f32, 0.0, 0.0, 1.0));
    }

    #[test]
    fn in_lab_is_animatable() {
        fn needs_animatable<T: crate::traits::Animatable>(_: T) {}
        needs_animatable(InLab(Srgba::new(1.0_f32, 0.0, 0.0, 1.0)));
    }

    #[test]
    fn in_oklch_is_animatable() {
        fn needs_animatable<T: crate::traits::Animatable>(_: T) {}
        needs_animatable(InOklch(Srgba::new(1.0_f32, 0.0, 0.0, 1.0)));
    }

    // ── Tween integration ───────────────────────────────────────────

    #[test]
    fn tween_with_srgba() {
        use crate::traits::Update;
        use crate::tween::Tween;

        let mut t = Tween::new(
            Srgba::new(0.0, 0.0, 0.0, 1.0),
            Srgba::new(1.0, 1.0, 1.0, 1.0),
        )
        .duration(1.0)
        .build();
        t.update(0.5);
        let v = t.value();
        assert!((v.red - 0.5).abs() < 1e-3);
    }

    #[test]
    fn tween_with_in_lab() {
        use crate::traits::Update;
        use crate::tween::Tween;

        let mut t = Tween::new(
            InLab(Srgba::new(1.0, 0.0, 0.0, 1.0)),
            InLab(Srgba::new(0.0, 0.0, 1.0, 1.0)),
        )
        .duration(1.0)
        .build();
        t.update(0.5);
        let v = t.value();
        // Result from Lab space should be valid sRGB
        assert!(v.0.red >= 0.0 && v.0.red <= 1.0);
        assert!(v.0.green >= 0.0 && v.0.green <= 1.0);
        assert!(v.0.blue >= 0.0 && v.0.blue <= 1.0);
    }

    // ── SpringAnimatable roundtrip ──────────────────────────────────

    #[test]
    fn srgba_spring_animatable_roundtrip() {
        let c = Srgba::new(0.5, 0.3, 0.8, 1.0);
        let components = c.to_components();
        assert_eq!(components.len(), 4);
        let rebuilt = <Srgba as SpringAnimatable>::from_components(&components);
        assert!((rebuilt.red - 0.5).abs() < 1e-6);
        assert!((rebuilt.green - 0.3).abs() < 1e-6);
        assert!((rebuilt.blue - 0.8).abs() < 1e-6);
    }

    #[test]
    fn in_lab_spring_animatable_roundtrip() {
        let c = InLab(Srgba::new(0.2, 0.7, 0.4, 0.9));
        let components = c.to_components();
        assert_eq!(components.len(), 4);
        let rebuilt = InLab::from_components(&components);
        assert!((rebuilt.0.red - 0.2).abs() < 1e-6);
    }
}
