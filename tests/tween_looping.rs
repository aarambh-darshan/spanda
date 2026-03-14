//! Integration test — tween looping modes.

use spanda::keyframe::Loop;
use spanda::tween::Tween;
use spanda::traits::Update;

#[test]
fn tween_ping_pong_stays_in_range() {
    let mut t = Tween::new(0.0_f32, 100.0)
        .duration(1.0)
        .looping(Loop::PingPong)
        .build();

    for _ in 0..600 {
        t.update(1.0 / 60.0);
        let v = t.value();
        assert!(
            v >= -1.0 && v <= 101.0,
            "PingPong value out of range: {v}"
        );
    }
}

#[test]
fn tween_loop_forever_runs_many_cycles() {
    let mut t = Tween::new(0.0_f32, 100.0)
        .duration(1.0)
        .looping(Loop::Forever)
        .build();

    // Run for 10 seconds at 60fps
    for _ in 0..600 {
        assert!(t.update(1.0 / 60.0));
    }
    assert!(!t.is_complete());
}

#[test]
fn tween_loop_times_completes_exactly() {
    let mut t = Tween::new(0.0_f32, 100.0)
        .duration(1.0)
        .looping(Loop::Times(3))
        .build();

    let dt = 0.1;
    let mut total_time = 0.0;

    while t.update(dt) {
        total_time += dt;
        if total_time > 10.0 {
            panic!("Loop::Times(3) did not complete within expected time");
        }
    }

    assert!(t.is_complete());
    assert!(
        total_time >= 2.8 && total_time <= 3.2,
        "Expected ~3.0s, got {total_time}"
    );
}

#[test]
fn tween_ping_pong_returns_to_start() {
    let mut t = Tween::new(0.0_f32, 100.0)
        .duration(1.0)
        .looping(Loop::PingPong)
        .build();

    // Complete one full ping-pong cycle (forward + backward = 2.0s)
    for _ in 0..120 {
        t.update(1.0 / 60.0);
    }

    // Should be back near start value
    let v = t.value();
    assert!(
        v < 10.0,
        "After full ping-pong cycle, expected near 0.0 but got {v}"
    );
}
