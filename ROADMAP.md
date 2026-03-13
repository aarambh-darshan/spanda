# spanda — Full Project Roadmap

> *From working prototype → battle-tested → crates.io published*
>
> This document tracks every phase, milestone, and task from the current
> state of the library through to a stable `1.0.0` release.
> Update the checkboxes as you go.

---

## Current Status (as of writing)

| Item | Status |
|------|--------|
| `traits.rs` | ✅ Complete, tests passing |
| `easing.rs` | ✅ Complete, benchmarked |
| `tween.rs` | ✅ Complete, 12 tests passing |
| `clock.rs` | ✅ Complete, 4 tests passing |
| `driver.rs` | ✅ Complete, 5 tests passing |
| `keyframe.rs` | ✅ Complete, 10 tests passing |
| `timeline.rs` | ✅ Complete, 8 tests passing |
| `spring.rs` | ✅ Complete, 8 tests passing |
| `integrations/bevy.rs` | ✅ Written |
| `integrations/wasm.rs` | ✅ Written |
| Unit tests | ✅ 60 passed, 0 failed |
| Doc tests | ✅ 13 passed |
| Integration tests | ✅ 10 passed |
| Benchmarks | ✅ Ran, results look great |
| Leptos demo | 🔧 In progress — errors being fixed |
| Published to crates.io | ❌ Not yet |

---

## Phase 0 — Fix Current Errors (Right Now)

> Fix all compilation and runtime errors found during Leptos integration
> testing before anything else.

### 0.1 Identify and document every error

- [ ] Run `cargo leptos build` and collect all compiler errors
- [ ] Run `cargo test --all-features` and note any failures
- [ ] Run `cargo test --no-default-features` and note any failures
- [ ] Run `cargo clippy --all-features` and note warnings
- [ ] List every error with its file, line, and root cause

### 0.2 Common errors to expect and fix

**WASM / no_std boundary errors**
- [ ] `std::time` used inside a no_std path → replace with `clock.rs` abstraction
- [ ] `Box<dyn Fn>` callbacks unavailable without `alloc` → gate with `#[cfg(feature = "std")]`
- [ ] `Mutex` / `Arc` used without `std` feature → gate correctly

**Leptos integration errors**
- [ ] `store_value` / `StoredValue` mutability mismatch with `Tween<T>` → wrap in `RefCell` if needed
- [ ] Signal updates not triggering re-render → check `set_*` is called inside interval correctly
- [ ] `IntervalHandle` drop cancelling animation too early → ensure handle is stored, not dropped
- [ ] `Spring::set_target()` method missing or named differently → verify API matches demo code

**Type / trait errors**
- [ ] `Tween<f32>` not `Send` if it contains non-Send fields → audit struct fields
- [ ] `KeyframeTrack` push methods returning wrong type in builder chain → fix return types
- [ ] `Loop` enum not in scope → check `use spanda::keyframe::Loop`

**Spring stability (runtime, not compile)**
- [ ] `spring_pos()` returns `NaN` → add `NaN` guard in `Spring::update()`, fallback to target
- [ ] Spring oscillates forever → verify epsilon settle check is correct
- [ ] Large `dt` causes blow-up → verify sub-stepping is active at 120 Hz internally

### 0.3 Fix checklist per file

**`spring.rs`**
- [ ] Add NaN guard: `if self.position.is_nan() { self.position = self.target; }`
- [ ] Verify sub-step logic: `let steps = (dt / 0.00833).ceil() as u32;`
- [ ] Verify `is_settled()` threshold is reachable with `wobbly()` config
- [ ] Add `set_position()` method for teleporting spring without velocity

**`tween.rs`**
- [ ] Verify `reset()` sets `elapsed = 0.0` AND `state = TweenState::Waiting` if delay > 0
- [ ] Verify `is_complete()` returns true only after full duration + delay
- [ ] Verify `update()` with very large `dt` does not skip past end value

**`keyframe.rs`**
- [ ] Verify `Loop::Forever` wraps elapsed correctly without float drift over long runs
- [ ] Verify `push_with_easing()` exists as a method (may have been named differently)
- [ ] Verify `value()` does not panic with 0 or 1 frame track

**`timeline.rs`**
- [ ] Verify `Sequence::new().then()` chain compiles cleanly
- [ ] Verify callbacks compile under `no_std` (should be gated)

**`integrations/wasm.rs`**
- [ ] Verify `RafDriver` compiles with `wasm` feature flag only
- [ ] Verify `performance.now()` binding works via `js-sys`

**`lib.rs`**
- [ ] Ensure all `pub use` re-exports match actual public items in each module
- [ ] Ensure `extern crate alloc` is present and gated correctly

### 0.4 Verification after fixes

