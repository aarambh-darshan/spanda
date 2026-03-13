//! Integration test — long-running looping keyframe track.

use spanda::keyframe::{KeyframeTrack, Loop};
use spanda::traits::Update;

#[test]
fn forever_loop_runs_many_cycles() {
    let mut track = KeyframeTrack::new()
        .push(0.0, 0.0_f32)
        .push(0.5, 100.0)
        .push(1.0, 0.0)
        .looping(Loop::Forever);

    // Run for 10 seconds at 60fps
    for _ in 0..600 {
        assert!(track.update(1.0 / 60.0));
    }
    assert!(!track.is_complete());

    // Value should be oscillating between 0 and 100
    let v = track.value();
    assert!(v >= -1.0 && v <= 101.0, "Value out of range: {v}");
}

#[test]
fn ping_pong_stays_in_range() {
    let mut track = KeyframeTrack::new()
        .push(0.0, 0.0_f32)
        .push(1.0, 100.0)
        .looping(Loop::PingPong);

    for _ in 0..600 {
        track.update(1.0 / 60.0);
        let v = track.value();
        assert!(
            v >= -1.0 && v <= 101.0,
            "PingPong value out of range: {v}"
        );
    }
}

#[test]
fn times_loop_completes_exactly() {
    let mut track = KeyframeTrack::new()
        .push(0.0, 0.0_f32)
        .push(1.0, 100.0)
        .looping(Loop::Times(3));

    let dt = 0.1;
    let mut total_time = 0.0;

    while track.update(dt) {
        total_time += dt;
        if total_time > 10.0 {
            panic!("Loop::Times(3) did not complete within expected time");
        }
    }

    assert!(track.is_complete());
    assert!(total_time >= 2.9 && total_time <= 3.1,
            "Expected ~3.0s, got {total_time}");
}
