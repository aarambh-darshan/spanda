# Contributing to spanda

Thank you for your interest in contributing to spanda! This document provides guidelines and information to make the contribution process smooth.

## License

By contributing to spanda, you agree that your contributions will be dual-licensed under the [MIT License](LICENSE-MIT) and [Apache License 2.0](LICENSE-APACHE), without any additional terms or conditions.

## Getting Started

1. Fork the repository: `https://github.com/aarambh-darshan/spanda`
2. Clone your fork: `git clone https://github.com/<your-username>/spanda`
3. Create a feature branch: `git checkout -b my-feature`
4. Make your changes
5. Run the checks (see below)
6. Commit and push
7. Open a pull request

## Development Setup

You need Rust stable (edition 2021). No other system dependencies are required for the core library.

### Running Tests

```bash
# All unit + integration + doc tests
cargo test

# With all features enabled
cargo test --features palette

# no_std compatibility (core modules only)
cargo test --no-default-features

# Integration tests only
cargo test --tests
```

### Linting & Formatting

```bash
cargo clippy --all-features -- -D warnings
cargo fmt --check
```

### Benchmarks

```bash
cargo bench
```

## Code Style

### General Conventions

- **All durations are in seconds** (`f32`), never milliseconds
- **Builder pattern** for complex constructors: `Type::new(...).option().option().build()`
- **`update(dt: f32) -> bool`** returns `false` when the animation is complete
- **`value()` / `position()`** to read current state from any animation
- **No `unsafe` code** — enforced by `#![forbid(unsafe_code)]`
- **All public items need doc comments** — enforced by `#![warn(missing_docs)]`
- **All public types need Debug** — enforced by `#![warn(missing_debug_implementations)]`

### Test Conventions

- Tests go in `#[cfg(test)] mod tests { ... }` at the bottom of each source file
- Integration tests go in `tests/`
- Use descriptive test names: `tween_delay_is_respected`, `spring_settles_to_target`
- Use `assert!((actual - expected).abs() < 1e-6)` for float comparison

### Module Structure

Each module follows this pattern:

```rust
//! Module-level documentation with example.

use crate::...;

/// Public type documentation.
pub struct MyType { ... }

impl MyType {
    /// Constructor.
    pub fn new(...) -> Self { ... }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn my_type_basic() { ... }
}
```

## Common Tasks

### Adding a New Easing Curve

1. Add the variant to the `Easing` enum in `src/easing.rs`
2. Add a `///` doc comment describing the curve
3. Implement the pure function (e.g., `pub fn my_ease(t: f32) -> f32`)
4. Add a match arm in `Easing::apply()`
5. Add a match arm in `Easing::name()`
6. Add match arms in `Debug` and `PartialEq` implementations
7. Add unit tests (at minimum: endpoints `apply(0.0) == 0.0`, `apply(1.0) == 1.0`)
8. Update `docs/easing.md`

### Adding a New Animatable Type

Implement the `Interpolate` trait:

```rust
use spanda::traits::Interpolate;

#[derive(Clone)]
struct MyType { x: f32, y: f32 }

impl Interpolate for MyType {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        MyType {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }
}
// MyType is now Animatable and works with Tween<MyType>, KeyframeTrack<MyType>, etc.
```

### Adding a Feature-Gated Module

1. Add the feature to `Cargo.toml` under `[features]`
2. Add the module declaration in `src/lib.rs` with `#[cfg(feature = "...")]`
3. Add re-exports with the same `#[cfg(...)]` gate
4. Test with `cargo test --features <your-feature>`
5. Document the feature requirement in doc comments

## Pull Request Guidelines

- **One feature per PR** — keep changes focused
- **Include tests** for new functionality
- **Update relevant docs** in `docs/` if applicable
- **Run the full check suite** before submitting:
  ```bash
  cargo test --features palette && cargo clippy --all-features -- -D warnings && cargo fmt --check
  ```
- **PR description** should explain the *why*, not just the *what*
- Keep commits clean and descriptive

## Reporting Issues

Please use [GitHub Issues](https://github.com/aarambh-darshan/spanda/issues) and include:

- Rust version (`rustc --version`)
- Feature flags you're using
- Minimal reproduction code
- Expected vs actual behaviour
