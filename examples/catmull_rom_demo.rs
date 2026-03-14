//! Catmull-Rom spline demo — smooth curves through points with tension control.
//!
//! Shows how CatmullRomSpline interpolates through points and how the
//! tension parameter affects the curve shape.
//!
//! Run with: `cargo run --example catmull_rom_demo`

use spanda::bezier::{CatmullRomSpline, PathEvaluate2D, tangent_angle_deg};

fn main() {
    let points = vec![
        [0.0, 50.0],
        [40.0, 10.0],
        [80.0, 90.0],
        [120.0, 30.0],
        [160.0, 70.0],
        [200.0, 50.0],
    ];

    println!("  ══════════════════════════════════════════════");
    println!("  CatmullRom Spline — passes through every point");
    println!("  ══════════════════════════════════════════════\n");
    println!("  Points: {:?}\n", points);

    let spline = CatmullRomSpline::new(points.clone());

    // Sample and display
    let steps = 20;
    println!("  {:>5}  {:>8}  {:>8}  {:>10}  {:>10}", "t", "x", "y", "tangent", "rotation");
    println!("  {:>5}  {:>8}  {:>8}  {:>10}  {:>10}", "---", "------", "------", "--------", "--------");

    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let pos = spline.evaluate([0.0, 0.0], t);
        let tan = spline.tangent([0.0, 0.0], t);
        let rot = tangent_angle_deg(tan);
        println!(
            "  {:.2}  {:8.2}  {:8.2}  ({:+.1},{:+.1})  {:+7.1}deg",
            t, pos[0], pos[1], tan[0], tan[1], rot,
        );
    }

    // ── Tension comparison ──

    println!("\n  ══════════════════════════════════════════════");
    println!("  Tension comparison — ASCII plot");
    println!("  ══════════════════════════════════════════════\n");

    let tensions = [0.0, 0.5, 1.0, 1.5];

    for &tension in &tensions {
        let spline = CatmullRomSpline::new(points.clone()).tension(tension);

        println!("  tension = {:.1}:", tension);
        draw_spline(&spline, &points, 60, 12);
        println!();
    }

    // ── Tangent direction visualization ──

    println!("  ══════════════════════════════════════════════");
    println!("  Tangent arrows along the curve");
    println!("  ══════════════════════════════════════════════\n");

    let spline = CatmullRomSpline::new(points.clone());

    for i in 0..=10 {
        let t = i as f32 / 10.0;
        let pos = spline.evaluate([0.0, 0.0], t);
        let tan = spline.tangent([0.0, 0.0], t);
        let rot = tangent_angle_deg(tan);

        // Pick an arrow character based on angle
        let arrow = angle_to_arrow(rot);

        println!(
            "  t={:.1}  ({:6.1}, {:5.1})  {arrow}  {:+.0}deg",
            t, pos[0], pos[1], rot,
        );
    }
}

fn angle_to_arrow(deg: f32) -> &'static str {
    let d = ((deg % 360.0) + 360.0) % 360.0;
    match d as u32 {
        0..=22 | 338..=360 => "->",
        23..=67 => "/^",
        68..=112 => "^",
        113..=157 => "^\\",
        158..=202 => "<-",
        203..=247 => "v/",
        248..=292 => "v",
        293..=337 => "\\v",
        _ => "->",
    }
}

fn draw_spline(spline: &CatmullRomSpline, knots: &[[f32; 2]], width: usize, height: usize) {
    let samples = 120;
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;

    let mut curve_pts = Vec::with_capacity(samples + 1);
    for i in 0..=samples {
        let t = i as f32 / samples as f32;
        let p = spline.evaluate([0.0, 0.0], t);
        min_x = min_x.min(p[0]);
        max_x = max_x.max(p[0]);
        min_y = min_y.min(p[1]);
        max_y = max_y.max(p[1]);
        curve_pts.push(p);
    }

    // Also include knots in bounds
    for k in knots {
        min_x = min_x.min(k[0]);
        max_x = max_x.max(k[0]);
        min_y = min_y.min(k[1]);
        max_y = max_y.max(k[1]);
    }

    let range_x = (max_x - min_x).max(1.0);
    let range_y = (max_y - min_y).max(1.0);

    let mut grid = vec![vec![' '; width]; height];

    // Plot curve
    for p in &curve_pts {
        let gx = ((p[0] - min_x) / range_x * (width - 1) as f32) as usize;
        let gy = ((p[1] - min_y) / range_y * (height - 1) as f32) as usize;
        let gy = (height - 1).saturating_sub(gy);
        if gx < width && gy < height && grid[gy][gx] == ' ' {
            grid[gy][gx] = '.';
        }
    }

    // Plot knots (overwrite curve)
    for k in knots {
        let gx = ((k[0] - min_x) / range_x * (width - 1) as f32) as usize;
        let gy = ((k[1] - min_y) / range_y * (height - 1) as f32) as usize;
        let gy = (height - 1).saturating_sub(gy);
        if gx < width && gy < height {
            grid[gy][gx] = 'O';
        }
    }

    for row in &grid {
        print!("  |");
        for ch in row {
            print!("{}", ch);
        }
        println!("|");
    }
}
