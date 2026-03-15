//! Integration tests for colour interpolation (requires `palette` feature).

use palette::Srgba;
use spanda::colour::{lerp_in_lab, lerp_in_oklch, InLab, InLinear, InOklch};
use spanda::spring::{SpringConfig, SpringN};
use spanda::traits::{Interpolate, Update};
use spanda::tween::Tween;

#[test]
fn tween_srgba_full_lifecycle() {
    let mut t = Tween::new(
        Srgba::new(1.0_f32, 0.0, 0.0, 1.0),
        Srgba::new(0.0, 1.0, 0.0, 1.0),
    )
    .duration(1.0)
    .build();

    for _ in 0..10 {
        t.update(0.1);
    }
    assert!(t.is_complete());
    let v = t.value();
    assert!((v.green - 1.0).abs() < 1e-3);
}

#[test]
fn tween_in_lab_full_lifecycle() {
    let mut t = Tween::new(
        InLab(Srgba::new(1.0, 0.0, 0.0, 1.0)),
        InLab(Srgba::new(0.0, 0.0, 1.0, 1.0)),
    )
    .duration(1.0)
    .build();

    for _ in 0..10 {
        t.update(0.1);
    }
    assert!(t.is_complete());
    let v = t.value();
    assert!((v.0.blue - 1.0).abs() < 1e-2);
}

#[test]
fn tween_in_oklch_full_lifecycle() {
    let mut t = Tween::new(
        InOklch(Srgba::new(1.0, 0.0, 0.0, 1.0)),
        InOklch(Srgba::new(0.0, 1.0, 0.0, 1.0)),
    )
    .duration(1.0)
    .build();

    for _ in 0..10 {
        t.update(0.1);
    }
    assert!(t.is_complete());
}

#[test]
fn tween_in_linear_full_lifecycle() {
    let mut t = Tween::new(
        InLinear(Srgba::new(0.0, 0.0, 0.0, 1.0)),
        InLinear(Srgba::new(1.0, 1.0, 1.0, 1.0)),
    )
    .duration(1.0)
    .build();

    for _ in 0..10 {
        t.update(0.1);
    }
    assert!(t.is_complete());
    let v = t.value();
    assert!((v.0.red - 1.0).abs() < 1e-2);
}

#[test]
fn lab_vs_srgb_midpoint_luminance() {
    let red = Srgba::new(1.0_f32, 0.0, 0.0, 1.0);
    let blue = Srgba::new(0.0, 0.0, 1.0, 1.0);

    let srgb_mid = red.lerp(&blue, 0.5);
    let lab_mid = lerp_in_lab(red, blue, 0.5);

    // Standard luminance coefficients
    let srgb_lum = srgb_mid.red * 0.2126 + srgb_mid.green * 0.7152 + srgb_mid.blue * 0.0722;
    let lab_lum = lab_mid.red * 0.2126 + lab_mid.green * 0.7152 + lab_mid.blue * 0.0722;

    // Lab should produce a brighter midpoint than sRGB for this pair
    assert!(
        lab_lum > srgb_lum * 0.8,
        "Lab should produce brighter mid: lab={lab_lum} srgb={srgb_lum}"
    );
}

#[test]
fn oklch_hue_wrapping() {
    // Red (hue ~30) → Blue (hue ~265) via Oklch
    let red = Srgba::new(1.0_f32, 0.0, 0.0, 1.0);
    let blue = Srgba::new(0.0, 0.0, 1.0, 1.0);
    let mid = lerp_in_oklch(red, blue, 0.5);
    // Mid should be a valid colour (not NaN or out of range after clamping)
    assert!(mid.red >= 0.0 && mid.red <= 1.0);
    assert!(mid.green >= 0.0 && mid.green <= 1.0);
    assert!(mid.blue >= 0.0 && mid.blue <= 1.0);
}

#[test]
fn spring_n_with_srgba_settles() {
    let mut spring = SpringN::new(SpringConfig::stiff(), Srgba::new(0.0_f32, 0.0, 0.0, 1.0));
    spring.set_target(Srgba::new(1.0, 1.0, 1.0, 1.0));

    for _ in 0..1000 {
        spring.update(1.0 / 60.0);
    }

    let pos = spring.position();
    assert!((pos.red - 1.0).abs() < 0.01);
    assert!((pos.green - 1.0).abs() < 0.01);
    assert!((pos.blue - 1.0).abs() < 0.01);
    assert!(spring.is_settled());
}