- [ ] `cargo build` → zero errors
- [ ] `cargo build --no-default-features` → zero errors
- [ ] `cargo build --all-features` → zero errors
- [ ] `cargo test` → all pass
- [ ] `cargo clippy --all-features -- -D warnings` → zero warnings
- [ ] Leptos demo runs in browser → all three animations visible and working
- [ ] Progress bar loops smoothly
- [ ] Opacity pulse never sticks at 0 or 1
- [ ] Spring box chases slider without NaN or freezing

---

## Phase 1 — Stabilise the API (Before Publishing)

> The Leptos test will reveal API friction points. Fix them now while you
> still can without a breaking change.

### 1.1 API review from Leptos experience

- [ ] Note every place where the demo felt awkward to write
- [ ] Note every method that was missing but expected
- [ ] Note every name that was confusing
- [ ] Write a list of proposed API changes

### 1.2 Common API improvements to make

- [ ] Add `Tween::restart()` as an alias for `reset()` (more intuitive)
- [ ] Add `Spring::snap_to(value: f32)` — teleport + zero velocity
- [ ] Add `Tween::from_value()` — start from current value, not fixed start
- [ ] Add `KeyframeTrack::clear()` — reset without rebuilding
- [ ] Add `Timeline::is_complete()` — parity with Tween
- [ ] Consider `Spring<T: Animatable>` generic version (currently only `f32`)
- [ ] Decide: should `update()` take `&mut self` or should there be a `Driver` wrapper?

### 1.3 Documentation pass

- [ ] Every `pub struct` has a `///` doc comment explaining what it is
- [ ] Every `pub fn` has a `///` doc comment with a usage example
- [ ] Every `pub enum` variant has a `///` explaining when to use it
- [ ] Every feature flag is documented in `lib.rs` top-level doc
- [ ] `README.md` has a quick-start that compiles and runs
- [ ] Run `cargo doc --all-features --open` and read through every page

### 1.4 Example polish

- [ ] `examples/tui_progress.rs` — compiles and runs cleanly
- [ ] `examples/tui_spinner.rs` — compiles and runs cleanly
- [ ] `examples/spring_demo.rs` — compiles and runs cleanly
- [ ] `examples/leptos_demo.rs` — extracted from test project, added as official example
- [ ] Each example has a comment at the top explaining what it demonstrates
- [ ] `cargo run --example tui_progress` works from repo root

---

## Phase 2 — Hardening (Quality Before Publish)

### 2.1 Edge case audit

Go through every public method and ask: *what happens with bad input?*

- [ ] `Tween::new(x, x)` — start equals end → should return start immediately
- [ ] `Tween` with `duration(0.0)` → should complete immediately on first update
- [ ] `Tween` with `duration(-1.0)` → clamp to 0.0 or panic with clear message
- [ ] `KeyframeTrack` with 0 keyframes → document behaviour, no panic
- [ ] `KeyframeTrack` with 1 keyframe → returns that value forever, no panic
- [ ] `KeyframeTrack::push()` called with time < previous frame time → sort or panic?
- [ ] `Spring` with `stiffness(0.0)` → document: returns target immediately
- [ ] `Spring` with `damping(0.0)` → document: oscillates forever
- [ ] `Spring` with `mass(0.0)` → guard against divide-by-zero
- [ ] `AnimationDriver::tick(0.0)` → no-op, no panic
- [ ] `AnimationDriver::tick(-1.0)` → clamp to 0.0, no panic
- [ ] `Timeline` with no entries → `update()` returns false immediately

### 2.2 Float safety audit

- [ ] Grep for every `/` division — add a `!= 0.0` guard or document assumption
- [ ] Grep for every `.sqrt()` — add `max(0.0)` before it
- [ ] Grep for every `.powf()` — document valid input range
- [ ] Add `debug_assert!(!value.is_nan())` at the end of every `update()` method
- [ ] Add `debug_assert!(!value.is_infinite())` similarly

### 2.3 Property-based testing (optional but good)

- [ ] Add `proptest` as a dev-dependency
- [ ] Property: for any `t ∈ [0,1]`, `easing.apply(t) ∈ [-0.5, 1.5]` (allows overshoot)
- [ ] Property: `tween.value()` never returns `NaN` for any `dt > 0`
- [ ] Property: `spring.update(dt)` never produces `NaN` for any valid config

### 2.4 Benchmark completeness

- [ ] Bench `Tween::update()` + `value()` — the hot path
- [ ] Bench `KeyframeTrack::update()` with 10 frames
- [ ] Bench `Spring::update()` with `wobbly()` config
- [ ] Bench `AnimationDriver::tick()` with 100 active tweens
- [ ] Bench `AnimationDriver::tick()` with 1000 active tweens
- [ ] Add results table to `README.md`

---

## Phase 3 — Pre-Publish Checklist

### 3.1 Metadata

