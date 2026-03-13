//! Easing functions — the *feel* of every animation.
//!
//! An easing function maps a linear progress value `t ∈ [0.0, 1.0]` to a
//! *curved* output, giving animations their character: a bounce, an elastic
//! snap, a smooth deceleration.
//!
//! # Usage
//!
//! ```rust
//! use spanda::easing::Easing;
//!
//! let t = 0.5_f32;
//! let curved = Easing::EaseOutBounce.apply(t);
//! ```
//!
//! You can also call the free functions directly for zero-overhead use:
//!
//! ```rust
//! use spanda::easing::ease_out_elastic;
//!
//! let t = ease_out_elastic(0.7);
//! ```
//!
//! # Cheat-sheet
//!
//! | Variant              | Character                                      |
//! |----------------------|------------------------------------------------|
//! | `Linear`             | Constant speed — robotic                       |
//! | `EaseInQuad`…`Quart` | Slow start, accelerate — `Quad` is subtle      |
//! | `EaseOutQuad`…       | Fast start, decelerate — great for exits       |
//! | `EaseInOutQuad`…     | Smooth in *and* out — use for UI elements      |
//! | `EaseInOutCubic`     | The most natural-feeling general purpose ease  |
//! | `EaseInBack`         | Slight recoil before moving — playful          |
//! | `EaseOutBack`        | Overshoots, settles — satisfying confirmations |
//! | `EaseInElastic`      | Wind-up spring — dramatic entrance             |
//! | `EaseOutElastic`     | Release spring — poppy, energetic              |
//! | `EaseOutBounce`      | Ball bouncing to rest — game UIs               |
//! | `EaseInOutBounce`    | Bounce both ends — very expressive             |
//! | `Custom(fn)`         | Your own curve                                 |

use core::f32::consts::PI;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// ── Constants ─────────────────────────────────────────────────────────────────

/// Overshoot constant used by Back easing.  Controls how far the animation
/// pulls back before launching.  1.70158 is the standard value (~10% back).
const BACK_C1: f32 = 1.70158;
const BACK_C2: f32 = BACK_C1 * 1.525;
const BACK_C3: f32 = BACK_C1 + 1.0;

/// Elastic easing constants (period and amplitude for standard feel).
const ELASTIC_C4: f32 = (2.0 * PI) / 3.0;
const ELASTIC_C5: f32 = (2.0 * PI) / 4.5;

// ── Easing enum ───────────────────────────────────────────────────────────────

/// All built-in easing curves plus a `Custom` escape hatch.
///
/// Use [`Easing::apply`] to evaluate any variant at a given `t`.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Easing {
    // ── Linear ────────────────────────────────────────────────────────────────
    Linear,

    // ── Polynomial ────────────────────────────────────────────────────────────
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,

    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,

    EaseInQuart,
    EaseOutQuart,
    EaseInOutQuart,

    EaseInQuint,
    EaseOutQuint,
    EaseInOutQuint,

    // ── Sinusoidal ────────────────────────────────────────────────────────────
    EaseInSine,
    EaseOutSine,
    EaseInOutSine,

    // ── Exponential ───────────────────────────────────────────────────────────
    EaseInExpo,
    EaseOutExpo,
    EaseInOutExpo,

    // ── Circular ──────────────────────────────────────────────────────────────
    EaseInCirc,
    EaseOutCirc,
    EaseInOutCirc,

    // ── Back (overshoot) ──────────────────────────────────────────────────────
    EaseInBack,
    EaseOutBack,
    EaseInOutBack,

    // ── Elastic ───────────────────────────────────────────────────────────────
    EaseInElastic,
    EaseOutElastic,
    EaseInOutElastic,

    // ── Bounce ────────────────────────────────────────────────────────────────
    EaseInBounce,
    EaseOutBounce,
    EaseInOutBounce,

    // ── Custom escape hatch ───────────────────────────────────────────────────
    /// Provide your own easing function.  Not serialisable — if you need
    /// serde support, convert to a named variant or store the name separately.
    #[cfg_attr(feature = "serde", serde(skip))]
    Custom(fn(f32) -> f32),
}

