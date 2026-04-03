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

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

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
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Easing {
    // ── Linear ────────────────────────────────────────────────────────────────
    /// Constant speed — no acceleration or deceleration.
    Linear,

    // ── Polynomial ────────────────────────────────────────────────────────────
    /// Quadratic ease-in — slow start, accelerates.
    EaseInQuad,
    /// Quadratic ease-out — fast start, decelerates.
    EaseOutQuad,
    /// Quadratic ease-in-out — smooth acceleration and deceleration.
    EaseInOutQuad,

    /// Cubic ease-in — slow start with moderate acceleration.
    EaseInCubic,
    /// Cubic ease-out — fast start with moderate deceleration.
    EaseOutCubic,
    /// Cubic ease-in-out — the most natural-feeling general purpose ease.
    EaseInOutCubic,

    /// Quartic ease-in — very slow start, sharp acceleration.
    EaseInQuart,
    /// Quartic ease-out — fast start, sharp deceleration.
    EaseOutQuart,
    /// Quartic ease-in-out — pronounced slow-fast-slow.
    EaseInOutQuart,

    /// Quintic ease-in — extreme slow start.
    EaseInQuint,
    /// Quintic ease-out — extreme fast start.
    EaseOutQuint,
    /// Quintic ease-in-out — very dramatic slow-fast-slow.
    EaseInOutQuint,

    // ── Sinusoidal ────────────────────────────────────────────────────────────
    /// Sinusoidal ease-in — gentle, subtle acceleration.
    EaseInSine,
    /// Sinusoidal ease-out — gentle, subtle deceleration.
    EaseOutSine,
    /// Sinusoidal ease-in-out — smooth and natural.
    EaseInOutSine,

    // ── Exponential ───────────────────────────────────────────────────────────
    /// Exponential ease-in — nearly frozen, then sudden burst.
    EaseInExpo,
    /// Exponential ease-out — rapid start, gradually stops.
    EaseOutExpo,
    /// Exponential ease-in-out — extreme contrast between slow and fast.
    EaseInOutExpo,

    // ── Circular ──────────────────────────────────────────────────────────────
    /// Circular ease-in — arc-shaped slow start.
    EaseInCirc,
    /// Circular ease-out — arc-shaped fast start.
    EaseOutCirc,
    /// Circular ease-in-out — arc-shaped acceleration and deceleration.
    EaseInOutCirc,

    // ── Back (overshoot) ──────────────────────────────────────────────────────
    /// Back ease-in — pulls back slightly before accelerating forward.
    EaseInBack,
    /// Back ease-out — overshoots the target, then settles back.
    EaseOutBack,
    /// Back ease-in-out — pulls back at start, overshoots at end.
    EaseInOutBack,

    // ── Elastic ───────────────────────────────────────────────────────────────
    /// Elastic ease-in — wind-up spring effect before launching.
    EaseInElastic,
    /// Elastic ease-out — spring release with oscillating overshoot.
    EaseOutElastic,
    /// Elastic ease-in-out — spring wind-up and release both ends.
    EaseInOutElastic,

    // ── Bounce ────────────────────────────────────────────────────────────────
    /// Bounce ease-in — bounces at the start before settling in.
    EaseInBounce,
    /// Bounce ease-out — ball bouncing to rest.
    EaseOutBounce,
    /// Bounce ease-in-out — bounces at both start and end.
    EaseInOutBounce,

    // ── Custom escape hatch ───────────────────────────────────────────────────
    /// Provide your own easing function.  Not serialisable — if you need
    /// serde support, convert to a named variant or store the name separately.
    #[cfg_attr(feature = "serde", serde(skip))]
    Custom(fn(f32) -> f32),

    // ── CSS-compatible timing functions ──────────────────────────────────────
    /// CSS `cubic-bezier(x1, y1, x2, y2)` easing.
    ///
    /// The four values are the two control points of a cubic Bezier on the
    /// unit square. Endpoints are implicitly `(0,0)` and `(1,1)`.
    ///
    /// ```rust
    /// use spanda::easing::Easing;
    ///
    /// // CSS "ease" equivalent: cubic-bezier(0.25, 0.1, 0.25, 1.0)
    /// let ease = Easing::CubicBezier(0.25, 0.1, 0.25, 1.0);
    /// let val = ease.apply(0.5);
    /// assert!(val > 0.5); // accelerates then decelerates
    /// ```
    CubicBezier(f32, f32, f32, f32),

    /// CSS `steps(n)` easing — snaps progress to `n` discrete jumps.
    ///
    /// ```rust
    /// use spanda::easing::Easing;
    ///
    /// let stepped = Easing::Steps(4);
    /// assert!((stepped.apply(0.0) - 0.0).abs() < 1e-6);
    /// assert!((stepped.apply(0.3) - 0.25).abs() < 1e-6);
    /// assert!((stepped.apply(1.0) - 1.0).abs() < 1e-6);
    /// ```
    Steps(u32),

    // ── Advanced parameterized easings (v0.8) ──────────────────────────────
    /// Deterministic rough/jittery easing — noise overlaid on a linear curve.
    /// `strength` controls deviation amplitude, `points` is noise sample count,
    /// `seed` ensures reproducibility.
    RoughEase {
        /// Deviation amplitude (0..1).
        strength: f32,
        /// Number of noise sample points.
        points: u32,
        /// PRNG seed for reproducibility.
        seed: u32,
    },

    /// Slow-fast-slow piecewise easing for dramatic effects.
    /// `ratio` (0..1) is the fraction of the curve that is "slow" on each side.
    /// `power` controls steepness. `yoyo_mode` mirrors the shape.
    SlowMo {
        /// Fraction of the curve that is "slow" on each side (0..1).
        ratio: f32,
        /// Steepness of the slow sections.
        power: f32,
        /// If true, the curve is mirrored.
        yoyo_mode: bool,
    },

    /// Perceptual exponential scale correction.
    /// Corrects non-linearity where `1→2` looks different from `2→4`.
    ExpoScale {
        /// Starting scale value.
        start_scale: f32,
        /// Ending scale value.
        end_scale: f32,
    },

    /// Sinusoidal wiggle — oscillates around the base curve.
    /// `frequency` is the number of full oscillations, `amplitude` is max deviation.
    Wiggle {
        /// Number of full oscillations.
        frequency: f32,
        /// Maximum deviation from linear base (0..1).
        amplitude: f32,
    },

    /// Parametric bounce with configurable strength and squash.
    /// `strength` (0..1) controls bounce height decay.
    /// `squash` (≥0) controls horizontal compression at each bounce.
    CustomBounce {
        /// Bounce height decay factor (0..1).
        strength: f32,
        /// Horizontal compression per bounce (≥0).
        squash: f32,
    },
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

            Self::CubicBezier(x1, y1, x2, y2) => cubic_bezier_ease(t, *x1, *y1, *x2, *y2),
            Self::Steps(n)        => steps_ease(t, *n),

            Self::RoughEase { strength, points, seed } =>
                rough_ease(t, *strength, *points, *seed),
            Self::SlowMo { ratio, power, yoyo_mode } =>
                slow_mo(t, *ratio, *power, *yoyo_mode),
            Self::ExpoScale { start_scale, end_scale } =>
                expo_scale(t, *start_scale, *end_scale),
            Self::Wiggle { frequency, amplitude } =>
                wiggle_ease(t, *frequency, *amplitude),
            Self::CustomBounce { strength, squash } =>
                custom_bounce(t, *strength, *squash),
        }
    }
}

