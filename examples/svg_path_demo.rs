//! SVG path parsing demo — parse SVG d-strings and animate along them.
//!
//! Shows how to convert SVG path data into a CompoundPath and evaluate
//! positions along it.
//!
//! Run with: `cargo run --example svg_path_demo`

use spanda::motion_path::CompoundPath;
use spanda::svg_path::SvgPathParser;

fn main() {
    let paths = [
        (
            "Simple line",
            "M 0 0 L 100 0 L 100 100 L 0 100 Z",
        ),
        (
            "S-curve",
            "M 0 0 C 0 80 100 -30 100 50 C 100 130 200 20 200 100",
        ),
        (
            "Heart shape",
            "M 100 30 C 100 0 50 0 50 30 C 50 60 100 80 100 100 C 100 80 150 60 150 30 C 150 0 100 0 100 30 Z",
        ),
        (
            "Arrow (H/V)",
            "M 0 50 H 80 V 20 L 120 50 L 80 80 V 50",
        ),
        (
            "Relative commands",
            "M 0 0 l 50 0 l 0 50 l -50 0 z",
        ),
    ];

    for (name, d_string) in &paths {
        println!("  ══════════════════════════════════════════════");
        println!("  {}", name);
        println!("  d=\"{}\"", d_string);
        println!("  ══════════════════════════════════════════════\n");

        let commands = SvgPathParser::parse(d_string);
        println!("  Parsed {} commands", commands.len());

        let path = CompoundPath::new(commands);
        println!("  Segments: {} | Arc length: {:.1}\n", path.segment_count(), path.arc_length());

        // Sample 11 points along the path
        let steps = 10;
        println!("  {:>5}  {:>8}  {:>8}  {:>10}", "u", "x", "y", "rotation");
        println!("  {:>5}  {:>8}  {:>8}  {:>10}", "---", "------", "------", "--------");

        for i in 0..=steps {
            let u = i as f32 / steps as f32;
            let pos = path.position(u);
            let rot = path.rotation_deg(u);
            println!(
                "  {:.1}  {:8.2}  {:8.2}  {:+8.1}deg",
                u, pos[0], pos[1], rot,
            );
        }

        // Show the path as a simple ASCII plot
        println!();
        draw_path(&path, 50, 15);
        println!();
    }
}

/// Draw a tiny ASCII plot of the path.
fn draw_path(path: &CompoundPath, width: usize, height: usize) {
    // Sample many points to find bounds
    let samples = 100;
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;

    let mut points = Vec::with_capacity(samples + 1);
    for i in 0..=samples {
        let u = i as f32 / samples as f32;
        let p = path.position(u);
        min_x = min_x.min(p[0]);
        max_x = max_x.max(p[0]);
        min_y = min_y.min(p[1]);
        max_y = max_y.max(p[1]);
        points.push(p);
    }

    // Add padding
    let range_x = (max_x - min_x).max(1.0);
    let range_y = (max_y - min_y).max(1.0);

    // Plot on a grid
    let mut grid = vec![vec![' '; width]; height];
    for (i, p) in points.iter().enumerate() {
        let gx = ((p[0] - min_x) / range_x * (width - 1) as f32) as usize;
        let gy = ((p[1] - min_y) / range_y * (height - 1) as f32) as usize;
        let gy = (height - 1).saturating_sub(gy); // flip y
        let ch = if i == 0 {
            'S'
        } else if i == points.len() - 1 {
            'E'
        } else {
            '*'
        };
        if gx < width && gy < height {
            grid[gy][gx] = ch;
        }
    }

    for row in &grid {
        print!  ("  |");
        for ch in row {
            print!("{}", ch);
        }
        println!("|");
    }
}
