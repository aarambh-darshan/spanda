//! Integration test — scroll-linked animations.

use spanda::clock::Clock;
use spanda::easing::Easing;
use spanda::scroll::{ScrollClock, ScrollDriver};
use spanda::traits::Update;
use spanda::tween::Tween;

#[test]
fn scroll_driver_completes_at_full_scroll() {
    let mut driver = ScrollDriver::new(0.0, 1000.0);
    driver.add(
        Tween::new(0.0_f32, 100.0)
            .duration(1.0)
            .easing(Easing::EaseOutCubic)
            .build(),
    );
    assert_eq!(driver.active_count(), 1);

    // Scroll to full extent
    driver.set_position(1000.0);
    assert_eq!(driver.active_count(), 0);
}

#[test]
fn scroll_driver_partial_progress() {
    let mut driver = ScrollDriver::new(0.0, 100.0);
    driver.add(Tween::new(0.0_f32, 100.0).duration(1.0).build());

    // Scroll halfway
    driver.set_position(50.0);
    assert_eq!(driver.active_count(), 1);
    assert!((driver.progress() - 0.5).abs() < 1e-6);
}

#[test]
fn scroll_clock_drives_tween_directly() {
    let mut clock = ScrollClock::new(0.0, 200.0);
    let mut tween = Tween::new(0.0_f32, 100.0).duration(1.0).build();

    // Scroll to 50% (100 of 200)
    clock.set_position(100.0);
    let dt = clock.delta();
    tween.update(dt);

    assert!(
        (tween.value() - 50.0).abs() < 1e-4,
        "Expected ~50, got {}",
        tween.value()
    );
}

#[test]
fn scroll_driver_multiple_animations() {
    let mut driver = ScrollDriver::new(0.0, 100.0);
    driver.add(Tween::new(0.0_f32, 1.0).duration(0.5).build());
    driver.add(Tween::new(0.0_f32, 1.0).duration(1.0).build());
    assert_eq!(driver.active_count(), 2);

    // Scroll to 50% → first completes (duration 0.5), second still running
    driver.set_position(50.0);
    assert_eq!(driver.active_count(), 1);

    // Scroll to 100% → second completes
    driver.set_position(100.0);
    assert_eq!(driver.active_count(), 0);
}

#[test]
fn scroll_clock_incremental_scrolling() {
    let mut clock = ScrollClock::new(0.0, 100.0);
    let mut tween = Tween::new(0.0_f32, 100.0).duration(1.0).build();

    // Simulate incremental scroll events
    for i in 1..=100 {
        clock.set_position(i as f32);
        let dt = clock.delta();
        tween.update(dt);
    }

    assert!(tween.is_complete());
    assert!(
        (tween.value() - 100.0).abs() < 1e-4,
        "Expected 100, got {}",
        tween.value()
    );
}