// Provide a sensible Debug even though fn pointers aren't fmt::Debug.
impl core::fmt::Debug for Easing {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Custom(_) => write!(f, "Easing::Custom(<fn>)"),
            Self::CubicBezier(x1, y1, x2, y2) => {
                write!(f, "Easing::CubicBezier({x1}, {y1}, {x2}, {y2})")
            }
            Self::Steps(n) => write!(f, "Easing::Steps({n})"),
            Self::RoughEase { strength, points, seed } => {
                write!(f, "Easing::RoughEase({strength}, {points}, {seed})")
            }
            Self::SlowMo { ratio, power, yoyo_mode } => {
                write!(f, "Easing::SlowMo({ratio}, {power}, {yoyo_mode})")
            }
            Self::ExpoScale { start_scale, end_scale } => {
                write!(f, "Easing::ExpoScale({start_scale}, {end_scale})")
            }
            Self::Wiggle { frequency, amplitude } => {
                write!(f, "Easing::Wiggle({frequency}, {amplitude})")
            }
            Self::CustomBounce { strength, squash } => {
                write!(f, "Easing::CustomBounce({strength}, {squash})")
            }
            _ => write!(f, "Easing::{}", self.name()),
        }
    }
}

impl PartialEq for Easing {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // Function pointer identity comparison is unreliable across
            // compilation units, so Custom variants are never equal to each other.
            (Self::Custom(_), Self::Custom(_)) => false,
            (Self::CubicBezier(x1, y1, x2, y2), Self::CubicBezier(a1, b1, a2, b2)) => {
                x1 == a1 && y1 == b1 && x2 == a2 && y2 == b2
            }
            (Self::Steps(a), Self::Steps(b)) => a == b,
            (Self::RoughEase { strength: s1, points: p1, seed: d1 },
             Self::RoughEase { strength: s2, points: p2, seed: d2 }) =>
                s1 == s2 && p1 == p2 && d1 == d2,
            (Self::SlowMo { ratio: r1, power: p1, yoyo_mode: y1 },
             Self::SlowMo { ratio: r2, power: p2, yoyo_mode: y2 }) =>
                r1 == r2 && p1 == p2 && y1 == y2,
            (Self::ExpoScale { start_scale: s1, end_scale: e1 },
             Self::ExpoScale { start_scale: s2, end_scale: e2 }) =>
                s1 == s2 && e1 == e2,
            (Self::Wiggle { frequency: f1, amplitude: a1 },
             Self::Wiggle { frequency: f2, amplitude: a2 }) =>
                f1 == f2 && a1 == a2,
            (Self::CustomBounce { strength: s1, squash: q1 },
             Self::CustomBounce { strength: s2, squash: q2 }) =>
                s1 == s2 && q1 == q2,
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
            Self::CubicBezier(..) => "CubicBezier",
            Self::Steps(_)        => "Steps",
            Self::RoughEase { .. } => "RoughEase",
            Self::SlowMo { .. }    => "SlowMo",
            Self::ExpoScale { .. } => "ExpoScale",
            Self::Wiggle { .. }    => "Wiggle",
            Self::CustomBounce { .. } => "CustomBounce",
        }
    }

    /// All 31 classic (non-parameterized) easing variants.
    ///
    /// Excludes `Custom`, `CubicBezier`, `Steps`, and advanced parameterized
    /// easings. Useful for building picker UIs or running benchmark sweeps.
    pub fn all_classic() -> &'static [Easing] {
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

    /// All named (non-Custom) variants including parameterized easings with
    /// representative default parameters.
    ///
    /// Useful for exhaustive testing. For UI pickers, prefer [`all_classic()`].
    pub fn all_named() -> Vec<Easing> {
        let mut v: Vec<Easing> = Self::all_classic().to_vec();
        v.extend_from_slice(&[
            Self::CubicBezier(0.25, 0.1, 0.25, 1.0),
            Self::Steps(4),
            Self::RoughEase { strength: 0.5, points: 20, seed: 42 },
            Self::SlowMo { ratio: 0.7, power: 0.7, yoyo_mode: false },
            Self::ExpoScale { start_scale: 1.0, end_scale: 10.0 },
            Self::Wiggle { frequency: 3.0, amplitude: 0.3 },
            Self::CustomBounce { strength: 0.5, squash: 0.0 },
        ]);
        v
    }
}

