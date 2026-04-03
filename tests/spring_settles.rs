//! Integration test — spring reaches target within N steps.

use spanda::spring::{Spring, SpringConfig};
use spanda::traits::Update;

#[test]
fn all_presets_settle_to_target() {
    let configs = [
        ("gentle", SpringConfig::gentle()),
        ("wobbly", SpringConfig::wobbly()),
        ("stiff", SpringConfig::stiff()),
        ("slow", SpringConfig::slow()),
    ];

    for (name, config) in &configs {
        let mut spring = Spring::new(config.clone());
        spring.set_target(100.0);

        let max_frames = 5000; // ~83 seconds at 60fps — plenty
        let mut settled = false;

        for _ in 0..max_frames {
            if !spring.update(1.0 / 60.0) {
                settled = true;
                break;
            }
        }

        assert!(
            settled,
            "Spring preset '{}' did not settle within {max_frames} frames",
            name
        );
        assert!(
            (spring.position() - 100.0).abs() < 0.01,
            "Spring '{}' settled at {} instead of 100.0",
            name,
            spring.position()
        );
    }
}

#[test]
fn spring_settles_within_reasonable_time() {
    let mut spring = Spring::new(SpringConfig::default());
    spring.set_target(50.0);

    let dt = 1.0 / 60.0;
    let mut frames = 0;

    while spring.update(dt) {
        frames += 1;
        if frames > 10000 {
            panic!("Default spring took too long to settle");
        }
    }

    // Default config should settle in under 600 frames (~10s)
    assert!(
        frames < 600,
        "Default spring took {frames} frames to settle (expected < 600)"
    );
}

#[test]
fn spring_target_change_mid_animation() {
    let mut spring = Spring::new(SpringConfig::stiff());
    spring.set_target(100.0);

    // Animate halfway
    for _ in 0..30 {
        spring.update(1.0 / 60.0);
    }
    assert!(!spring.is_settled());

    // Change target mid-flight
    spring.set_target(200.0);
    assert!(!spring.is_settled());

    // Let it settle
    for _ in 0..5000 {
        if !spring.update(1.0 / 60.0) {
            break;
        }
    }

    assert!(spring.is_settled());
    assert!(
        (spring.position() - 200.0).abs() < 0.01,
        "Spring settled at {} instead of 200.0",
        spring.position()
    );
}
