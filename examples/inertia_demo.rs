//! spanda v0.8.0 — inertia physics demo.
//!
//! Demonstrates the Inertia struct decelerating from a velocity impulse.
//! Run with: cargo run --example inertia_demo

use spanda::inertia::{Inertia, InertiaConfig, InertiaN};
use spanda::traits::Update;

fn main() {
    println!("=== spanda Inertia Demo ===\n");

    // ── 1D inertia with different presets ────────────────────────────
    let presets = [
        ("default_flick", InertiaConfig::default_flick()),
        ("heavy", InertiaConfig::heavy()),
        ("snappy", InertiaConfig::snappy()),
    ];

    for (name, config) in &presets {
        let mut inertia = Inertia::new(config.clone())
            .with_velocity(1000.0)
            .with_position(0.0);

        let mut frames = 0u32;
        print!("{name:14} | ");

        while inertia.update(1.0 / 60.0) {
            frames += 1;
            if frames % 10 == 0 {
                let bar_pos = (inertia.position() / 50.0).min(40.0) as usize;
                let bar: String = (0..40)
                    .map(|i| if i == bar_pos { '#' } else { '.' })
                    .collect();
                print!(
                    "\r{name:14} | {bar} pos={:7.1} vel={:7.1}",
                    inertia.position(),
                    inertia.velocity()
                );
            }
        }

        println!(
            "\r{name:14} | settled after {frames:4} frames, final pos = {:.1}",
            inertia.position()
        );
    }

    // ── 2D inertia ───────────────────────────────────────────────────
    println!("\n--- 2D Inertia (fling gesture) ---\n");

    let mut inertia2d = InertiaN::new(InertiaConfig::default_flick(), [0.0_f32, 0.0])
        .with_velocity([600.0, -400.0]);

    println!("  frame |     x     |     y     | settled");
    println!("  ------|-----------|-----------|--------");

    let mut frame = 0;
    loop {
        let running = inertia2d.update(1.0 / 60.0);
        frame += 1;

        if frame % 20 == 0 || !running {
            let pos = inertia2d.position();
            println!(
                "  {:5} | {:9.2} | {:9.2} | {}",
                frame,
                pos[0],
                pos[1],
                if inertia2d.is_settled() { "yes" } else { "no" }
            );
        }

        if !running {
            break;
        }
        if frame > 5000 {
            println!("  (stopped after 5000 frames)");
            break;
        }
    }

    println!("\nDone!");
}
