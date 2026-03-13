//! Animated progress bar in the terminal using spanda + ratatui (mock).
//!
//! This example demonstrates a simple use of `Tween<f32>` to animate
//! a progress value from 0.0 to 1.0 with an easing curve.
//!
//! Run with: `cargo run --example tui_progress`

use spanda::clock::{Clock, WallClock};
use spanda::easing::Easing;
use spanda::traits::Update;
use spanda::tween::Tween;

fn main() {
    let mut tween = Tween::new(0.0_f32, 1.0)
        .duration(2.0)
        .easing(Easing::EaseInOutCubic)
        .build();

    let mut clock = WallClock::new();

    println!("Animating progress bar (2 seconds)...\n");

    loop {
        let dt = clock.delta();
        tween.update(dt);

        let value = tween.value();
        let bar_width = 40;
        let filled = (value * bar_width as f32) as usize;
        let empty = bar_width - filled;

        print!("\r  [");
        for _ in 0..filled {
            print!("█");
        }
        for _ in 0..empty {
            print!("░");
        }
        print!("] {:5.1}%", value * 100.0);

        if tween.is_complete() {
            println!("\n\n  ✓ Animation complete!");
            break;
        }

        std::thread::sleep(std::time::Duration::from_millis(16)); // ~60fps
    }
}
