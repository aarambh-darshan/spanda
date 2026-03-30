//! Gesture recognition demo — simulates tap, swipe, and pinch sequences.
//!
//! Run: `cargo run --example gesture_demo`

use spanda::drag::PointerData;
use spanda::gesture::{Gesture, GestureRecognizer};

fn pointer(id: i32, x: f32, y: f32) -> PointerData {
    PointerData {
        x,
        y,
        pressure: 0.5,
        pointer_id: id,
    }
}

fn main() {
    println!("╔═══════════════════════════════════════╗");
    println!("║   spanda — Gesture Recognition Demo   ║");
    println!("╚═══════════════════════════════════════╝");
    println!();

    // ── Tap ──────────────────────────────────────────────────────────────
    {
        let mut r = GestureRecognizer::new();
        r.on_pointer_down(pointer(0, 200.0, 300.0));
        r.update(0.05);
        let g = r.on_pointer_up(pointer(0, 201.0, 300.0));
        print_gesture("Tap", &g);
    }

    // ── Long Press ───────────────────────────────────────────────────────
    {
        let mut r = GestureRecognizer::new();
        r.on_pointer_down(pointer(0, 100.0, 100.0));
        let g = r.update(0.6);
        print_gesture("Long Press", &g);
    }

    // ── Swipe Right ──────────────────────────────────────────────────────
    {
        let mut r = GestureRecognizer::new();
        r.on_pointer_down(pointer(0, 100.0, 200.0));
        r.update(0.08);
        r.on_pointer_move(pointer(0, 400.0, 210.0));
        let g = r.on_pointer_up(pointer(0, 400.0, 210.0));
        print_gesture("Swipe Right", &g);
    }

    // ── Swipe Up ─────────────────────────────────────────────────────────
    {
        let mut r = GestureRecognizer::new();
        r.on_pointer_down(pointer(0, 200.0, 500.0));
        r.update(0.06);
        r.on_pointer_move(pointer(0, 205.0, 100.0));
        let g = r.on_pointer_up(pointer(0, 205.0, 100.0));
        print_gesture("Swipe Up", &g);
    }

    // ── Pinch Zoom ───────────────────────────────────────────────────────
    {
        let mut r = GestureRecognizer::new();
        r.on_pointer_down(pointer(0, 100.0, 200.0));
        r.on_pointer_down(pointer(1, 200.0, 200.0));
        r.on_pointer_move(pointer(0, 50.0, 200.0));
        let g = r.on_pointer_move(pointer(1, 350.0, 200.0));
        print_gesture("Pinch Zoom", &g);
    }

    println!();
    println!("All gesture simulations complete ✓");
}

fn print_gesture(label: &str, gesture: &Option<Gesture>) {
    match gesture {
        Some(g) => println!("→ {label:15} → {g:?}"),
        None => println!("→ {label:15} → (no gesture detected)"),
    }
}
