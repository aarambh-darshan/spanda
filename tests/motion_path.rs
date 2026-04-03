//! Integration test — Bezier paths and MotionPath.

use spanda::easing::Easing;
use spanda::path::{BezierPath, MotionPath, MotionPathTween, PathEvaluate};
use spanda::traits::Update;

#[test]
fn cubic_bezier_s_curve_traversal() {
    let curve = BezierPath::cubic([0.0_f32, 0.0], [0.0, 100.0], [100.0, 100.0], [100.0, 0.0]);

    // Check start and end
    let start = curve.evaluate(0.0);
    let end = curve.evaluate(1.0);
    assert!((start[0]).abs() < 1e-6);
    assert!((start[1]).abs() < 1e-6);
    assert!((end[0] - 100.0).abs() < 1e-6);
    assert!((end[1]).abs() < 1e-6);

    // Midpoint should have significant y value
    let mid = curve.evaluate(0.5);
    assert!(mid[1] > 50.0, "Expected y > 50 at midpoint, got {}", mid[1]);
}

#[test]
fn quadratic_bezier_arc() {
    let curve = BezierPath::quadratic([0.0_f32, 0.0], [50.0, 100.0], [100.0, 0.0]);

    // Symmetric arc — x at midpoint should be ~50
    let mid = curve.evaluate(0.5);
    assert!(
        (mid[0] - 50.0).abs() < 1e-4,
        "Expected x~50, got {}",
        mid[0]
    );
    assert!(mid[1] > 40.0, "Expected y > 40 at midpoint, got {}", mid[1]);
}

#[test]
fn motion_path_multi_segment_traversal() {
    let path = MotionPath::new().line(0.0_f32, 50.0).line(50.0, 100.0);

    // t=0 → 0, t=0.5 → 50, t=1.0 → 100
    assert!((path.evaluate(0.0) - 0.0).abs() < 1e-6);
    assert!((path.evaluate(0.5) - 50.0).abs() < 1e-4);
    assert!((path.evaluate(1.0) - 100.0).abs() < 1e-4);

    // t=0.25 → midpoint of first segment → 25
    assert!((path.evaluate(0.25) - 25.0).abs() < 1e-4);
}

#[test]
fn motion_path_tween_drives_along_path() {
    let path = MotionPath::new()
        .line([0.0_f32, 0.0], [100.0, 0.0])
        .line([100.0, 0.0], [100.0, 100.0]);

    let mut tween = MotionPathTween::new(path)
        .duration(2.0)
        .easing(Easing::Linear);

    // At t=0.5 (1 second), should be at corner [100, 0]
    tween.update(1.0);
    let pos = tween.value();
    assert!(
        (pos[0] - 100.0).abs() < 1e-3,
        "Expected x~100, got {}",
        pos[0]
    );
    assert!((pos[1]).abs() < 1e-3, "Expected y~0, got {}", pos[1]);

    // At t=1.0 (2 seconds), should be at end [100, 100]
    tween.update(1.0);
    assert!(tween.is_complete());
    let final_pos = tween.value();
    assert!((final_pos[0] - 100.0).abs() < 1e-3);
    assert!((final_pos[1] - 100.0).abs() < 1e-3);
}

#[test]
fn motion_path_tween_works_in_driver() {
    use spanda::driver::AnimationDriver;

    let path = MotionPath::new().line(0.0_f32, 100.0);

    let tween = MotionPathTween::new(path).duration(1.0);

    let mut driver = AnimationDriver::new();
    driver.add(tween);
    assert_eq!(driver.active_count(), 1);

    driver.tick(1.0);
    assert_eq!(driver.active_count(), 0);
}

#[test]
fn motion_path_with_bezier_curves() {
    let path = MotionPath::new()
        .cubic([0.0_f32, 0.0], [50.0, 100.0], [100.0, 100.0], [150.0, 0.0])
        .line([150.0, 0.0], [200.0, 0.0]);

    let start = path.evaluate(0.0);
    let end = path.evaluate(1.0);
    assert!((start[0]).abs() < 1e-6);
    assert!((end[0] - 200.0).abs() < 1e-4);
}
