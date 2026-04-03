//! Integration tests for the full motion path system:
//! CatmullRomSpline, PolyPath, CompoundPath, SvgPathParser.

use spanda::bezier::{CatmullRomSpline, PathEvaluate2D, tangent_angle_deg};
use spanda::motion_path::{CompoundPath, PathCommand, PolyPath};
use spanda::svg_path::SvgPathParser;

// ── CatmullRomSpline ──────────────────────────────────────────────────────

#[test]
fn catmull_rom_smooth_path_through_points() {
    let spline = CatmullRomSpline::new(vec![
        [0.0, 0.0],
        [50.0, 100.0],
        [100.0, 50.0],
        [150.0, 100.0],
        [200.0, 0.0],
    ]);

    // Passes through first and last points
    let start = spline.evaluate([0.0, 0.0], 0.0);
    let end = spline.evaluate([0.0, 0.0], 1.0);
    assert!((start[0]).abs() < 1e-3);
    assert!((end[0] - 200.0).abs() < 1e-3);

    // Passes through middle knot at t=0.5 (3rd of 5 points, segment 2 of 4)
    let mid = spline.evaluate([0.0, 0.0], 0.5);
    assert!(
        (mid[0] - 100.0).abs() < 1.0,
        "Expected x~100 at midpoint, got {}",
        mid[0]
    );
}

#[test]
fn catmull_rom_tangent_direction() {
    // Upward path: tangent should point positive y
    let spline = CatmullRomSpline::new(vec![[0.0, 0.0], [0.0, 100.0]]);

    let tan = spline.tangent([0.0, 0.0], 0.5);
    assert!(tan[1] > 0.0, "Expected positive y tangent for upward path");
}

#[test]
fn tangent_angle_helpers() {
    // Right: 0 degrees
    assert!((tangent_angle_deg([1.0, 0.0])).abs() < 1e-3);
    // Up: 90 degrees
    assert!((tangent_angle_deg([0.0, 1.0]) - 90.0).abs() < 1e-3);
    // Left: 180 degrees
    assert!((tangent_angle_deg([-1.0, 0.0]).abs() - 180.0).abs() < 1e-3);
    // Down: -90 degrees
    assert!((tangent_angle_deg([0.0, -1.0]) + 90.0).abs() < 1e-3);
}

// ── PolyPath ──────────────────────────────────────────────────────────────

#[test]
fn polypath_arc_length_parameterization() {
    // Straight horizontal path: arc-length = parametric (already uniform)
    let path = PolyPath::from_points(vec![[0.0, 0.0], [100.0, 0.0], [200.0, 0.0], [300.0, 0.0]]);

    // Arc length should be ~300
    assert!((path.arc_length() - 300.0).abs() < 5.0);

    // Quarter progress should give x~75
    let q = path.position(0.25);
    assert!((q[0] - 75.0).abs() < 10.0, "Expected x~75, got {}", q[0]);
}

#[test]
fn polypath_offsets_restrict_range() {
    let path = PolyPath::from_points(vec![[0.0, 0.0], [200.0, 0.0]])
        .start_offset(0.25)
        .end_offset(0.75);

    let start = path.position(0.0);
    let end = path.position(1.0);

    // Should traverse only the middle 50% of the path
    assert!(
        start[0] > 30.0,
        "start_offset should skip beginning: {}",
        start[0]
    );
    assert!(end[0] < 170.0, "end_offset should stop early: {}", end[0]);
}

#[test]
fn polypath_auto_rotate() {
    // Path going right then up
    let path = PolyPath::from_points(vec![[0.0, 0.0], [100.0, 0.0], [100.0, 100.0]]);

    // Near start: heading right (~0 degrees)
    let rot_start = path.rotation_deg(0.05);
    assert!(
        rot_start.abs() < 30.0,
        "Near start should be ~0 deg, got {rot_start}"
    );

    // Near end: heading up (~90 degrees)
    let rot_end = path.rotation_deg(0.95);
    assert!(
        (rot_end - 90.0).abs() < 30.0,
        "Near end should be ~90 deg, got {rot_end}"
    );
}

// ── CompoundPath ──────────────────────────────────────────────────────────

