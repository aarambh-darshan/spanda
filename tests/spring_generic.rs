//! Integration tests for generic SpringN — multi-dimensional spring physics.

use spanda::spring::{SpringConfig, SpringN};
use spanda::traits::Update;

#[test]
fn spring_n_2d_settles_to_target() {
    let mut spring = SpringN::new(SpringConfig::default(), [0.0_f32, 0.0]);
    spring.set_target([100.0, 200.0]);

    for _ in 0..1000 {
        spring.update(1.0 / 60.0);
    }

    let pos = spring.position();
    assert!((pos[0] - 100.0).abs() < 0.1, "x did not settle: {}", pos[0]);
    assert!((pos[1] - 200.0).abs() < 0.1, "y did not settle: {}", pos[1]);
    assert!(spring.is_settled());
}

#[test]
fn spring_n_3d_settles_to_target() {
    let mut spring = SpringN::new(SpringConfig::stiff(), [0.0_f32, 0.0, 0.0]);
    spring.set_target([50.0, 100.0, 150.0]);

    for _ in 0..1000 {
        spring.update(1.0 / 60.0);
    }

    let pos = spring.position();
    assert!((pos[0] - 50.0).abs() < 0.1);
    assert!((pos[1] - 100.0).abs() < 0.1);
    assert!((pos[2] - 150.0).abs() < 0.1);
    assert!(spring.is_settled());
}

#[test]
fn spring_n_4d_settles_rgba() {
    let mut spring = SpringN::new(SpringConfig::gentle(), [0.0_f32; 4]);
    spring.set_target([1.0, 0.5, 0.0, 0.8]);

    for _ in 0..2000 {
        spring.update(1.0 / 60.0);
    }

    let pos = spring.position();
    assert!((pos[0] - 1.0).abs() < 0.01);
    assert!((pos[1] - 0.5).abs() < 0.01);
    assert!((pos[2] - 0.0).abs() < 0.01);
    assert!((pos[3] - 0.8).abs() < 0.01);
    assert!(spring.is_settled());
}

#[test]
fn spring_n_retarget_preserves_velocity() {
    let mut spring = SpringN::new(SpringConfig::wobbly(), [0.0_f32, 0.0]);
    spring.set_target([100.0, 100.0]);

    // Advance partway — spring should have velocity
    for _ in 0..20 {
        spring.update(1.0 / 60.0);
    }

    let vel_before = spring.velocity_components().to_vec();
    assert!(
        vel_before.iter().any(|v| v.abs() > 0.1),
        "should have velocity"
    );

    // Retarget mid-flight
    spring.set_target([200.0, 0.0]);

    // Velocity should still be present (not reset)
    let vel_after = spring.velocity_components().to_vec();
    assert!(
        vel_after.iter().any(|v| v.abs() > 0.1),
        "velocity preserved after retarget"
    );

    // Should settle to new target
    for _ in 0..2000 {
        spring.update(1.0 / 60.0);
    }

    let pos = spring.position();
    assert!((pos[0] - 200.0).abs() < 0.5);
    assert!((pos[1] - 0.0).abs() < 0.5);
}

#[test]
fn spring_n_wobbly_2d_overshoots() {
    let mut spring = SpringN::new(SpringConfig::wobbly(), [0.0_f32, 0.0]);
    spring.set_target([100.0, 100.0]);

    let mut max_x = 0.0_f32;
    let mut max_y = 0.0_f32;
    for _ in 0..500 {
        spring.update(1.0 / 60.0);
        let pos = spring.position();
        max_x = max_x.max(pos[0]);
        max_y = max_y.max(pos[1]);
    }

    assert!(max_x > 100.0, "2D wobbly should overshoot x: {max_x}");
    assert!(max_y > 100.0, "2D wobbly should overshoot y: {max_y}");
}

#[test]
fn spring_n_f32_matches_spring_behaviour() {
    use spanda::spring::Spring;

    let config = SpringConfig::default();
    let mut spring_f32 = Spring::new(config.clone());
    let mut spring_n = SpringN::new(config, 0.0_f32);

    spring_f32.set_target(100.0);
    spring_n.set_target(100.0);

    for _ in 0..200 {
        spring_f32.update(1.0 / 60.0);
        spring_n.update(1.0 / 60.0);

        // Both should track very closely
        assert!(
            (spring_f32.position() - spring_n.position()).abs() < 0.01,
            "Spring and SpringN<f32> diverged: {} vs {}",
            spring_f32.position(),
            spring_n.position()
        );
    }
}
