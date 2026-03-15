# Colour Animation

> Activate with `features = ["palette"]`

spanda provides first-class colour interpolation via the [palette](https://crates.io/crates/palette) crate. All supported colour types work seamlessly with `Tween<T>`, `KeyframeTrack<T>`, `SpringN<T>`, and every other animation primitive.

---

## Quick Start

```rust
use palette::Srgba;
use spanda::{Tween, Easing};
use spanda::traits::Update;

let mut tween = Tween::new(
    Srgba::new(1.0_f32, 0.0, 0.0, 1.0),  // red
    Srgba::new(0.0_f32, 0.0, 1.0, 1.0),  // blue
)
    .duration(1.0)
    .easing(Easing::EaseInOutCubic)
    .build();

tween.update(0.5);
let colour = tween.value();
```

---

## The Dark-Midpoint Problem

Interpolating colours in sRGB space produces dull, dark midpoints. For example, a Red → Cyan gradient in sRGB passes through muddy grey, while the same gradient in Lab or Oklch space maintains brightness:

| Space | Behaviour |
|-------|-----------|
| **sRGB** | Fast, but midpoints are dark/dull |
| **Linear RGB** | Physically correct, no gamma artefacts |
| **CIE L\*a\*b\*** | Perceptually uniform — maintains brightness |
| **OKLCh** | Perceptually uniform + natural hue rotation |

Run the demo to see this in your terminal:

```bash
cargo run --example colour_demo --features palette
```

---

## Colour-Space-Aware Wrappers

Use the newtype wrappers to interpolate in a perceptual colour space while keeping your start/end values as `Srgba`:

### `InLab` — CIE L\*a\*b\*

```rust
use palette::Srgba;
use spanda::colour::InLab;
use spanda::{Tween, Easing};

let mut tween = Tween::new(
    InLab(Srgba::new(1.0, 0.0, 0.0, 1.0)),
    InLab(Srgba::new(0.0, 0.0, 1.0, 1.0)),
)
    .duration(1.0)
    .easing(Easing::EaseInOutCubic)
    .build();
```

Access the inner `Srgba` with `.0`:

```rust
let srgba = tween.value().0;
```

### `InOklch` — OKLCh

Best for gradients that involve hue rotation (e.g. rainbow effects). Uses shortest-arc hue interpolation automatically.

```rust
use spanda::colour::InOklch;

let tween = Tween::new(
    InOklch(Srgba::new(1.0, 0.0, 0.0, 1.0)),  // red
    InOklch(Srgba::new(0.0, 1.0, 0.0, 1.0)),  // green
).duration(2.0).build();
```

### `InLinear` — Linear RGB

Physically correct blending without gamma-curve artefacts.

```rust
use spanda::colour::InLinear;

let tween = Tween::new(
    InLinear(Srgba::new(1.0, 1.0, 1.0, 1.0)),
    InLinear(Srgba::new(0.0, 0.0, 0.0, 1.0)),
).duration(1.0).build();
```

---

## Which Colour Space to Choose

| Goal | Use |
|------|-----|
| Maximum performance | `Tween<Srgba>` (direct sRGB lerp) |
| Physically correct blending | `Tween<InLinear>` |
| Perceptually smooth gradient | `Tween<InLab>` |
| Smooth + hue rotation | `Tween<InOklch>` |
| Spring physics on colour | `SpringN<Srgba>` or `SpringN<InLab>` |

---

## Convenience Functions

For one-off interpolation without creating a tween:

```rust
use palette::Srgba;
use spanda::colour::{lerp_in_lab, lerp_in_oklch, lerp_in_linear};

let red = Srgba::new(1.0, 0.0, 0.0, 1.0);
let blue = Srgba::new(0.0, 0.0, 1.0, 1.0);

let lab_mid = lerp_in_lab(red, blue, 0.5);
let oklch_mid = lerp_in_oklch(red, blue, 0.5);
let linear_mid = lerp_in_linear(red, blue, 0.5);
```

---

## Using with SpringN

Colour types implement `SpringAnimatable`, so they work with spring physics:

```rust
use palette::Srgba;
use spanda::spring::{SpringN, SpringConfig};

let mut spring = SpringN::new(
    SpringConfig::wobbly(),
    Srgba::new(1.0, 0.0, 0.0, 1.0),  // start red
);
spring.set_target(Srgba::new(0.0, 0.0, 1.0, 1.0));  // target blue

// Tick at 60fps
spring.update(1.0 / 60.0);
let current_colour = spring.position();
```

---

## Supported Types

| palette Type | `Interpolate` | `SpringAnimatable` | Notes |
|-------------|:---:|:---:|-------|
| `Srgba<f32>` | ✓ | ✓ | Most common web colour |
| `Srgb<f32>` | ✓ | ✓ | sRGB without alpha |
| `LinSrgba<f32>` | ✓ | ✓ | Linear sRGB + alpha |
| `LinSrgb<f32>` | ✓ | ✓ | Linear sRGB |
| `Laba<f32>` | ✓ | ✓ | CIE L\*a\*b\* + alpha |
| `Lab<f32>` | ✓ | ✓ | CIE L\*a\*b\* |
| `Oklcha<f32>` | ✓ | — | OKLCh + alpha (shortest-arc hue) |
| `Oklch<f32>` | ✓ | — | OKLCh (shortest-arc hue) |
| `Hsla<f32>` | ✓ | — | HSL + alpha (shortest-arc hue) |
| `InLab` | ✓ | ✓ | sRGB wrapper, interpolates in Lab |
| `InOklch` | ✓ | ✓ | sRGB wrapper, interpolates in OKLCh |
| `InLinear` | ✓ | ✓ | sRGB wrapper, interpolates in linear RGB |

> Hue-based types (Oklch, Hsl) lack `SpringAnimatable` because spring physics doesn't handle circular hue values well.