// ── Pure easing functions (pub — zero-cost for direct calls) ─────────────────

/// Linear easing — identity function, constant speed.
#[inline] pub fn linear(t: f32) -> f32 { t }

// ── Quad ──────────────────────────────────────────────────────────────────────

/// Quadratic ease-in: `t²`.
#[inline] pub fn ease_in_quad(t: f32) -> f32  { t * t }
/// Quadratic ease-out: `1 - (1-t)²`.
#[inline] pub fn ease_out_quad(t: f32) -> f32 { 1.0 - (1.0 - t) * (1.0 - t) }
/// Quadratic ease-in-out: smooth acceleration then deceleration.
#[inline] pub fn ease_in_out_quad(t: f32) -> f32 {
    if t < 0.5 { 2.0 * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(2) / 2.0 }
}

// ── Cubic ─────────────────────────────────────────────────────────────────────

/// Cubic ease-in: `t³`.
#[inline] pub fn ease_in_cubic(t: f32) -> f32  { t * t * t }
/// Cubic ease-out: `1 - (1-t)³`.
#[inline] pub fn ease_out_cubic(t: f32) -> f32 { 1.0 - (1.0 - t).powi(3) }
/// Cubic ease-in-out: smooth acceleration then deceleration.
#[inline] pub fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 { 4.0 * t * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(3) / 2.0 }
}

