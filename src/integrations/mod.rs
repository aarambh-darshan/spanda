//! Integration modules for specific platforms and engines.
//!
//! Each sub-module is gated behind a feature flag:
//!
//! | Feature | Module | What it provides |
//! |---------|--------|------------------|
//! | `bevy`  | [`bevy`] | `SpandaPlugin` — auto-ticks Tween/Spring components |
//! | `wasm`  | [`wasm`] | `RafDriver` — `requestAnimationFrame` loop |

#[cfg(feature = "bevy")]
pub mod bevy;

#[cfg(feature = "wasm")]
pub mod wasm;
