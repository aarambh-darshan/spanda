//! Integration tests for spanda v0.9.0 — Gesture Recognition.

use spanda::drag::PointerData;
use spanda::gesture::{Gesture, GestureConfig, GestureRecognizer, SwipeDirection};

fn pointer(id: i32, x: f32, y: f32) -> PointerData {
    PointerData {
        x,
        y,
        pressure: 0.5,
        pointer_id: id,
    }
}

// ── Tap ─────────────────────────────────────────────────────────────────────

#[test]
fn gesture_tap_detected() {
    let mut r = GestureRecognizer::new();
    r.on_pointer_down(pointer(0, 200.0, 300.0));
    r.update(0.05);
    let g = r.on_pointer_up(pointer(0, 201.0, 300.0));
    match g {
        Some(Gesture::Tap { position }) => {
            assert!((position[0] - 200.0).abs() < 1e-6);
            assert!((position[1] - 300.0).abs() < 1e-6);
        }
        other => panic!("expected Tap, got {:?}", other),
    }
}

#[test]
fn gesture_tap_rejected_if_too_far() {
    let mut r = GestureRecognizer::new();
    r.on_pointer_down(pointer(0, 100.0, 100.0));
    r.update(0.05);
    r.on_pointer_move(pointer(0, 200.0, 100.0)); // 100px movement
    let g = r.on_pointer_up(pointer(0, 200.0, 100.0));
    assert!(
        !matches!(g, Some(Gesture::Tap { .. })),
        "Should not be a tap after 100px movement"
    );
}

// ── Long Press ──────────────────────────────────────────────────────────────

#[test]
fn gesture_long_press_detected() {
    let mut r = GestureRecognizer::new();
    r.on_pointer_down(pointer(0, 100.0, 100.0));
    // Hold for 0.6s (threshold is 0.5s)
    let g = r.update(0.6);
    match g {
        Some(Gesture::LongPress { position, duration }) => {
            assert!((position[0] - 100.0).abs() < 1e-6);
            assert!(duration >= 0.5);
        }
        other => panic!("expected LongPress, got {:?}", other),
    }
}

#[test]
fn gesture_long_press_cancelled_by_move() {
    let mut r = GestureRecognizer::new();
    r.on_pointer_down(pointer(0, 100.0, 100.0));
    r.update(0.2);
    r.on_pointer_move(pointer(0, 200.0, 200.0)); // big movement
    let g = r.update(0.5);
    assert!(
        !matches!(g, Some(Gesture::LongPress { .. })),
        "Long press should be cancelled by large movement"
    );
}

// ── Swipe ───────────────────────────────────────────────────────────────────

#[test]
fn gesture_swipe_right() {
    let mut r = GestureRecognizer::new();
    r.on_pointer_down(pointer(0, 100.0, 100.0));
    r.update(0.1); // 100ms
    r.on_pointer_move(pointer(0, 300.0, 105.0)); // 200px right
    let g = r.on_pointer_up(pointer(0, 300.0, 105.0));
    match g {
        Some(Gesture::Swipe {
            direction,
            velocity,
            ..
        }) => {
            assert_eq!(direction, SwipeDirection::Right);
            assert!(velocity > 300.0, "velocity = {velocity}");
        }
        other => panic!("expected Swipe Right, got {:?}", other),
    }
}

#[test]
fn gesture_swipe_up() {
    let mut r = GestureRecognizer::new();
    r.on_pointer_down(pointer(0, 100.0, 400.0));
    r.update(0.08);
    r.on_pointer_move(pointer(0, 105.0, 100.0)); // 300px up
    let g = r.on_pointer_up(pointer(0, 105.0, 100.0));
    match g {
        Some(Gesture::Swipe { direction, .. }) => {
            assert_eq!(direction, SwipeDirection::Up);
        }
        other => panic!("expected Swipe Up, got {:?}", other),
    }
}

#[test]
fn gesture_swipe_rejected_slow() {
    let mut r = GestureRecognizer::new();
    r.on_pointer_down(pointer(0, 100.0, 100.0));
    r.update(3.0); // 3 seconds — too slow
    r.on_pointer_move(pointer(0, 200.0, 100.0)); // 100px
    let g = r.on_pointer_up(pointer(0, 200.0, 100.0));
    // 100px / 3s ≈ 33 px/s — way below 300 px/s threshold
    assert!(
        !matches!(g, Some(Gesture::Swipe { .. })),
        "Should not be a swipe at 33 px/s"
    );
}

// ── Pinch ───────────────────────────────────────────────────────────────────

#[test]
fn gesture_pinch_zoom() {
    let mut r = GestureRecognizer::new();
    // Two fingers 100px apart
    r.on_pointer_down(pointer(0, 100.0, 200.0));
    r.on_pointer_down(pointer(1, 200.0, 200.0));
    // Spread to 300px apart
    r.on_pointer_move(pointer(0, 50.0, 200.0));
    let g = r.on_pointer_move(pointer(1, 350.0, 200.0));
    match g {
        Some(Gesture::Pinch { scale, center }) => {
            assert!(scale > 1.5, "scale = {scale}");
            assert!((center[1] - 200.0).abs() < 1e-4);
        }
        other => panic!("expected Pinch, got {:?}", other),
    }
}

// ── Rotation ────────────────────────────────────────────────────────────────

#[test]
fn gesture_rotation() {
    let mut r = GestureRecognizer::with_config(GestureConfig {
        rotation_min_angle: 0.01,
        pinch_min_scale_delta: 100.0, // suppress pinch
        ..Default::default()
    });
    // Initial: horizontal pair at y=200
    r.on_pointer_down(pointer(0, 100.0, 200.0));
    r.on_pointer_down(pointer(1, 300.0, 200.0));
    // Rotate: keep first finger, move second vertically (equidistant)
    // New angle: atan2(-100, 100) ≈ -0.785
    r.on_pointer_move(pointer(1, 200.0, 100.0));
    let g = r.on_pointer_move(pointer(0, 100.0, 200.0));
    match g {
        Some(Gesture::Rotate { angle, .. }) => {
            assert!(angle.abs() > 0.3, "angle = {angle}");
        }
        other => panic!("expected Rotate, got {:?}", other),
    }
}

// ── Config ──────────────────────────────────────────────────────────────────

#[test]
fn gesture_config_custom_thresholds() {
    let config = GestureConfig {
        tap_max_distance: 100.0,
        tap_max_duration: 2.0,
        long_press_threshold: 3.0, // prevent long press from firing during test
        ..Default::default()
    };
    let mut r = GestureRecognizer::with_config(config);
    r.on_pointer_down(pointer(0, 100.0, 100.0));
    r.update(1.0);
    r.on_pointer_move(pointer(0, 150.0, 120.0)); // 54px movement
    let g = r.on_pointer_up(pointer(0, 150.0, 120.0));
    // With generous threshold (100px max, 2s max), this should be a tap
    match g {
        Some(Gesture::Tap { .. }) => {}
        other => panic!("expected Tap with generous thresholds, got {:?}", other),
    }
}