// ── Quart ─────────────────────────────────────────────────────────────────────

/// Quartic ease-in: `t⁴`.
#[inline] pub fn ease_in_quart(t: f32) -> f32  { t * t * t * t }
/// Quartic ease-out: `1 - (1-t)⁴`.
#[inline] pub fn ease_out_quart(t: f32) -> f32 { 1.0 - (1.0 - t).powi(4) }
/// Quartic ease-in-out: pronounced slow-fast-slow.
#[inline] pub fn ease_in_out_quart(t: f32) -> f32 {
    if t < 0.5 { 8.0 * t * t * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(4) / 2.0 }
}

// ── Quint ─────────────────────────────────────────────────────────────────────

/// Quintic ease-in: `t⁵`.
#[inline] pub fn ease_in_quint(t: f32) -> f32  { t * t * t * t * t }
/// Quintic ease-out: `1 - (1-t)⁵`.
#[inline] pub fn ease_out_quint(t: f32) -> f32 { 1.0 - (1.0 - t).powi(5) }
/// Quintic ease-in-out: very dramatic slow-fast-slow.
#[inline] pub fn ease_in_out_quint(t: f32) -> f32 {
    if t < 0.5 { 16.0 * t * t * t * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(5) / 2.0 }
}

// ── Sine ──────────────────────────────────────────────────────────────────────

/// Sinusoidal ease-in: gentle acceleration using a sine curve.
#[inline] pub fn ease_in_sine(t: f32) -> f32    { 1.0 - ((t * PI / 2.0).cos()) }
/// Sinusoidal ease-out: gentle deceleration using a sine curve.
#[inline] pub fn ease_out_sine(t: f32) -> f32   { (t * PI / 2.0).sin() }
/// Sinusoidal ease-in-out: smooth and natural sine-based curve.
#[inline] pub fn ease_in_out_sine(t: f32) -> f32 { -(((PI * t).cos()) - 1.0) / 2.0 }

// ── Expo ──────────────────────────────────────────────────────────────────────

/// Exponential ease-in: nearly frozen then sudden burst.
#[inline] pub fn ease_in_expo(t: f32) -> f32 {
    if t == 0.0 { 0.0 } else { (2.0_f32).powf(10.0 * t - 10.0) }
}
/// Exponential ease-out: rapid start, gradually stops.
#[inline] pub fn ease_out_expo(t: f32) -> f32 {
    if t == 1.0 { 1.0 } else { 1.0 - (2.0_f32).powf(-10.0 * t) }
}
/// Exponential ease-in-out: extreme contrast between slow and fast.
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

/// Circular ease-in: arc-shaped slow start.
#[inline] pub fn ease_in_circ(t: f32) -> f32  { 1.0 - (1.0 - t * t).sqrt() }
/// Circular ease-out: arc-shaped fast start.
#[inline] pub fn ease_out_circ(t: f32) -> f32 { (1.0 - (t - 1.0).powi(2)).sqrt() }
/// Circular ease-in-out: arc-shaped acceleration and deceleration.
#[inline] pub fn ease_in_out_circ(t: f32) -> f32 {
    if t < 0.5 {
        (1.0 - (1.0 - (2.0 * t).powi(2)).sqrt()) / 2.0
    } else {
        ((1.0 - (-2.0 * t + 2.0).powi(2)).sqrt() + 1.0) / 2.0
    }
}

