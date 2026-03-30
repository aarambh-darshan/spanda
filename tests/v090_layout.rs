//! Integration tests for spanda v0.9.0 — Layout Animation.

use spanda::easing::Easing;
use spanda::layout::{LayoutAnimator, Rect, SharedElementTransition};
use spanda::traits::Update;

// ── LayoutAnimator ──────────────────────────────────────────────────────────

#[test]
fn layout_animator_track_and_transition() {
    let mut la = LayoutAnimator::new();
    la.track("card-1", Rect::new(0.0, 0.0, 200.0, 100.0));
    la.track("card-2", Rect::new(0.0, 120.0, 200.0, 100.0));

    let transitions = la.compute_transitions(
        &[
            ("card-1", Rect::new(0.0, 120.0, 200.0, 100.0)),
            ("card-2", Rect::new(0.0, 0.0, 200.0, 100.0)),
        ],
        0.4,
        Easing::EaseOutCubic,
    );

    assert_eq!(transitions.len(), 2, "Both cards should have transitions");
    assert!(la.is_animating());
}

#[test]
fn layout_animator_no_change_no_transition() {
    let mut la = LayoutAnimator::new();
    la.track("a", Rect::new(10.0, 20.0, 100.0, 50.0));

    let transitions = la.compute_transitions(
        &[("a", Rect::new(10.0, 20.0, 100.0, 50.0))],
        0.4,
        Easing::Linear,
    );

    assert_eq!(transitions.len(), 0, "No movement = no transition");
}

#[test]
fn layout_animator_update_completes_animations() {
    let mut la = LayoutAnimator::new();
    la.track("box", Rect::new(0.0, 0.0, 100.0, 100.0));
    la.compute_transitions(
        &[("box", Rect::new(200.0, 200.0, 100.0, 100.0))],
        0.3,
        Easing::Linear,
    );

    assert!(la.is_animating());

    // Tick for the full duration
    la.update(0.3);

    // Animations should be done
    assert!(!la.is_animating());
}

#[test]
fn layout_animator_css_transform_during_animation() {
    let mut la = LayoutAnimator::new();
    la.track("el", Rect::new(0.0, 0.0, 100.0, 50.0));
    la.compute_transitions(
        &[("el", Rect::new(100.0, 100.0, 100.0, 50.0))],
        0.5,
        Easing::Linear,
    );

    let css = la.css_transform("el");
    assert!(css.is_some());
    let css_str = css.unwrap();
    assert!(css_str.contains("translate("), "css = {css_str}");
    assert!(css_str.contains("scale("), "css = {css_str}");
}

#[test]
fn layout_animator_reorder_list() {
    let mut la = LayoutAnimator::new();
    let old = &[
        ("item-a", Rect::new(0.0, 0.0, 300.0, 60.0)),
        ("item-b", Rect::new(0.0, 70.0, 300.0, 60.0)),
        ("item-c", Rect::new(0.0, 140.0, 300.0, 60.0)),
    ];
    let new = &[
        ("item-c", Rect::new(0.0, 0.0, 300.0, 60.0)),
        ("item-a", Rect::new(0.0, 70.0, 300.0, 60.0)),
        ("item-b", Rect::new(0.0, 140.0, 300.0, 60.0)),
    ];

    let transitions = la.animate_reorder(old, new, 0.4, Easing::EaseOutCubic);
    assert_eq!(transitions.len(), 3, "All 3 items should animate");
}

#[test]
fn layout_animator_enter_animation() {
    let mut la = LayoutAnimator::new();
    let anim = la.animate_enter(
        "new-card",
        Rect::new(50.0, 50.0, 200.0, 100.0),
        0.3,
        Easing::EaseOutCubic,
    );

    // Scale should start from 0 (entering from nothing)
    let (_, _, sx, sy) = anim.transform();
    assert!(sx.abs() < 0.01, "sx should be ~0 at start: {sx}");
    assert!(sy.abs() < 0.01, "sy should be ~0 at start: {sy}");
    assert_eq!(la.tracked_count(), 1);
}

#[test]
fn layout_animator_exit_animation() {
    let mut la = LayoutAnimator::new();
    la.track("leaving", Rect::new(10.0, 20.0, 100.0, 50.0));

    let anim = la.animate_exit("leaving", 0.3, Easing::EaseInCubic);
    assert!(anim.is_some(), "Should produce exit animation");
    assert_eq!(la.tracked_count(), 0, "Element should be untracked");

    // Complete the exit animation
    let mut anim = anim.unwrap();
    anim.update(0.3);
    assert!(anim.is_complete());
    let (_, _, sx, sy) = anim.transform();
    assert!((sx).abs() < 0.01, "sx should be ~0 at end: {sx}");
    assert!((sy).abs() < 0.01, "sy should be ~0 at end: {sy}");
}

// ── SharedElementTransition ─────────────────────────────────────────────────

#[test]
fn shared_element_transition_lifecycle() {
    let source = Rect::new(10.0, 10.0, 50.0, 50.0);
    let target = Rect::new(200.0, 300.0, 400.0, 300.0);
    let mut set = SharedElementTransition::new(source, target, 0.5, Easing::EaseOutCubic);

    assert!(!set.is_complete());

    // At t=0, transform should show the offset from target → source
    let css_start = set.css_transform();
    assert!(css_start.contains("translate("));

    // Complete
    set.update(0.5);
    assert!(set.is_complete());

    // At completion, transform should be identity
    let css_end = set.css_transform();
    assert!(css_end.contains("translate(0"));
}