#[test]
fn compound_path_mixed_segments() {
    let path = CompoundPath::new(vec![
        PathCommand::MoveTo([0.0, 0.0]),
        PathCommand::CubicTo {
            control1: [33.0, 66.0],
            control2: [66.0, 66.0],
            end: [100.0, 0.0],
        },
        PathCommand::LineTo([200.0, 0.0]),
    ]);

    assert_eq!(path.segment_count(), 2);

    let start = path.position(0.0);
    let end = path.position(1.0);
    assert!((start[0]).abs() < 1.0);
    assert!((end[0] - 200.0).abs() < 1.0);

    // Cubic part should curve above y=0
    let quarter = path.position(0.25);
    assert!(
        quarter[1] > 5.0,
        "Cubic segment should curve up, got y={}",
        quarter[1]
    );
}

#[test]
fn compound_path_closed_triangle() {
    let path = CompoundPath::new(vec![
        PathCommand::MoveTo([0.0, 0.0]),
        PathCommand::LineTo([100.0, 0.0]),
        PathCommand::LineTo([50.0, 86.6]),
        PathCommand::Close,
    ]);

    assert_eq!(path.segment_count(), 3);

    // End should return to start
    let end = path.position(1.0);
    assert!(
        (end[0]).abs() < 2.0,
        "Should close near origin, got x={}",
        end[0]
    );
    assert!(
        (end[1]).abs() < 2.0,
        "Should close near origin, got y={}",
        end[1]
    );
}

// ── SvgPathParser ─────────────────────────────────────────────────────────

#[test]
fn svg_path_full_pipeline() {
    // Parse SVG → CompoundPath → evaluate
    let svg_d = "M 0 0 C 50 100 100 100 150 0 L 200 0";
    let commands = SvgPathParser::parse(svg_d);
    let path = CompoundPath::new(commands);

    let start = path.position(0.0);
    let end = path.position(1.0);
    assert!((start[0]).abs() < 1.0);
    assert!((end[0] - 200.0).abs() < 1.0);
}

#[test]
fn svg_path_relative_commands() {
    let cmds = SvgPathParser::parse("M 0 0 l 100 0 l 0 100");
    let path = CompoundPath::new(cmds);

    let end = path.position(1.0);
    assert!((end[0] - 100.0).abs() < 1.0);
    assert!((end[1] - 100.0).abs() < 1.0);
}

#[test]
fn svg_path_with_offsets() {
    let cmds = SvgPathParser::parse("M 0 0 L 100 0 L 200 0 L 300 0");
    let path = CompoundPath::new(cmds).start_offset(0.25).end_offset(0.75);

    let start = path.position(0.0);
    let end = path.position(1.0);

    // Should only traverse middle 50%
    assert!(start[0] > 50.0, "Expected start near 75, got {}", start[0]);
    assert!(end[0] < 250.0, "Expected end near 225, got {}", end[0]);
}

#[test]
fn svg_path_with_auto_rotate() {
    let cmds = SvgPathParser::parse("M 0 0 L 100 0 L 100 100");
    let path = CompoundPath::new(cmds).rotation_offset(45.0);

    let rot = path.rotation_deg(0.05);
    // Near start heading right: tangent ~0° + offset 45° = ~45°
    assert!(
        (rot - 45.0).abs() < 15.0,
        "Expected ~45 deg with offset, got {rot}"
    );
}

// ── Easing: CubicBezier and Steps ─────────────────────────────────────────

#[test]
fn cubic_bezier_easing_with_tween() {
    use spanda::traits::Update;
    use spanda::{Easing, Tween};

    let mut tween = Tween::new(0.0_f32, 100.0)
        .duration(1.0)
        .easing(Easing::CubicBezier(0.25, 0.1, 0.25, 1.0))
        .build();

    for _ in 0..10 {
        tween.update(0.1);
    }

    assert!(tween.is_complete());
    assert!((tween.value() - 100.0).abs() < 1e-4);
}

#[test]
fn steps_easing_with_tween() {
    use spanda::traits::Update;
    use spanda::{Easing, Tween};

    let mut tween = Tween::new(0.0_f32, 100.0)
        .duration(1.0)
        .easing(Easing::Steps(4))
        .build();

    // At t=0.3, steps(4) -> step 1/4 = 25.0
    tween.update(0.3);
    let val = tween.value();
    assert!(
        (val - 25.0).abs() < 1e-3,
        "Expected 25 at t=0.3 with Steps(4), got {val}"
    );
}
