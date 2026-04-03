//! Integration test — multi-step sequence completes in order.

use spanda::easing::Easing;
use spanda::timeline::Sequence;
use spanda::traits::Update;
use spanda::tween::Tween;

#[test]
fn sequence_completes_in_order() {
    let mut timeline = Sequence::new()
        .then(
            Tween::new(0.0_f32, 100.0)
                .duration(0.5)
                .easing(Easing::EaseOutCubic)
                .build(),
            0.5,
        )
        .gap(0.1)
        .then(
            Tween::new(100.0_f32, 200.0)
                .duration(0.3)
                .easing(Easing::EaseInOutQuad)
                .build(),
            0.3,
        )
        .gap(0.05)
        .then(
            Tween::new(200.0_f32, 0.0)
                .duration(0.4)
                .easing(Easing::EaseInBack)
                .build(),
            0.4,
        )
        .build();

    timeline.play();

    let dt = 0.01;
    let mut total = 0.0;

    while timeline.update(dt) {
        total += dt;
        if total > 10.0 {
            panic!("Timeline did not complete within 10 seconds");
        }
    }

    // Total expected: 0.5 + 0.1 + 0.3 + 0.05 + 0.4 = 1.35s
    assert!(total >= 1.3 && total <= 1.5, "Expected ~1.35s, got {total}");
}

#[test]
fn concurrent_timeline_completes() {
    use spanda::timeline::Timeline;

    let mut timeline = Timeline::new()
        .add("fade", Tween::new(0.0_f32, 1.0).duration(0.5).build(), 0.0)
        .add(
            "slide",
            Tween::new(0.0_f32, 100.0).duration(0.8).build(),
            0.0,
        )
        .add("scale", Tween::new(0.5_f32, 1.0).duration(0.3).build(), 0.4);

    timeline.play();

    let dt = 0.01;
    let mut total = 0.0;

    while timeline.update(dt) {
        total += dt;
        if total > 10.0 {
            panic!("Concurrent timeline did not complete");
        }
    }

    // Should complete when the longest entry finishes: slide at 0.8s
    assert!(total >= 0.7 && total <= 0.9, "Expected ~0.8s, got {total}");
}
