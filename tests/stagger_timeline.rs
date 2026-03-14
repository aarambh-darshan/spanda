//! Integration test — stagger utility.

use spanda::easing::Easing;
use spanda::timeline::stagger;
use spanda::traits::Update;
use spanda::tween::Tween;

#[test]
fn stagger_five_tweens_completes() {
    let tweens: Vec<_> = (0..5)
        .map(|i| {
            let end = (i + 1) as f32 * 20.0;
            (
                Tween::new(0.0_f32, end)
                    .duration(0.3)
                    .easing(Easing::EaseOutCubic)
                    .build(),
                0.3,
            )
        })
        .collect();

    let mut tl = stagger(tweens, 0.1);
    tl.play();

    let dt = 0.01;
    let mut total = 0.0;

    while tl.update(dt) {
        total += dt;
        if total > 5.0 {
            panic!("Stagger timeline did not complete");
        }
    }

    // Last tween starts at 0.4, runs 0.3 = total 0.7s
    assert!(
        total >= 0.65 && total <= 0.8,
        "Expected ~0.7s, got {total}"
    );
}

#[test]
fn stagger_with_time_scale() {
    let tweens: Vec<_> = (0..3)
        .map(|_| (Tween::new(0.0_f32, 1.0).duration(0.5).build(), 0.5))
        .collect();

    let mut tl = stagger(tweens, 0.2);
    tl.set_time_scale(2.0);
    tl.play();

    let dt = 0.01;
    let mut total = 0.0;

    while tl.update(dt) {
        total += dt;
        if total > 5.0 {
            panic!("Stagger timeline with time_scale did not complete");
        }
    }

    // At 2x speed, should complete in roughly half the time (~0.45s instead of ~0.9s)
    assert!(
        total >= 0.3 && total <= 0.55,
        "Expected ~0.45s at 2x speed, got {total}"
    );
}
