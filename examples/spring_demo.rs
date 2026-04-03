//! Spring physics demo in the terminal.
//!
//! Shows a spring settling toward different targets, printing position and
//! velocity each frame.
//!
//! Run with: `cargo run --example spring_demo`

use spanda::spring::{Spring, SpringConfig};
use spanda::traits::Update;

fn main() {
    let configs = [
        ("Gentle", SpringConfig::gentle()),
        ("Wobbly", SpringConfig::wobbly()),
        ("Stiff", SpringConfig::stiff()),
        ("Slow", SpringConfig::slow()),
    ];

    for (name, config) in &configs {
        let mut spring = Spring::new(config.clone());
        spring.set_target(100.0);

        let bar_width = 50;
        let mut frames = 0;

        println!("\n  ── {} Spring ──", name);
        println!(
            "  Target: 100.0 | stiffness={}, damping={}",
            config.stiffness, config.damping
        );
        println!();

        while !spring.is_settled() && frames < 300 {
            spring.update(1.0 / 60.0);
            frames += 1;

            if frames % 5 == 0 {
                let pos = spring.position().clamp(0.0, 120.0);
                let bar_pos = ((pos / 120.0) * bar_width as f32) as usize;
                let target_pos = ((100.0 / 120.0) * bar_width as f32) as usize;

                print!("  ");
                for i in 0..bar_width {
                    if i == target_pos {
                        print!("│");
                    } else if i == bar_pos {
                        print!("●");
                    } else {
                        print!(" ");
                    }
                }
                println!(
                    "  pos={:6.1} vel={:6.1}",
                    spring.position(),
                    spring.velocity()
                );
            }
        }

        println!(
            "  → Settled in {} frames ({:.0}ms at 60fps)\n",
            frames,
            frames as f32 / 60.0 * 1000.0
        );
    }
}