impl Easing {
    /// Evaluate the easing curve at `t ∈ [0.0, 1.0]`.
    ///
    /// Input is clamped to `[0.0, 1.0]` before evaluation so you never need to
    /// guard the caller.
    #[inline]
    pub fn apply(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Linear          => linear(t),

            Self::EaseInQuad      => ease_in_quad(t),
            Self::EaseOutQuad     => ease_out_quad(t),
            Self::EaseInOutQuad   => ease_in_out_quad(t),

            Self::EaseInCubic     => ease_in_cubic(t),
            Self::EaseOutCubic    => ease_out_cubic(t),
            Self::EaseInOutCubic  => ease_in_out_cubic(t),

            Self::EaseInQuart     => ease_in_quart(t),
            Self::EaseOutQuart    => ease_out_quart(t),
            Self::EaseInOutQuart  => ease_in_out_quart(t),

            Self::EaseInQuint     => ease_in_quint(t),
            Self::EaseOutQuint    => ease_out_quint(t),
            Self::EaseInOutQuint  => ease_in_out_quint(t),

            Self::EaseInSine      => ease_in_sine(t),
            Self::EaseOutSine     => ease_out_sine(t),
            Self::EaseInOutSine   => ease_in_out_sine(t),

            Self::EaseInExpo      => ease_in_expo(t),
            Self::EaseOutExpo     => ease_out_expo(t),
            Self::EaseInOutExpo   => ease_in_out_expo(t),

            Self::EaseInCirc      => ease_in_circ(t),
            Self::EaseOutCirc     => ease_out_circ(t),
            Self::EaseInOutCirc   => ease_in_out_circ(t),

            Self::EaseInBack      => ease_in_back(t),
            Self::EaseOutBack     => ease_out_back(t),
            Self::EaseInOutBack   => ease_in_out_back(t),

            Self::EaseInElastic   => ease_in_elastic(t),
            Self::EaseOutElastic  => ease_out_elastic(t),
            Self::EaseInOutElastic => ease_in_out_elastic(t),

            Self::EaseInBounce    => ease_in_bounce(t),
            Self::EaseOutBounce   => ease_out_bounce(t),
            Self::EaseInOutBounce => ease_in_out_bounce(t),

            Self::Custom(f)       => f(t),
        }
    }
}

// Provide a sensible Debug even though fn pointers aren't fmt::Debug.
impl core::fmt::Debug for Easing {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Custom(_) => write!(f, "Easing::Custom(<fn>)"),
            _ => write!(f, "Easing::{}", self.name()),
        }
    }
}

impl PartialEq for Easing {
    fn eq(&self, other: &Self) -> bool {
        // Custom variants compare by function pointer address.
        match (self, other) {
            (Self::Custom(a), Self::Custom(b)) => (*a as usize) == (*b as usize),
            _ => self.name() == other.name(),
        }
    }
}

impl Easing {
    /// Human-readable name — useful for debug UIs and serialisation.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Linear           => "Linear",
            Self::EaseInQuad       => "EaseInQuad",
            Self::EaseOutQuad      => "EaseOutQuad",
            Self::EaseInOutQuad    => "EaseInOutQuad",
            Self::EaseInCubic      => "EaseInCubic",
            Self::EaseOutCubic     => "EaseOutCubic",
            Self::EaseInOutCubic   => "EaseInOutCubic",
            Self::EaseInQuart      => "EaseInQuart",
            Self::EaseOutQuart     => "EaseOutQuart",
            Self::EaseInOutQuart   => "EaseInOutQuart",
            Self::EaseInQuint      => "EaseInQuint",
            Self::EaseOutQuint     => "EaseOutQuint",
            Self::EaseInOutQuint   => "EaseInOutQuint",
            Self::EaseInSine       => "EaseInSine",
            Self::EaseOutSine      => "EaseOutSine",
            Self::EaseInOutSine    => "EaseInOutSine",
            Self::EaseInExpo       => "EaseInExpo",
            Self::EaseOutExpo      => "EaseOutExpo",
            Self::EaseInOutExpo    => "EaseInOutExpo",
            Self::EaseInCirc       => "EaseInCirc",
            Self::EaseOutCirc      => "EaseOutCirc",
            Self::EaseInOutCirc    => "EaseInOutCirc",
            Self::EaseInBack       => "EaseInBack",
            Self::EaseOutBack      => "EaseOutBack",
            Self::EaseInOutBack    => "EaseInOutBack",
            Self::EaseInElastic    => "EaseInElastic",
            Self::EaseOutElastic   => "EaseOutElastic",
            Self::EaseInOutElastic => "EaseInOutElastic",
            Self::EaseInBounce     => "EaseInBounce",
            Self::EaseOutBounce    => "EaseOutBounce",
            Self::EaseInOutBounce  => "EaseInOutBounce",
            Self::Custom(_)        => "Custom",
        }
    }

    /// All named (non-Custom) variants — useful for building picker UIs or
    /// running benchmark sweeps.
    pub fn all_named() -> &'static [Easing] {
        &[
            Self::Linear,
            Self::EaseInQuad,     Self::EaseOutQuad,     Self::EaseInOutQuad,
            Self::EaseInCubic,    Self::EaseOutCubic,    Self::EaseInOutCubic,
            Self::EaseInQuart,    Self::EaseOutQuart,    Self::EaseInOutQuart,
            Self::EaseInQuint,    Self::EaseOutQuint,    Self::EaseInOutQuint,
            Self::EaseInSine,     Self::EaseOutSine,     Self::EaseInOutSine,
            Self::EaseInExpo,     Self::EaseOutExpo,     Self::EaseInOutExpo,
            Self::EaseInCirc,     Self::EaseOutCirc,     Self::EaseInOutCirc,
            Self::EaseInBack,     Self::EaseOutBack,     Self::EaseInOutBack,
            Self::EaseInElastic,  Self::EaseOutElastic,  Self::EaseInOutElastic,
            Self::EaseInBounce,   Self::EaseOutBounce,   Self::EaseInOutBounce,
        ]
    }
}

