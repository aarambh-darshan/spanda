//! Motion path demo — animate a point along various path types.
//!
//! Shows PolyPath and CompoundPath with arc-length parameterization,
//! auto-rotate, and start/end offsets.
//!
//! Run with: `cargo run --example motion_path_demo`

use spanda::motion_path::{CompoundPath, PathCommand, PolyPath};

fn main() {
    println!("  ══════════════════════════════════════════════");
    println!("  PolyPath — smooth curve through points");
    println!("  ══════════════════════════════════════════════\n");

    let path = PolyPath::from_points(vec![
        [0.0, 0.0],
        [50.0, 80.0],
        [100.0, 20.0],
        [150.0, 90.0],
        [200.0, 0.0],
    ]);

    println!("  Arc length: {:.1} units\n", path.arc_length());

    // Animate along the path in 20 steps
    let steps = 20;
    for i in 0..=steps {
        let u = i as f32 / steps as f32;
        let pos = path.position(u);
        let rot = path.rotation_deg(u);

        // Visualize x position as a bar
        let bar_x = (pos[0] / 200.0 * 40.0) as usize;
        print!("  u={:.2}  ", u);
        for j in 0..42 {
            if j == bar_x {
                print!("*");
            } else {
                print!(".");
            }
        }
        println!("  ({:6.1}, {:5.1})  rot={:+6.1}deg", pos[0], pos[1], rot);
    }

    // ── PolyPath with offsets ──

    println!("\n  ══════════════════════════════════════════════");
    println!("  PolyPath with start/end offsets (0.25 - 0.75)");
    println!("  ══════════════════════════════════════════════\n");

    let offset_path = PolyPath::from_points(vec![[0.0, 0.0], [100.0, 0.0], [200.0, 0.0]])
        .start_offset(0.25)
        .end_offset(0.75);

    for i in 0..=10 {
        let u = i as f32 / 10.0;
        let pos = offset_path.position(u);
        println!("  u={:.1} => x={:6.1} (range ~50-150)", u, pos[0]);
    }

    // ── CompoundPath ──

    println!("\n  ══════════════════════════════════════════════");
    println!("  CompoundPath — cubic bezier + line segment");
    println!("  ══════════════════════════════════════════════\n");

    let compound = CompoundPath::new(vec![
        PathCommand::MoveTo([0.0, 0.0]),
        PathCommand::CubicTo {
            control1: [33.0, 100.0],
            control2: [66.0, 100.0],
            end: [100.0, 0.0],
        },
        PathCommand::LineTo([200.0, 0.0]),
    ]);

    println!(
        "  Segments: {}  |  Arc length: {:.1}\n",
        compound.segment_count(),
        compound.arc_length()
    );

    for i in 0..=steps {
        let u = i as f32 / steps as f32;
        let pos = compound.position(u);
        let rot = compound.rotation_deg(u);

        // Visualize y height (curve goes up then flat)
        let bar_y = (pos[1].max(0.0) / 100.0 * 20.0) as usize;
        print!("  u={:.2}  ", u);
        for j in 0..22 {
            if j == bar_y {
                print!("o");
            } else {
                print!(" ");
            }
        }
        println!("  ({:6.1}, {:5.1})  rot={:+6.1}deg", pos[0], pos[1], rot);
    }

    // ── Tension comparison ──

    println!("\n  ══════════════════════════════════════════════");
    println!("  Tension comparison: 0.0 vs 0.5 vs 1.5");
    println!("  ══════════════════════════════════════════════\n");

    let points = vec![[0.0, 0.0], [50.0, 100.0], [100.0, 0.0]];

    let low = PolyPath::from_points_with_tension(points.clone(), 0.0);
    let mid = PolyPath::from_points(points.clone());
    let high = PolyPath::from_points_with_tension(points, 1.5);

    println!(
        "  {:>5}  {:>12}  {:>12}  {:>12}",
        "u", "tension=0.0", "tension=0.5", "tension=1.5"
    );
    println!(
        "  {:>5}  {:>12}  {:>12}  {:>12}",
        "---", "-----------", "-----------", "-----------"
    );

    for i in 0..=10 {
        let u = i as f32 / 10.0;
        let p_low = low.position(u);
        let p_mid = mid.position(u);
        let p_high = high.position(u);
        println!(
            "  {:.1}  ({:5.1},{:5.1})  ({:5.1},{:5.1})  ({:5.1},{:5.1})",
            u, p_low[0], p_low[1], p_mid[0], p_mid[1], p_high[0], p_high[1],
        );
    }
}
