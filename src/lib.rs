//! # spanda
//!
//! *Sanskrit: स्पन्द — vibration, pulse, the throb of motion.*
//!
//! A general-purpose animation library for Rust.  Zero mandatory dependencies,
//! `no_std`-ready, and designed to work anywhere: terminal UIs, web (WASM),
//! game engines (Bevy), or native desktop apps.
//!
//! ## Feature flags
//!
//! | Flag       | What it adds                                          |
//! |------------|-------------------------------------------------------|
//! | `std`      | *(default)* wall-clock driver, thread-safe internals  |
//! | `serde`    | `Serialize`/`Deserialize` on all public types         |
//! | `bevy`     | `SpandaPlugin` for Bevy 0.13                          |
//! | `wasm`     | `requestAnimationFrame` driver                        |
//! | `palette`  | Colour interpolation via the `palette` crate          |
//! | `tokio`    | `async` / `.await` on timeline completion             |
//!
//! ## Quick start
//!
//! ```rust
//! use spanda::{Tween, Easing};
//! use spanda::traits::Update;
//!
//! let mut tween = Tween::new(0.0_f32, 100.0)
//!     .duration(1.0)
//!     .easing(Easing::EaseOutCubic)
//!     .build();
//!
//! // Simulate 10 frames:
//! for _ in 0..10 {
//!     tween.update(0.1);
//! }
//!
//! assert!(tween.is_complete());
//! assert!((tween.value() - 100.0).abs() < 1e-6);
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![warn(missing_docs, missing_debug_implementations)]

#[cfg(not(feature = "std"))]
extern crate alloc;

// ── Module declarations ───────────────────────────────────────────────────────

pub mod easing;
pub mod traits;
pub mod tween;
pub mod keyframe;
pub mod timeline;
pub mod spring;
pub mod clock;
pub mod driver;
pub mod scroll;
pub mod path;
pub mod bezier;
pub mod motion_path;
pub mod svg_path;
pub mod svg_draw;
pub mod morph;
pub mod inertia;
pub mod drag;
pub mod integrations;

#[cfg(feature = "palette")]
pub mod colour;

// ── Top-level re-exports (ergonomic imports) ──────────────────────────────────

pub use easing::Easing;
pub use traits::{Animatable, Interpolate, Update};
pub use tween::{Tween, TweenState, snap_to, round_to};
pub use keyframe::{KeyframeTrack, Keyframe, Loop};
pub use timeline::{Timeline, Sequence, At, stagger};
pub use spring::{Spring, SpringConfig, SpringN, SpringAnimatable};
pub use driver::{AnimationDriver, AnimationId};
pub use clock::{Clock, ManualClock, MockClock};
pub use scroll::{ScrollClock, ScrollDriver};
pub use path::{BezierPath, MotionPath, MotionPathTween, PathEvaluate};
pub use bezier::{CatmullRomSpline, PathEvaluate2D, tangent_angle, tangent_angle_deg};
pub use motion_path::{PolyPath, CompoundPath, PathCommand};
pub use svg_path::SvgPathParser;
pub use svg_draw::{draw_on, draw_on_reverse};
pub use morph::{MorphPath, resample};
pub use inertia::{Inertia, InertiaN, InertiaConfig};
pub use drag::{DragState, DragConstraints, DragAxis, PointerData};

#[cfg(feature = "std")]
pub use clock::WallClock;

#[cfg(feature = "palette")]
pub use colour::{lerp_in_lab, lerp_in_linear, lerp_in_oklch, InLab, InLinear, InOklch};
