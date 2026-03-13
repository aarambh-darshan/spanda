//! Full tween lifecycle using MockClock — deterministic integration test.

use spanda::clock::{Clock, MockClock};
use spanda::easing::Easing;
use spanda::traits::Update;
use spanda::tween::Tween;

#[test]
fn tween_lifecycle_with_mock_clock() {
    let mut tween = Tween::new(0.0_f32, 100.0)
        .duration(1.0)
        .easing(Easing::EaseOutCubic)
        .delay(0.5)
        .build();

    let mut clock = MockClock::new(0.1); // 10 fps

    // During delay (5 ticks = 0.5s)
    for _ in 0..5 {
        let dt = clock.delta();
        tween.update(dt);
    }
    // Should still be near start value
    assert!((tween.value() - 0.0).abs() < 1.0);

    // Animation phase (10 ticks = 1.0s)
    for _ in 0..10 {
        let dt = clock.delta();
        tween.update(dt);
    }

    assert!(tween.is_complete());
    assert!((tween.value() - 100.0).abs() < 1e-4);
}

#[test]
fn tween_with_all_easings() {
    for easing in Easing::all_named() {
        let mut tween = Tween::new(0.0_f32, 1.0)
            .duration(1.0)
            .easing(easing.clone())
            .build();

        let mut clock = MockClock::new(0.01);

        // Run for 105 ticks to ensure we cross the 1.0s duration despite f32 precision
        for _ in 0..105 {
            tween.update(clock.delta());
        }

        assert!(
            tween.is_complete(),
            "Tween with {} did not complete",
            easing.name()
        );
        assert!(
            (tween.value() - 1.0).abs() < 1e-4,
            "Tween with {} ended at {} instead of 1.0",
            easing.name(),
            tween.value()
        );
    }
}