// ── Back ──────────────────────────────────────────────────────────────────────

/// Back ease-in: pulls back slightly before accelerating forward.
#[inline] pub fn ease_in_back(t: f32) -> f32 {
    BACK_C3 * t * t * t - BACK_C1 * t * t
}
/// Back ease-out: overshoots the target, then settles back.
#[inline] pub fn ease_out_back(t: f32) -> f32 {
    let t = t - 1.0;
    1.0 + BACK_C3 * t * t * t + BACK_C1 * t * t
}
/// Back ease-in-out: pulls back at start, overshoots at end.
#[inline] pub fn ease_in_out_back(t: f32) -> f32 {
    if t < 0.5 {
        ((2.0 * t).powi(2) * ((BACK_C2 + 1.0) * 2.0 * t - BACK_C2)) / 2.0
    } else {
        ((2.0 * t - 2.0).powi(2) * ((BACK_C2 + 1.0) * (t * 2.0 - 2.0) + BACK_C2) + 2.0) / 2.0
    }
}

// ── Elastic ───────────────────────────────────────────────────────────────────

/// Elastic ease-in: wind-up spring effect before launching.
#[inline] pub fn ease_in_elastic(t: f32) -> f32 {
    if t == 0.0 { return 0.0; }
    if t == 1.0 { return 1.0; }
    -(2.0_f32).powf(10.0 * t - 10.0) * ((10.0 * t - 10.75) * ELASTIC_C4).sin()
}
/// Elastic ease-out: spring release with oscillating overshoot.
#[inline] pub fn ease_out_elastic(t: f32) -> f32 {
    if t == 0.0 { return 0.0; }
    if t == 1.0 { return 1.0; }
    (2.0_f32).powf(-10.0 * t) * ((10.0 * t - 0.75) * ELASTIC_C4).sin() + 1.0
}
/// Elastic ease-in-out: spring wind-up and release both ends.
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

/// Bounce ease-out: ball bouncing to rest.
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

/// Bounce ease-in: bounces at the start before settling in.
#[inline] pub fn ease_in_bounce(t: f32) -> f32 {
    1.0 - ease_out_bounce(1.0 - t)
}

/// Bounce ease-in-out: bounces at both start and end.
#[inline] pub fn ease_in_out_bounce(t: f32) -> f32 {
    if t < 0.5 {
        (1.0 - ease_out_bounce(1.0 - 2.0 * t)) / 2.0
    } else {
        (1.0 + ease_out_bounce(2.0 * t - 1.0)) / 2.0
    }
}

// ── CSS cubic-bezier() ──────────────────────────────────────────────────────

/// Evaluate a CSS `cubic-bezier(x1, y1, x2, y2)` easing at progress `t`.
///
/// Uses Newton's method to solve for the parametric `u` where `x(u) = t`,
/// then returns `y(u)`.  This is the same algorithm browsers use for CSS
/// transitions.
#[inline]
pub fn cubic_bezier_ease(t: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    if t <= 0.0 {
        return 0.0;
    }
    if t >= 1.0 {
        return 1.0;
    }

    // Find parameter u such that bezier_x(u) = t using Newton-Raphson.
    let mut u = t; // initial guess
    for _ in 0..8 {
        let x = sample_bezier(u, x1, x2) - t;
        let dx = sample_bezier_derivative(u, x1, x2);
        if dx.abs() < 1e-10 {
            break;
        }
        u -= x / dx;
        u = u.clamp(0.0, 1.0);
    }

    // Refine with bisection if Newton didn't converge close enough
    let mut x_val = sample_bezier(u, x1, x2) - t;
    if x_val.abs() > 1e-6 {
        let mut lo = 0.0_f32;
        let mut hi = 1.0_f32;
        u = t;
        for _ in 0..20 {
            x_val = sample_bezier(u, x1, x2) - t;
            if x_val.abs() < 1e-7 {
                break;
            }
            if x_val > 0.0 {
                hi = u;
            } else {
                lo = u;
            }
            u = (lo + hi) * 0.5;
        }
    }

    sample_bezier(u, y1, y2)
}

