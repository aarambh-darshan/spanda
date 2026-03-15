//! Colour interpolation comparison demo.
//!
//! Shows sRGB vs Lab vs Oklch interpolation side-by-side in the terminal.
//!
//! Run with: `cargo run --example colour_demo --features palette`

use palette::Srgba;
use spanda::colour::{lerp_in_lab, lerp_in_oklch, lerp_in_linear};
use spanda::traits::Interpolate;

fn srgba_to_ansi(c: Srgba) -> String {
    let r = (c.red.clamp(0.0, 1.0) * 255.0) as u8;
    let g = (c.green.clamp(0.0, 1.0) * 255.0) as u8;
    let b = (c.blue.clamp(0.0, 1.0) * 255.0) as u8;
    format!("\x1b[48;2;{r};{g};{b}m  \x1b[0m")
}

fn print_gradient(label: &str, steps: usize, f: impl Fn(f32) -> Srgba) {
    print!("  {label:<9}");
    for i in 0..steps {
        let t = i as f32 / (steps - 1) as f32;
        print!("{}", srgba_to_ansi(f(t)));
    }
    println!();
}

fn main() {
    let pairs: &[(&str, Srgba, Srgba)] = &[
        (
            "Red -> Blue",
            Srgba::new(1.0, 0.0, 0.0, 1.0),
            Srgba::new(0.0, 0.0, 1.0, 1.0),
        ),
        (
            "Red -> Cyan",
            Srgba::new(1.0, 0.0, 0.0, 1.0),
            Srgba::new(0.0, 1.0, 1.0, 1.0),
        ),
        (
            "Yellow -> Blue",
            Srgba::new(1.0, 1.0, 0.0, 1.0),
            Srgba::new(0.0, 0.0, 1.0, 1.0),
        ),
        (
            "Green -> Magenta",
            Srgba::new(0.0, 0.8, 0.0, 1.0),
            Srgba::new(0.8, 0.0, 0.8, 1.0),
        ),
    ];

    let steps = 32;

    println!();
    println!("  spanda 0.7.0 — Colour Interpolation Comparison");
    println!("  ================================================");

    for (label, from, to) in pairs {
        println!("\n  {label}:");
        let (f, t) = (*from, *to);
        print_gradient("sRGB", steps, |p| f.lerp(&t, p));
        print_gradient("Linear", steps, |p| lerp_in_linear(f, t, p));
        print_gradient("Lab", steps, |p| lerp_in_lab(f, t, p));
        print_gradient("Oklch", steps, |p| lerp_in_oklch(f, t, p));
    }

    println!();
    println!("  sRGB:    component-wise lerp (fast, but dull/dark midpoints)");
    println!("  Linear:  physically correct blending (no gamma artifacts)");
    println!("  Lab:     perceptually uniform (maintains brightness)");
    println!("  Oklch:   perceptually uniform + natural hue rotation");
    println!();
}
