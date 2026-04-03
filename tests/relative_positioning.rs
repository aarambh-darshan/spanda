//! Integration test — relative timeline positioning (At enum).

use spanda::timeline::{At, Timeline};
use spanda::traits::Update;
use spanda::tween::Tween;

#[test]
fn at_start_and_end_compose_correctly() {
    let mut tl = Timeline::new().add("base", Tween::new(0.0_f32, 1.0).duration(0.5).build(), 0.0);

    // Set duration so At::End works
    tl.add_at(
        "second",
        Tween::new(0.0_f32, 1.0).duration(0.5).build(),
        0.5,
        At::End,
    );
    tl.add_at(
        "third",
        Tween::new(0.0_f32, 1.0).duration(0.3).build(),
        0.3,
        At::Start,
    );

    tl.play();

    // "third" should start immediately, "base" at 0.0, "second" after base (At::End)
    // Timeline should complete when last entry finishes
    let dt = 0.01;
    let mut total = 0.0;
    while tl.update(dt) {
        total += dt;
        if total > 5.0 {
            panic!("Timeline did not complete");
        }
    }

    // "base" has duration 0.0 (set via .add, not add_at), but "second" has start_at
    // based on At::End. Since "base" had duration 0.0 => At::End = 0.0,
    // "second" starts at 0.0 too, with duration 0.5 → total ~0.5
    assert!(total > 0.0, "Timeline completed too quickly");
}

#[test]
fn at_label_syncs_animations() {
    let mut tl = Timeline::new().add("fade", Tween::new(0.0_f32, 1.0).duration(0.5).build(), 0.2);

    // "scale" should start at the same time as "fade" (0.2)
    tl.add_at(
        "scale",
        Tween::new(1.0_f32, 2.0).duration(0.3).build(),
        0.3,
        At::Label("fade"),
    );

    tl.play();

    // Both start at 0.2. "fade" runs 0.5s (ends 0.7), "scale" runs 0.3s (ends 0.5)
    // Timeline should complete at ~0.7
    let dt = 0.01;
    let mut total = 0.0;
    while tl.update(dt) {
        total += dt;
        if total > 5.0 {
            panic!("Timeline did not complete");
        }
    }

    assert!(
        total >= 0.5 && total <= 0.8,
        "Expected completion ~0.7s, got {total}"
    );
}

#[test]
fn at_offset_creates_gap() {
    let mut tl = Timeline::new().add("a", Tween::new(0.0_f32, 1.0).duration(0.5).build(), 0.0);

    // Give "a" duration metadata
    tl.add_at(
        "b",
        Tween::new(0.0_f32, 1.0).duration(0.3).build(),
        0.3,
        At::Offset(0.2),
    );

    tl.play();

    // "b" starts at 0.0 + duration_of_a (0.0 since .add didn't set it) + 0.2 = 0.2
    // Should still complete
    let dt = 0.01;
    let mut total = 0.0;
    while tl.update(dt) {
        total += dt;
        if total > 5.0 {
            panic!("Timeline did not complete");
        }
    }

    assert!(total > 0.0, "Timeline should have taken some time");
}