/// Sample the X or Y axis of a 1D cubic Bezier with control points at
/// `c1` and `c2`, implicit start 0 and end 1.
#[inline]
fn sample_bezier(u: f32, c1: f32, c2: f32) -> f32 {
    // B(u) = 3(1-u)²u·c1 + 3(1-u)u²·c2 + u³
    let u2 = u * u;
    let u3 = u2 * u;
    let inv = 1.0 - u;
    let inv2 = inv * inv;
    3.0 * inv2 * u * c1 + 3.0 * inv * u2 * c2 + u3
}

/// Derivative of the 1D cubic Bezier.
#[inline]
fn sample_bezier_derivative(u: f32, c1: f32, c2: f32) -> f32 {
    let u2 = u * u;
    let inv = 1.0 - u;
    3.0 * inv * inv * c1 + 6.0 * inv * u * (c2 - c1) + 3.0 * u2 * (1.0 - c2)
}

// ── CSS steps() ─────────────────────────────────────────────────────────────

/// Evaluate a CSS `steps(n)` easing at progress `t`.
///
/// Divides the `[0, 1]` range into `n` equal steps. Uses `jump-start`
/// semantics: the first jump happens at the start of each interval.
#[inline]
pub fn steps_ease(t: f32, n: u32) -> f32 {
    if n == 0 || t <= 0.0 {
        return 0.0;
    }
    if t >= 1.0 {
        return 1.0;
    }
    let n_f = n as f32;
    let step = (t * n_f).floor();
    step / n_f
}

// ── Advanced parameterized easings (v0.8) ──────────────────────────────────

/// Deterministic rough/jittery easing — noise overlaid on a linear curve.
///
/// Uses a hash-based PRNG to generate `points` noise samples, then linearly
/// interpolates between them. `strength` controls deviation amplitude (0..1),
/// `seed` ensures reproducibility.
#[inline]
pub fn rough_ease(t: f32, strength: f32, points: u32, seed: u32) -> f32 {
    if points == 0 || strength == 0.0 {
        return t;
    }
    // Determine which segment t falls in
    let n = points as f32;
    let scaled = t * n;
    let idx = (scaled as u32).min(points - 1);
    let frac = scaled - idx as f32;

    // Deterministic hash for noise at each sample point
    let noise_at = |i: u32| -> f32 {
        let h = i.wrapping_mul(2654435761).wrapping_add(seed.wrapping_mul(2246822519));
        let h = h ^ (h >> 16);
        let h = h.wrapping_mul(0x45d9f3b);
        let h = h ^ (h >> 16);
        // Map to [-1, 1]
        (h as f32 / u32::MAX as f32) * 2.0 - 1.0
    };

    let n0 = noise_at(idx);
    let n1 = if idx + 1 < points { noise_at(idx + 1) } else { 0.0 };
    let noise = n0 + (n1 - n0) * frac;

    (t + noise * strength).clamp(0.0, 1.0)
}

/// Slow-fast-slow piecewise easing for dramatic effects.
///
/// `ratio` (0..1) controls how much of the curve is "slow" — higher values
/// mean more slow time at edges. `power` controls steepness of the slow
/// sections. If `yoyo_mode` is true, the curve is mirrored.
#[inline]
pub fn slow_mo(t: f32, ratio: f32, power: f32, yoyo_mode: bool) -> f32 {
    let t = if yoyo_mode && t > 0.5 { 1.0 - t } else { t };
    let ratio = ratio.clamp(0.0, 1.0);
    let half_ratio = ratio * 0.5;
    let slow_end = 1.0 - half_ratio;

    if t < half_ratio {
        // Slow start section — power curve from 0 to transition point
        let local = t / half_ratio;
        let transition_y = half_ratio; // linear value at boundary
        transition_y * local.powf(1.0 / power.max(0.01))
    } else if t > slow_end {
        // Slow end section — power curve approaching 1
        let local = (t - slow_end) / half_ratio;
        let transition_y = slow_end;
        transition_y + (1.0 - transition_y) * local.powf(power.max(0.01))
    } else {
        // Fast middle — linear
        t
    }
}