- [ ] `Cargo.toml` version is `0.1.0`
- [ ] `description` is clear and under 140 characters
- [ ] `keywords` has 5 entries (crates.io maximum)
- [ ] `categories` is correct (check crates.io category list)
- [ ] `repository` URL points to real GitHub repo
- [ ] `homepage` set (can be same as repository)
- [ ] `rust-version` set to minimum supported Rust version
- [ ] `license = "MIT OR Apache-2.0"` is present
- [ ] Both `LICENSE-MIT` and `LICENSE-APACHE` files exist in repo root

### 3.2 Repository

- [ ] GitHub repo is public at `github.com/aarambh-darshan/spanda`
- [ ] `README.md` renders correctly on GitHub (check with browser)
- [ ] `CHANGELOG.md` has a `## [0.1.0] - YYYY-MM-DD` entry listing everything
- [ ] `.gitignore` excludes `/target`, `Cargo.lock` (for libraries)
- [ ] CI is set up (GitHub Actions) — see section 3.5

### 3.3 Final test matrix

```bash
# Run all of these, all must pass with zero warnings:
cargo test
cargo test --no-default-features
cargo test --all-features
cargo test --features serde
cargo test --features bevy
cargo clippy --all-features -- -D warnings
cargo doc --all-features --no-deps
cargo package --list           # verify no unwanted files are included
cargo publish --dry-run
```

### 3.4 README.md must contain

- [ ] Crate name + one-line description
- [ ] Badges: crates.io version, docs.rs, CI status, license
- [ ] Quick-start code block (copy-paste, must compile)
- [ ] Feature flag table
- [ ] Comparison table vs GSAP (good for discoverability)
- [ ] Benchmark results summary
- [ ] "Contributing" section
- [ ] License section

### 3.5 GitHub Actions CI (`.github/workflows/ci.yml`)

```yaml
name: CI
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test
      - run: cargo test --no-default-features
      - run: cargo test --all-features
      - run: cargo clippy --all-features -- -D warnings
      - run: cargo doc --all-features --no-deps
```

- [ ] CI file committed and passing on GitHub

---

## Phase 4 — Publish 0.1.0

- [ ] All Phase 0–3 tasks complete
- [ ] `cargo login` with your crates.io API token
- [ ] `cargo publish` — first publish
- [ ] Verify on `crates.io/crates/spanda`
- [ ] Verify docs at `docs.rs/spanda`
- [ ] Post on the Aarambh Dev Hub Discord announcing the crate
- [ ] Write a LinkedIn post (use "Aarambh Dev Hub", not personal name)
- [ ] Record YouTube Episode 1

---

## Phase 5 — Post-Publish Iterations

### 0.2.0 — Keyframe + Timeline polish
- [ ] Fix any API friction found by early users or from your own usage
- [ ] Add `Sequence::gap()` if not already present
- [ ] Add `Timeline::label()` for named entries
- [ ] Ship `examples/tui_spinner.rs` as a working example

### 0.3.0 — Spring generics
- [ ] `Spring<T: Animatable>` — generic over any interpolatable type
- [ ] `SpringN` internal component array approach
- [ ] Example: spring-animated `[f32; 2]` position

### 0.4.0 — Bevy integration polished
- [ ] Test SpandaPlugin against latest Bevy version
- [ ] `TweenCompleted` event working in example
- [ ] Ship `examples/bevy_bounce.rs`
- [ ] Update Bevy version in Cargo.toml if needed

### 0.5.0 — WASM / web polished
- [ ] `RafDriver` tested via wasm-pack in a real browser
- [ ] Ship `examples/wasm_tween/` project
- [ ] Add to docs: how to use with Leptos specifically
- [ ] Add to docs: how to use with Dioxus

### 0.6.0 — palette colour support
- [ ] `Interpolate` impl for `palette::Srgba`
- [ ] Example: colour animation

### 1.0.0 — Stable
- [ ] No breaking changes for at least one minor version cycle
- [ ] All examples compile and run
- [ ] Full docs.rs coverage
- [ ] Blog post on Medium (Aarambh Dev Hub)
- [ ] YouTube Episode 8: *Shipping to crates.io — full publish workflow*

---

## Version History

| Version | Date | Notes |
|---------|------|-------|
| `0.0.1` | — | Internal prototype |
| `0.1.0` | TBD | First publish — core complete |
| `0.2.0` | TBD | API improvements post-Leptos test |
| `1.0.0` | TBD | Stable API |

---

## Quick Reference — Useful Commands

```bash
# Development
cargo test                            # run all unit + integration tests
cargo test --no-default-features      # verify no_std path
cargo test --all-features             # verify all feature combinations
cargo clippy --all-features           # lint
cargo doc --all-features --open       # build + open docs
cargo bench                           # run criterion benchmarks

# Leptos test
cargo leptos watch                    # hot-reload dev server

# Pre-publish
cargo package --list                  # see what gets uploaded
cargo publish --dry-run               # final check before real publish

# Publish
cargo login                           # authenticate with crates.io
cargo publish                         # ship it
```

---

*Roadmap version: 1.0 — Aarambh Dev Hub / spanda*
*Update this file after completing each phase.*
