//! Integration tests for spanda v0.8.0 pure-math features.

use spanda::easing::Easing;
use spanda::traits::Update;

// ── DrawSVG ──────────────────────────────────────────────────────────────────

#[test]
fn draw_on_full_lifecycle() {
    let mut tween = spanda::draw_on(300.0)
        .duration(1.0)
        .easing(Easing::EaseInOutCubic)
        .build();

    assert!((tween.value() - 300.0).abs() < 1e-6);

    tween.update(0.5);
    let mid = tween.value();
    assert!(mid > 0.0 && mid < 300.0, "mid={mid}");

    tween.update(0.5);
    assert!((tween.value()).abs() < 1e-6);
    assert!(tween.is_complete());
}

#[test]
fn draw_on_reverse_full_lifecycle() {
    let mut tween = spanda::draw_on_reverse(200.0).duration(0.5).build();

    tween.update(0.5);
    assert!((tween.value() - 200.0).abs() < 1e-6);
}

// ── MorphPath ────────────────────────────────────────────────────────────────

#[test]
fn morph_path_lifecycle() {
    let circle = vec![[100.0, 0.0], [0.0, 100.0], [-100.0, 0.0], [0.0, -100.0]];
    let square = vec![
        [100.0, 100.0],
        [-100.0, 100.0],
        [-100.0, -100.0],
        [100.0, -100.0],
    ];

    let mut morph = spanda::MorphPath::new(circle, square)
        .duration(1.0)
        .easing(Easing::EaseOutCubic)
        .build();

    let start = morph.value();
    assert!((start[0][0] - 100.0).abs() < 1e-6);

    morph.update(0.5);
    let mid = morph.value();
    // Mid should be between circle and square
    assert!(mid[0][0] > 99.0 && mid[0][0] < 101.0);

    morph.update(0.5);
    let end = morph.value();
    assert!((end[0][0] - 100.0).abs() < 1e-5);
    assert!((end[0][1] - 100.0).abs() < 1e-5);
    assert!(morph.is_complete());
}

#[test]
fn morph_with_resample() {
    let from = vec![[0.0, 0.0], [100.0, 0.0]];
    let to = vec![[0.0, 0.0], [50.0, 50.0], [100.0, 0.0], [150.0, -50.0]];

    let morph = spanda::MorphPath::new(from, to).duration(1.0).build();

    // Auto-resampled to 4 points
    assert_eq!(morph.value().len(), 4);
}

// ── Inertia ──────────────────────────────────────────────────────────────────

#[test]
fn inertia_settles() {
    let mut inertia = spanda::Inertia::new(spanda::InertiaConfig::snappy())
        .with_velocity(1000.0)
        .with_position(0.0);

    let mut frames = 0u32;
    while inertia.update(1.0 / 60.0) {
        frames += 1;
        if frames > 10000 {
            panic!("inertia never settled");
        }
    }

    assert!(inertia.is_settled());
    assert!(inertia.position() > 0.0);
    assert!(frames > 0 && frames < 1000);
}

#[test]
fn inertia_n_2d_settles() {
    let mut inertia = spanda::InertiaN::new(spanda::InertiaConfig::default_flick(), [0.0_f32, 0.0])
        .with_velocity([500.0, -300.0]);

    while inertia.update(1.0 / 60.0) {}

    assert!(inertia.is_settled());
    let pos = inertia.position();
    assert!(pos[0] > 0.0);
    assert!(pos[1] < 0.0);
}

// ── Drag ─────────────────────────────────────────────────────────────────────

#[test]
fn drag_to_inertia_flow() {
    let mut drag = spanda::DragState::new();

    drag.on_pointer_down(100.0, 200.0);
    drag.on_pointer_move(120.0, 210.0, 1.0 / 60.0);
    drag.on_pointer_move(150.0, 220.0, 1.0 / 60.0);
    drag.on_pointer_move(200.0, 240.0, 1.0 / 60.0);

    assert!(drag.is_dragging());
    let mut inertia = drag.on_pointer_up();
    assert!(!drag.is_dragging());

    // Inertia should carry momentum
    let start_pos = inertia.position();
    inertia.update(0.1);
    let end_pos = inertia.position();
    assert!(end_pos[0] > start_pos[0], "Should continue in X");
}

// ── Easing variants ──────────────────────────────────────────────────────────

#[test]
fn all_new_easings_endpoints_lifecycle() {
    let variants: Vec<Easing> = vec![
        Easing::RoughEase {
            strength: 0.3,
            points: 20,
            seed: 1,
        },
        Easing::SlowMo {
            ratio: 0.5,
            power: 0.7,
            yoyo_mode: false,
        },
        Easing::ExpoScale {
            start_scale: 1.0,
            end_scale: 100.0,
        },
        Easing::Wiggle {
            frequency: 3.0,
            amplitude: 0.2,
        },
        Easing::CustomBounce {
            strength: 0.5,
            squash: 0.3,
        },
    ];

    for easing in &variants {
        let t0 = easing.apply(0.0);
        let t1 = easing.apply(1.0);
        assert!(t0.abs() < 0.05, "{:?} apply(0) = {t0}", easing);
        assert!((t1 - 1.0).abs() < 0.05, "{:?} apply(1) = {t1}", easing);
    }
}

#[test]
fn new_easings_work_in_tweens() {
    let variants: Vec<Easing> = vec![
        Easing::RoughEase {
            strength: 0.2,
            points: 10,
            seed: 42,
        },
        Easing::SlowMo {
            ratio: 0.3,
            power: 0.5,
            yoyo_mode: false,
        },
        Easing::ExpoScale {
            start_scale: 1.0,
            end_scale: 10.0,
        },
        Easing::Wiggle {
            frequency: 2.0,
            amplitude: 0.1,
        },
        Easing::CustomBounce {
            strength: 0.5,
            squash: 0.0,
        },
    ];

    for easing in variants {
        let mut tween = spanda::Tween::new(0.0_f32, 100.0)
            .duration(1.0)
            .easing(easing)
            .build();

        tween.update(0.5);
        let mid = tween.value();
        assert!(mid > -20.0 && mid < 120.0, "mid out of range: {mid}");

        tween.update(0.5);
        assert!((tween.value() - 100.0).abs() < 5.0);
    }
}

// ── SplitText (pure part) ────────────────────────────────────────────────────

#[test]
fn split_text_integration() {
    use spanda::integrations::split_text::SplitText;

    let split = SplitText::from_str("Animate every letter");
    assert_eq!(split.word_count(), 3);
    assert_eq!(split.words()[0].text, "Animate");
    assert_eq!(split.words()[2].text, "letter");

    // chars include spaces
    let non_space_chars: Vec<_> = split.chars().iter().filter(|c| c.ch != ' ').collect();
    assert_eq!(non_space_chars.len(), 18); // "Animateeveryletter" = 18 chars
}