/// Perceptual exponential scale correction.
///
/// Maps linear progress to exponentially-scaled output, correcting the
/// perception that `1→2` looks different from `2→4`. Returns a value
/// normalised to `[0, 1]`.
#[inline]
pub fn expo_scale(t: f32, start_scale: f32, end_scale: f32) -> f32 {
    let start = start_scale.max(0.001); // avoid log(0)
    let end = end_scale.max(0.001);
    if (start - end).abs() < 1e-10 {
        return t;
    }
    let ratio = end / start;
    (start * ratio.powf(t) - start) / (end - start)
}

/// Sinusoidal wiggle easing — oscillates around the base linear curve.
///
/// `frequency` is the number of full oscillations across the animation.
/// `amplitude` (0..1) controls the maximum deviation from the linear base.
/// Output is clamped to `[0, 1]`.
#[inline]
pub fn wiggle_ease(t: f32, frequency: f32, amplitude: f32) -> f32 {
    (t + amplitude * (frequency * 2.0 * PI * t).sin()).clamp(0.0, 1.0)
}

/// Parametric bounce with configurable strength and squash.
///
/// `strength` (0..1) controls how quickly bounces decay — lower means more
/// bounces with less height each. `squash` (≥0) compresses each bounce
/// horizontally, making them sharper.
#[inline]
pub fn custom_bounce(t: f32, strength: f32, squash: f32) -> f32 {
    if t == 0.0 {
        return 0.0;
    }
    if t >= 1.0 {
        return 1.0;
    }
    let strength = strength.clamp(0.01, 1.0);

    // Number of bounces derived from strength: fewer bounces = stronger
    let bounces = (1.0 / strength).ceil() as u32;
    let bounces = bounces.max(2);

    // Find which bounce segment we're in
    let mut seg_start = 0.0_f32;
    let mut seg_width = 1.0_f32;
    let decay = strength;

    for _ in 0..bounces {
        let seg_end = seg_start + seg_width;
        if t < seg_end || seg_width < 0.001 {
            // We're in this segment — parabolic bounce
            let local = (t - seg_start) / seg_width;
            // Apply squash: compress the parabola horizontally
            let squashed = if squash > 0.0 {
                let mid = 0.5;
                let offset = local - mid;
                (mid + offset * (1.0 + squash)).clamp(0.0, 1.0)
            } else {
                local
            };
            // Parabolic arc: 4 * x * (1 - x), peaks at 1.0 for first bounce
            let height = if seg_start == 0.0 { 1.0 } else { seg_width * 4.0 };
            let arc = 4.0 * squashed * (1.0 - squashed);
            return (1.0 - height.min(1.0) * (1.0 - arc)).clamp(0.0, 1.0);
        }
        seg_start += seg_width;
        seg_width *= decay;
    }

    1.0
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

    // ── CubicBezier tests ──────────────────────────────────────────────────

    #[test]
    fn cubic_bezier_endpoints() {
        let e = Easing::CubicBezier(0.25, 0.1, 0.25, 1.0);
        assert!((e.apply(0.0) - 0.0).abs() < 1e-6);
        assert!((e.apply(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn cubic_bezier_css_ease() {
        // CSS "ease" is cubic-bezier(0.25, 0.1, 0.25, 1.0)
        let ease = Easing::CubicBezier(0.25, 0.1, 0.25, 1.0);
        let mid = ease.apply(0.5);
        // CSS ease at t=0.5 should be > 0.5 (decelerating)
        assert!(mid > 0.5, "Expected > 0.5, got {mid}");
    }

    #[test]
    fn cubic_bezier_linear_equivalent() {
        // cubic-bezier(0, 0, 1, 1) should be approximately linear
        let lin = Easing::CubicBezier(0.0, 0.0, 1.0, 1.0);
        for i in 0..=10 {
            let t = i as f32 / 10.0;
            assert!(
                (lin.apply(t) - t).abs() < 0.02,
                "CubicBezier(0,0,1,1) at t={t} = {}, expected ~{t}",
                lin.apply(t)
            );
        }
    }

    #[test]
    fn cubic_bezier_ease_in_out() {
        // CSS "ease-in-out" is cubic-bezier(0.42, 0, 0.58, 1)
        let eio = Easing::CubicBezier(0.42, 0.0, 0.58, 1.0);
        let mid = eio.apply(0.5);
        // Should be approximately 0.5 at midpoint (symmetric)
        assert!((mid - 0.5).abs() < 0.05, "Expected ~0.5, got {mid}");
    }

    // ── Steps tests ────────────────────────────────────────────────────────

    #[test]
    fn steps_basic() {
        let stepped = Easing::Steps(4);
        assert!((stepped.apply(0.0) - 0.0).abs() < 1e-6);
        assert!((stepped.apply(0.1) - 0.0).abs() < 1e-6);
        assert!((stepped.apply(0.3) - 0.25).abs() < 1e-6);
        assert!((stepped.apply(0.5) - 0.5).abs() < 1e-6);
        assert!((stepped.apply(0.8) - 0.75).abs() < 1e-6);
        assert!((stepped.apply(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn steps_one() {
        let s = Easing::Steps(1);
        assert!((s.apply(0.0) - 0.0).abs() < 1e-6);
        assert!((s.apply(0.5) - 0.0).abs() < 1e-6);
        assert!((s.apply(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn steps_zero_returns_zero() {
        let s = Easing::Steps(0);
        assert!((s.apply(0.5) - 0.0).abs() < 1e-6);
    }

    // ── Advanced parameterized easing tests (v0.8) ──────────────────────────

    #[test]
    fn rough_ease_endpoints() {
        let e = Easing::RoughEase { strength: 0.5, points: 20, seed: 42 };
        assert!((e.apply(0.0)).abs() < 1e-5, "RoughEase(0) should be ~0");
        assert!((e.apply(1.0) - 1.0).abs() < 1e-5, "RoughEase(1) should be ~1");
    }

    #[test]
    fn rough_ease_zero_strength_is_linear() {
        let e = Easing::RoughEase { strength: 0.0, points: 20, seed: 42 };
        for i in 0..=10 {
            let t = i as f32 / 10.0;
            assert!((e.apply(t) - t).abs() < 1e-6);
        }
    }

    #[test]
    fn slow_mo_endpoints() {
        let e = Easing::SlowMo { ratio: 0.7, power: 0.7, yoyo_mode: false };
        assert!((e.apply(0.0)).abs() < 1e-5);
        assert!((e.apply(1.0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn expo_scale_endpoints() {
        let e = Easing::ExpoScale { start_scale: 1.0, end_scale: 10.0 };
        assert!((e.apply(0.0)).abs() < 1e-5);
        assert!((e.apply(1.0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn expo_scale_equal_scales_is_linear() {
        let e = Easing::ExpoScale { start_scale: 5.0, end_scale: 5.0 };
        for i in 0..=10 {
            let t = i as f32 / 10.0;
            assert!((e.apply(t) - t).abs() < 1e-5);
        }
    }

    #[test]
    fn wiggle_zero_amplitude_is_linear() {
        let e = Easing::Wiggle { frequency: 5.0, amplitude: 0.0 };
        for i in 0..=10 {
            let t = i as f32 / 10.0;
            assert!((e.apply(t) - t).abs() < 1e-6);
        }
    }

    #[test]
    fn wiggle_endpoints() {
        let e = Easing::Wiggle { frequency: 3.0, amplitude: 0.3 };
        assert!((e.apply(0.0)).abs() < 1e-5);
        assert!((e.apply(1.0) - 1.0).abs() < 1e-2);
    }

    #[test]
    fn custom_bounce_endpoints() {
        let e = Easing::CustomBounce { strength: 0.5, squash: 0.0 };
        assert!((e.apply(0.0)).abs() < 1e-5);
        assert!((e.apply(1.0) - 1.0).abs() < 1e-5);
    }
}