// ── Pure easing functions (pub — zero-cost for direct calls) ─────────────────

#[inline] pub fn linear(t: f32) -> f32 { t }

// ── Quad ──────────────────────────────────────────────────────────────────────

#[inline] pub fn ease_in_quad(t: f32) -> f32  { t * t }
#[inline] pub fn ease_out_quad(t: f32) -> f32 { 1.0 - (1.0 - t) * (1.0 - t) }
#[inline] pub fn ease_in_out_quad(t: f32) -> f32 {
    if t < 0.5 { 2.0 * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(2) / 2.0 }
}

// ── Cubic ─────────────────────────────────────────────────────────────────────

#[inline] pub fn ease_in_cubic(t: f32) -> f32  { t * t * t }
#[inline] pub fn ease_out_cubic(t: f32) -> f32 { 1.0 - (1.0 - t).powi(3) }
#[inline] pub fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 { 4.0 * t * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(3) / 2.0 }
}

// ── Quart ─────────────────────────────────────────────────────────────────────

#[inline] pub fn ease_in_quart(t: f32) -> f32  { t * t * t * t }
#[inline] pub fn ease_out_quart(t: f32) -> f32 { 1.0 - (1.0 - t).powi(4) }
#[inline] pub fn ease_in_out_quart(t: f32) -> f32 {
    if t < 0.5 { 8.0 * t * t * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(4) / 2.0 }
}

// ── Quint ─────────────────────────────────────────────────────────────────────

#[inline] pub fn ease_in_quint(t: f32) -> f32  { t * t * t * t * t }
#[inline] pub fn ease_out_quint(t: f32) -> f32 { 1.0 - (1.0 - t).powi(5) }
#[inline] pub fn ease_in_out_quint(t: f32) -> f32 {
    if t < 0.5 { 16.0 * t * t * t * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(5) / 2.0 }
}

// ── Sine ──────────────────────────────────────────────────────────────────────

#[inline] pub fn ease_in_sine(t: f32) -> f32    { 1.0 - ((t * PI / 2.0).cos()) }
#[inline] pub fn ease_out_sine(t: f32) -> f32   { (t * PI / 2.0).sin() }
#[inline] pub fn ease_in_out_sine(t: f32) -> f32 { -(((PI * t).cos()) - 1.0) / 2.0 }

// ── Expo ──────────────────────────────────────────────────────────────────────

#[inline] pub fn ease_in_expo(t: f32) -> f32 {
    if t == 0.0 { 0.0 } else { (2.0_f32).powf(10.0 * t - 10.0) }
}
#[inline] pub fn ease_out_expo(t: f32) -> f32 {
    if t == 1.0 { 1.0 } else { 1.0 - (2.0_f32).powf(-10.0 * t) }
}
#[inline] pub fn ease_in_out_expo(t: f32) -> f32 {
    if t == 0.0 { return 0.0; }
    if t == 1.0 { return 1.0; }
    if t < 0.5 {
        (2.0_f32).powf(20.0 * t - 10.0) / 2.0
    } else {
        (2.0 - (2.0_f32).powf(-20.0 * t + 10.0)) / 2.0
    }
}

// ── Circ ──────────────────────────────────────────────────────────────────────

#[inline] pub fn ease_in_circ(t: f32) -> f32  { 1.0 - (1.0 - t * t).sqrt() }
#[inline] pub fn ease_out_circ(t: f32) -> f32 { (1.0 - (t - 1.0).powi(2)).sqrt() }
#[inline] pub fn ease_in_out_circ(t: f32) -> f32 {
    if t < 0.5 {
        (1.0 - (1.0 - (2.0 * t).powi(2)).sqrt()) / 2.0
    } else {
        ((1.0 - (-2.0 * t + 2.0).powi(2)).sqrt() + 1.0) / 2.0
    }
}

// ── Back ──────────────────────────────────────────────────────────────────────

