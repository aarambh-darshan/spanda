//! Integration modules for specific platforms and engines.
//!
//! Each sub-module is gated behind a feature flag:
//!
//! | Feature    | Module | What it provides |
//! |------------|--------|------------------|
//! | `bevy`     | [`bevy`] | `SpandaPlugin` — auto-ticks Tween/Spring components |
//! | `wasm`     | [`wasm`] | `RafDriver` — `requestAnimationFrame` loop |
//! | `wasm-dom` | [`flip`], [`split_text`], [`scroll_smoother`], [`draggable`], [`observer`] | DOM interaction plugins |

#[cfg(feature = "bevy")]
pub mod bevy;

#[cfg(feature = "wasm")]
pub mod wasm;

/// SplitText is always compiled (pure string splitting works everywhere).
/// DOM injection methods are gated behind `wasm-dom`.
pub mod split_text;

#[cfg(feature = "wasm-dom")]
pub mod flip;

#[cfg(feature = "wasm-dom")]
pub mod scroll_smoother;

#[cfg(feature = "wasm-dom")]
pub mod draggable;

#[cfg(feature = "wasm-dom")]
pub mod observer;
