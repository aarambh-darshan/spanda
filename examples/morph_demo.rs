//! spanda v0.8.0 — shape morphing demo.
//!
//! Demonstrates MorphPath interpolating between two shapes.
//! Run with: cargo run --example morph_demo

use spanda::morph::MorphPath;
use spanda::easing::Easing;

fn main() {
    println!("=== spanda MorphPath Demo ===\n");

    // Triangle → Square morph
    let triangle = vec![
        [20.0, 0.0],
        [0.0, 40.0],
        [40.0, 40.0],
    ];
    let square = vec![
        [0.0, 0.0],
        [0.0, 40.0],
        [40.0, 40.0],
    ];

    let mut morph = MorphPath::new(triangle, square)
        .duration(1.0)
        .easing(Easing::EaseInOutCubic)
        .build();

    let steps = 10;
    println!("Triangle → Square morph ({steps} steps):\n");
    println!("  t    |  P0           |  P1           |  P2");
    println!("  -----|---------------|---------------|---------------");

    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        morph.seek(t);
        let pts = morph.value();
        println!(
            "  {:.1}  | ({:5.1}, {:5.1}) | ({:5.1}, {:5.1}) | ({:5.1}, {:5.1})",
            t,
            pts[0][0], pts[0][1],
            pts[1][0], pts[1][1],
            pts[2][0], pts[2][1],
        );
    }

    // Auto-resample demo
    println!("\n--- Resample demo ---\n");

    let few = vec![[0.0, 0.0], [100.0, 100.0]];
    let many = vec![[0.0, 0.0], [30.0, 50.0], [70.0, 50.0], [100.0, 0.0]];

    let morph2 = MorphPath::new(few, many)
        .duration(1.0)
        .build();

    let pts = morph2.value();
    println!("  2 points auto-resampled to {} points:", pts.len());
    for (i, p) in pts.iter().enumerate() {
        println!("  P{i}: ({:.1}, {:.1})", p[0], p[1]);
    }

    // DrawSVG demo
    println!("\n--- DrawSVG demo ---\n");

    let path_length = 320.0;
    let mut draw = spanda::draw_on(path_length)
        .duration(1.0)
        .easing(Easing::EaseInOutCubic)
        .build();

    println!("  stroke-dashoffset animation (path_length = {path_length}):\n");
    for i in 0..=10 {
        let t = i as f32 / 10.0;
        draw.seek(t);
        let offset = draw.value();
        let drawn_pct = ((path_length - offset) / path_length * 100.0) as usize;
        let bar: String = (0..50).map(|j| if j < drawn_pct / 2 { '#' } else { '.' }).collect();
        println!("  t={t:.1} offset={offset:6.1} [{bar}]");
    }

    println!("\nDone!");
}