#[inline] pub fn ease_in_back(t: f32) -> f32 {
    BACK_C3 * t * t * t - BACK_C1 * t * t
}
#[inline] pub fn ease_out_back(t: f32) -> f32 {
    let t = t - 1.0;
    1.0 + BACK_C3 * t * t * t + BACK_C1 * t * t
}
#[inline] pub fn ease_in_out_back(t: f32) -> f32 {
    if t < 0.5 {
        ((2.0 * t).powi(2) * ((BACK_C2 + 1.0) * 2.0 * t - BACK_C2)) / 2.0
    } else {
        ((2.0 * t - 2.0).powi(2) * ((BACK_C2 + 1.0) * (t * 2.0 - 2.0) + BACK_C2) + 2.0) / 2.0
    }
}

// ── Elastic ───────────────────────────────────────────────────────────────────

#[inline] pub fn ease_in_elastic(t: f32) -> f32 {
    if t == 0.0 { return 0.0; }
    if t == 1.0 { return 1.0; }
    -(2.0_f32).powf(10.0 * t - 10.0) * ((10.0 * t - 10.75) * ELASTIC_C4).sin()
}
#[inline] pub fn ease_out_elastic(t: f32) -> f32 {
    if t == 0.0 { return 0.0; }
    if t == 1.0 { return 1.0; }
    (2.0_f32).powf(-10.0 * t) * ((10.0 * t - 0.75) * ELASTIC_C4).sin() + 1.0
}
#[inline] pub fn ease_in_out_elastic(t: f32) -> f32 {
    if t == 0.0 { return 0.0; }
    if t == 1.0 { return 1.0; }
    if t < 0.5 {
        -((2.0_f32).powf(20.0 * t - 10.0) * ((20.0 * t - 11.125) * ELASTIC_C5).sin()) / 2.0
    } else {
        (2.0_f32).powf(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * ELASTIC_C5).sin() / 2.0 + 1.0
    }
}

// ── Bounce ────────────────────────────────────────────────────────────────────

#[inline] pub fn ease_out_bounce(t: f32) -> f32 {
    const N: f32 = 7.5625;
    const D: f32 = 2.75;
    if t < 1.0 / D {
        N * t * t
    } else if t < 2.0 / D {
        let t = t - 1.5 / D;
        N * t * t + 0.75
    } else if t < 2.5 / D {
        let t = t - 2.25 / D;
        N * t * t + 0.9375
    } else {
        let t = t - 2.625 / D;
        N * t * t + 0.984375
    }
}

#[inline] pub fn ease_in_bounce(t: f32) -> f32 {
    1.0 - ease_out_bounce(1.0 - t)
}

#[inline] pub fn ease_in_out_bounce(t: f32) -> f32 {
    if t < 0.5 {
        (1.0 - ease_out_bounce(1.0 - 2.0 * t)) / 2.0
    } else {
        (1.0 + ease_out_bounce(2.0 * t - 1.0)) / 2.0
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Every easing must map 0 → 0 and 1 → 1 (within float tolerance).
    #[test]
    fn endpoints_for_all_named() {
        for easing in Easing::all_named() {
            let t0 = easing.apply(0.0);
            let t1 = easing.apply(1.0);
            assert!(
                t0.abs() < 1e-5,
                "{} apply(0.0) = {} (expected ~0.0)", easing.name(), t0
            );
            assert!(
                (t1 - 1.0).abs() < 1e-5,
                "{} apply(1.0) = {} (expected ~1.0)", easing.name(), t1
            );
        }
    }

    /// Input clamping — values outside [0,1] must not panic.
    #[test]
    fn clamping_does_not_panic() {
        for easing in Easing::all_named() {
            let _ = easing.apply(-0.5);
            let _ = easing.apply(1.5);
        }
    }

    #[test]
    fn linear_is_identity() {
        for i in 0..=10 {
            let t = i as f32 / 10.0;
            assert!((linear(t) - t).abs() < 1e-7, "linear({t}) != {t}");
        }
    }

    #[test]
    fn ease_in_out_cubic_is_symmetric() {
        for i in 1..10 {
            let t = i as f32 / 10.0;
            let forward  = ease_in_out_cubic(t);
            let backward = 1.0 - ease_in_out_cubic(1.0 - t);
            assert!(
                (forward - backward).abs() < 1e-6,
                "EaseInOutCubic symmetry broken at t={t}: {forward} vs {backward}"
            );
        }
    }

    #[test]
    fn bounce_out_midpoint_is_above_half() {
        // Bounce curves accelerate — midpoint should be well above 0.5.
        assert!(ease_out_bounce(0.5) > 0.5);
    }

    #[test]
    fn custom_easing_works() {
        let e = Easing::Custom(|t| t * t);
        assert!((e.apply(0.5) - 0.25).abs() < 1e-7);
    }

    #[test]
    fn easing_enum_name_roundtrip() {
        for e in Easing::all_named() {
            assert!(!e.name().is_empty());
        }
    }
}
