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
pub mod integrations;

// ── Top-level re-exports (ergonomic imports) ──────────────────────────────────

pub use easing::Easing;
pub use traits::{Animatable, Interpolate, Update};
pub use tween::{Tween, TweenState};
pub use keyframe::{KeyframeTrack, Keyframe, Loop};
pub use timeline::{Timeline, Sequence};
pub use spring::{Spring, SpringConfig};
pub use driver::{AnimationDriver, AnimationId};
pub use clock::{Clock, ManualClock, MockClock};

#[cfg(feature = "std")]
pub use clock::WallClock;
