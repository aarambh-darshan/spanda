//! Spinning animation using KeyframeTrack in the terminal.
//!
//! Cycles through braille spinner characters using a looping keyframe track.
//!
//! Run with: `cargo run --example tui_spinner`

use spanda::clock::{Clock, WallClock};
use spanda::keyframe::{KeyframeTrack, Loop};
use spanda::traits::Update;

fn main() {
    // Use integer keyframes to index into a spinner character array
    let mut track = KeyframeTrack::new()
        .push(0.0, 0_i32)
        .push(0.1, 1)
        .push(0.2, 2)
        .push(0.3, 3)
        .push(0.4, 4)
        .push(0.5, 5)
        .push(0.6, 6)
        .push(0.7, 7)
        .looping(Loop::Forever);

    let frames = ['⣾', '⣽', '⣻', '⢿', '⡿', '⣟', '⣯', '⣷'];

    let mut clock = WallClock::new();
    let start = std::time::Instant::now();
    let run_duration = std::time::Duration::from_secs(5);

    println!("Spinning for 5 seconds...\n");

    while start.elapsed() < run_duration {
        let dt = clock.delta();
        track.update(dt);

        let idx = (track.value().unwrap() as usize).min(frames.len() - 1);
        print!("\r  {} Loading...", frames[idx]);

        std::thread::sleep(std::time::Duration::from_millis(16));
    }

    println!("\r  ✓ Done!            ");
}
