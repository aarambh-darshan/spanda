//! Automatic FLIP-style layout animation.
//!
//! `LayoutAnimator` tracks element positions by string ID and automatically
//! creates FLIP (First-Last-Invert-Play) animations when elements move.
//! Works with pure-math [`Rect`] structs — no DOM dependency.
//!
//! For DOM binding, enable the `wasm-dom` feature for
//! [`LayoutAnimator::track_element`] which calls `getBoundingClientRect()`.
//!
//! # Example
//!
//! ```rust
//! use spanda::layout::{LayoutAnimator, Rect};
//! use spanda::easing::Easing;
//!
//! let mut layout = LayoutAnimator::new();
//! layout.track("card-1", Rect::new(0.0, 0.0, 100.0, 50.0));
//! layout.track("card-2", Rect::new(0.0, 60.0, 100.0, 50.0));
//!
//! // After layout change — cards swapped
//! let transitions = layout.compute_transitions(
//!     &[
//!         ("card-1", Rect::new(0.0, 60.0, 100.0, 50.0)),
//!         ("card-2", Rect::new(0.0, 0.0, 100.0, 50.0)),
//!     ],
//!     0.4,
//!     Easing::EaseOutCubic,
//! );
//! assert_eq!(transitions.len(), 2);
//! ```

use crate::easing::Easing;
use crate::traits::Update;
use crate::tween::Tween;

#[cfg(not(feature = "std"))]
use alloc::{collections::BTreeMap as HashMap, format, string::String, vec::Vec};

#[cfg(feature = "std")]
use std::collections::HashMap;

// ── Rect ────────────────────────────────────────────────────────────────────

/// A captured element bounding rect.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    /// Left edge X (px).
    pub x: f32,
    /// Top edge Y (px).
    pub y: f32,
    /// Width (px).
    pub width: f32,
    /// Height (px).
    pub height: f32,
}

impl Rect {
    /// Create a new rect.
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Zero-sized rect at origin.
    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }

    /// Capture a DOM element's bounding rect.
    #[cfg(feature = "wasm-dom")]
    pub fn from_element(element: &web_sys::Element) -> Self {
        let r = element.get_bounding_client_rect();
        Self {
            x: r.x() as f32,
            y: r.y() as f32,
            width: r.width() as f32,
            height: r.height() as f32,
        }
    }

    /// Center point of the rect.
    pub fn center(&self) -> [f32; 2] {
        [self.x + self.width * 0.5, self.y + self.height * 0.5]
    }
}

// ── LayoutAnimation ─────────────────────────────────────────────────────────

/// A FLIP animation for a single element — translate + scale.
///
/// Created by [`LayoutAnimator::compute_transitions`].
#[derive(Debug)]
pub struct LayoutAnimation {
    /// Horizontal translation tween (px).
    pub translate_x: Tween<f32>,
    /// Vertical translation tween (px).
    pub translate_y: Tween<f32>,
    /// Horizontal scale tween.
    pub scale_x: Tween<f32>,
    /// Vertical scale tween.
    pub scale_y: Tween<f32>,
}

impl LayoutAnimation {
    /// Create a FLIP animation from two rects.
    fn from_rects(first: &Rect, last: &Rect, duration: f32, easing: Easing) -> Self {
        let dx = first.x - last.x;
        let dy = first.y - last.y;
        let sx = if last.width > 0.0 {
            first.width / last.width
        } else {
            1.0
        };
        let sy = if last.height > 0.0 {
            first.height / last.height
        } else {
            1.0
        };

        Self {
            translate_x: Tween::new(dx, 0.0)
                .duration(duration)
                .easing(easing.clone())
                .build(),
            translate_y: Tween::new(dy, 0.0)
                .duration(duration)
                .easing(easing.clone())
                .build(),
            scale_x: Tween::new(sx, 1.0)
                .duration(duration)
                .easing(easing.clone())
                .build(),
            scale_y: Tween::new(sy, 1.0)
                .duration(duration)
                .easing(easing)
                .build(),
        }
    }

    /// Whether all tweens have completed.
    pub fn is_complete(&self) -> bool {
        self.translate_x.is_complete()
            && self.translate_y.is_complete()
            && self.scale_x.is_complete()
            && self.scale_y.is_complete()
    }

    /// Get the current transform values as `(tx, ty, sx, sy)`.
    pub fn transform(&self) -> (f32, f32, f32, f32) {
        (
            self.translate_x.value(),
            self.translate_y.value(),
            self.scale_x.value(),
            self.scale_y.value(),
        )
    }

    /// Format the current transform as a CSS `transform` string.
    pub fn css_transform(&self) -> String {
        let (tx, ty, sx, sy) = self.transform();
        format!("translate({tx}px, {ty}px) scale({sx}, {sy})")
    }
}

impl Update for LayoutAnimation {
    fn update(&mut self, dt: f32) -> bool {
        let a = self.translate_x.update(dt);
        let b = self.translate_y.update(dt);
        let c = self.scale_x.update(dt);
        let d = self.scale_y.update(dt);
        a || b || c || d
    }
}

// ── LayoutTransition ────────────────────────────────────────────────────────

/// A layout transition result — contains the element ID and its animation.
#[derive(Debug)]
pub struct LayoutTransition {
    /// Element identifier.
    pub id: String,
    /// FLIP animation for this element.
    pub animation: LayoutAnimation,
}

// ── SharedElementTransition ─────────────────────────────────────────────────

/// Shared element transition between two views.
///
/// Animates an element from a source rect to a target rect, useful for
/// cross-view hero transitions.
#[derive(Debug)]
pub struct SharedElementTransition {
    /// Source rect (where the element was).
    pub source_rect: Rect,
    /// Target rect (where the element is going).
    pub target_rect: Rect,
    /// The underlying animation.
    pub animation: LayoutAnimation,
}

impl SharedElementTransition {
    /// Create a new shared element transition.
    pub fn new(source: Rect, target: Rect, duration: f32, easing: Easing) -> Self {
        let animation = LayoutAnimation::from_rects(&source, &target, duration, easing);
        Self {
            source_rect: source,
            target_rect: target,
            animation,
        }
    }

    /// Tick the transition.
    pub fn update(&mut self, dt: f32) {
        self.animation.update(dt);
    }

    /// Get the current CSS transform string.
    pub fn css_transform(&self) -> String {
        self.animation.css_transform()
    }

    /// Whether the transition is complete.
    pub fn is_complete(&self) -> bool {
        self.animation.is_complete()
    }
}

// ── LayoutAnimator ──────────────────────────────────────────────────────────

/// Tracks elements by string ID and creates FLIP layout animations.
///
/// # Workflow
///
/// 1. **Track** elements with their current rects
/// 2. **Mutate** the layout (reorder, add, remove)
/// 3. **Compute** transitions by providing the new rects
/// 4. **Tick** the animator each frame to advance animations
/// 5. **Read** CSS transforms and apply to elements
#[derive(Debug)]
pub struct LayoutAnimator {
    /// Currently tracked element rects.
    tracked: HashMap<String, Rect>,
    /// Active layout animations.
    animations: HashMap<String, LayoutAnimation>,
}

impl LayoutAnimator {
    /// Create a new empty layout animator.
    pub fn new() -> Self {
        Self {
            tracked: HashMap::new(),
            animations: HashMap::new(),
        }
    }

    /// Track an element by ID with its current rect.
    pub fn track(&mut self, id: &str, rect: Rect) {
        self.tracked.insert(id.into(), rect);
    }

    /// Track a DOM element by ID.
    #[cfg(feature = "wasm-dom")]
    pub fn track_element(&mut self, element: &web_sys::Element, id: &str) {
        self.track(id, Rect::from_element(element));
    }

    /// Remove an element from tracking.
    pub fn untrack(&mut self, id: &str) {
        self.tracked.remove(id);
        self.animations.remove(id);
    }

    /// Compute FLIP transitions for elements that moved.
    ///
    /// `new_rects` contains the post-mutation positions.  For each element
    /// that moved, a [`LayoutTransition`] is created and the stored rect
    /// is updated.
    pub fn compute_transitions(
        &mut self,
        new_rects: &[(&str, Rect)],
        duration: f32,
        easing: Easing,
    ) -> Vec<LayoutTransition> {
        let mut transitions = Vec::new();

        for &(id, new_rect) in new_rects {
            if let Some(old_rect) = self.tracked.get(id) {
                // Only animate if the rect actually changed
                let dx = (old_rect.x - new_rect.x).abs();
                let dy = (old_rect.y - new_rect.y).abs();
                let dw = (old_rect.width - new_rect.width).abs();
                let dh = (old_rect.height - new_rect.height).abs();

                if dx > 0.5 || dy > 0.5 || dw > 0.5 || dh > 0.5 {
                    let anim =
                        LayoutAnimation::from_rects(old_rect, &new_rect, duration, easing.clone());
                    self.animations.insert(
                        id.into(),
                        LayoutAnimation::from_rects(old_rect, &new_rect, duration, easing.clone()),
                    );
                    transitions.push(LayoutTransition {
                        id: id.into(),
                        animation: anim,
                    });
                }
            }

            // Update stored rect
            self.tracked.insert(id.into(), new_rect);
        }

        transitions
    }

    /// Convenience: animate all tracked elements to new positions.
    ///
    /// Calls `compute_transitions` internally.
    pub fn animate_to_new_positions(
        &mut self,
        new_rects: &[(&str, Rect)],
        duration: f32,
        easing: Easing,
    ) {
        let _ = self.compute_transitions(new_rects, duration, easing);
    }

    /// Batch FLIP for list reorder.
    ///
    /// Pass the old and new element positions.  Returns transitions for all
    /// elements that changed position.
    pub fn animate_reorder(
        &mut self,
        old_rects: &[(&str, Rect)],
        new_rects: &[(&str, Rect)],
        duration: f32,
        easing: Easing,
    ) -> Vec<LayoutTransition> {
        // Store old rects
        for &(id, rect) in old_rects {
            self.tracked.insert(id.into(), rect);
        }
        // Compute transitions to new rects
        self.compute_transitions(new_rects, duration, easing)
    }

    /// Animate an element entering the layout (fade/scale in).
    ///
    /// Creates an animation from a zero-sized rect at the target position
    /// to the target rect.
    pub fn animate_enter(
        &mut self,
        id: &str,
        target_rect: Rect,
        duration: f32,
        easing: Easing,
    ) -> LayoutAnimation {
        let from = Rect::new(
            target_rect.x + target_rect.width * 0.5,
            target_rect.y + target_rect.height * 0.5,
            0.0,
            0.0,
        );
        let anim = LayoutAnimation::from_rects(&from, &target_rect, duration, easing.clone());
        self.tracked.insert(id.into(), target_rect);
        self.animations.insert(
            id.into(),
            LayoutAnimation::from_rects(&from, &target_rect, duration, easing),
        );
        anim
    }

    /// Animate an element exiting the layout (fade/scale out).
    ///
    /// Returns `None` if the element is not tracked.
    pub fn animate_exit(
        &mut self,
        id: &str,
        duration: f32,
        easing: Easing,
    ) -> Option<LayoutAnimation> {
        let old_rect = self.tracked.remove(id)?;
        // Animate: translate to center, scale from 1 → 0
        let cx = old_rect.width * 0.5;
        let cy = old_rect.height * 0.5;
        let anim = LayoutAnimation {
            translate_x: Tween::new(0.0, cx)
                .duration(duration)
                .easing(easing.clone())
                .build(),
            translate_y: Tween::new(0.0, cy)
                .duration(duration)
                .easing(easing.clone())
                .build(),
            scale_x: Tween::new(1.0, 0.0)
                .duration(duration)
                .easing(easing.clone())
                .build(),
            scale_y: Tween::new(1.0, 0.0)
                .duration(duration)
                .easing(easing)
                .build(),
        };
        Some(anim)
    }

    /// Tick all active layout animations.
    pub fn update(&mut self, dt: f32) {
        self.animations.retain(|_id, anim| anim.update(dt));
    }

    /// Get the current CSS transform for an element (if animating).
    pub fn css_transform(&self, id: &str) -> Option<String> {
        self.animations.get(id).map(|a| a.css_transform())
    }

    /// Whether any elements are currently animating.
    pub fn is_animating(&self) -> bool {
        !self.animations.is_empty()
    }

    /// Number of tracked elements.
    pub fn tracked_count(&self) -> usize {
        self.tracked.len()
    }

    /// Number of active animations.
    pub fn animation_count(&self) -> usize {
        self.animations.len()
    }
}

impl Default for LayoutAnimator {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_center() {
        let r = Rect::new(10.0, 20.0, 100.0, 50.0);
        let c = r.center();
        assert!((c[0] - 60.0).abs() < 1e-6);
        assert!((c[1] - 45.0).abs() < 1e-6);
    }

    #[test]
    fn layout_animation_from_rects() {
        let first = Rect::new(0.0, 0.0, 100.0, 50.0);
        let last = Rect::new(50.0, 100.0, 200.0, 100.0);
        let anim = LayoutAnimation::from_rects(&first, &last, 0.3, Easing::Linear);

        let (tx, ty, sx, sy) = anim.transform();
        // At t=0: translate should be (0-50, 0-100) = (-50, -100)
        assert!((tx - (-50.0)).abs() < 1e-4, "tx={tx}");
        assert!((ty - (-100.0)).abs() < 1e-4, "ty={ty}");
        assert!((sx - 0.5).abs() < 1e-4, "sx={sx}");
        assert!((sy - 0.5).abs() < 1e-4, "sy={sy}");
    }

    #[test]
    fn layout_animation_completes() {
        let first = Rect::new(0.0, 0.0, 100.0, 50.0);
        let last = Rect::new(50.0, 100.0, 100.0, 50.0);
        let mut anim = LayoutAnimation::from_rects(&first, &last, 0.3, Easing::Linear);

        assert!(!anim.is_complete());
        anim.update(0.3);
        assert!(anim.is_complete());

        let (tx, ty, sx, sy) = anim.transform();
        assert!((tx).abs() < 1e-4);
        assert!((ty).abs() < 1e-4);
        assert!((sx - 1.0).abs() < 1e-4);
        assert!((sy - 1.0).abs() < 1e-4);
    }

    #[test]
    fn css_transform_format() {
        let first = Rect::new(10.0, 20.0, 100.0, 50.0);
        let last = Rect::new(10.0, 20.0, 100.0, 50.0);
        let mut anim = LayoutAnimation::from_rects(&first, &last, 0.1, Easing::Linear);
        anim.update(0.1);
        let css = anim.css_transform();
        assert!(css.contains("translate("));
        assert!(css.contains("scale("));
    }

    #[test]
    fn layout_animator_track_and_transition() {
        let mut la = LayoutAnimator::new();
        la.track("a", Rect::new(0.0, 0.0, 100.0, 50.0));
        la.track("b", Rect::new(0.0, 60.0, 100.0, 50.0));

        let transitions = la.compute_transitions(
            &[
                ("a", Rect::new(0.0, 60.0, 100.0, 50.0)),
                ("b", Rect::new(0.0, 0.0, 100.0, 50.0)),
            ],
            0.4,
            Easing::EaseOutCubic,
        );

        assert_eq!(transitions.len(), 2);
        assert!(la.is_animating());
    }

    #[test]
    fn layout_animator_no_change() {
        let mut la = LayoutAnimator::new();
        la.track("a", Rect::new(0.0, 0.0, 100.0, 50.0));

        let transitions = la.compute_transitions(
            &[("a", Rect::new(0.0, 0.0, 100.0, 50.0))],
            0.4,
            Easing::Linear,
        );

        assert_eq!(transitions.len(), 0);
    }

    #[test]
    fn layout_animator_update_completes() {
        let mut la = LayoutAnimator::new();
        la.track("a", Rect::new(0.0, 0.0, 100.0, 50.0));
        la.compute_transitions(
            &[("a", Rect::new(0.0, 100.0, 100.0, 50.0))],
            0.3,
            Easing::Linear,
        );

        assert!(la.is_animating());
        la.update(0.3);
        assert!(!la.is_animating());
    }

    #[test]
    fn layout_animator_reorder() {
        let mut la = LayoutAnimator::new();
        let old = &[
            ("a", Rect::new(0.0, 0.0, 100.0, 50.0)),
            ("b", Rect::new(0.0, 60.0, 100.0, 50.0)),
            ("c", Rect::new(0.0, 120.0, 100.0, 50.0)),
        ];
        let new = &[
            ("c", Rect::new(0.0, 0.0, 100.0, 50.0)),
            ("a", Rect::new(0.0, 60.0, 100.0, 50.0)),
            ("b", Rect::new(0.0, 120.0, 100.0, 50.0)),
        ];
        let transitions = la.animate_reorder(old, new, 0.3, Easing::EaseOutCubic);
        assert_eq!(transitions.len(), 3);
    }

    #[test]
    fn layout_animator_enter() {
        let mut la = LayoutAnimator::new();
        let anim = la.animate_enter(
            "new-el",
            Rect::new(50.0, 50.0, 100.0, 100.0),
            0.3,
            Easing::Linear,
        );
        // Scale starts from 0 — initial scale_x and scale_y at t=0
        let (_, _, sx, sy) = anim.transform();
        assert!((sx).abs() < 0.01, "sx={sx}");
        assert!((sy).abs() < 0.01, "sy={sy}");
    }

    #[test]
    fn layout_animator_exit() {
        let mut la = LayoutAnimator::new();
        la.track("bye", Rect::new(0.0, 0.0, 100.0, 50.0));
        let anim = la.animate_exit("bye", 0.3, Easing::Linear);
        assert!(anim.is_some());
        assert_eq!(la.tracked_count(), 0);
    }

    #[test]
    fn layout_animator_exit_unknown() {
        let mut la = LayoutAnimator::new();
        let anim = la.animate_exit("nonexistent", 0.3, Easing::Linear);
        assert!(anim.is_none());
    }

    #[test]
    fn shared_element_transition() {
        let source = Rect::new(10.0, 10.0, 50.0, 50.0);
        let target = Rect::new(100.0, 200.0, 200.0, 200.0);
        let mut set = SharedElementTransition::new(source, target, 0.5, Easing::EaseOutCubic);

        assert!(!set.is_complete());
        set.update(0.5);
        assert!(set.is_complete());

        let css = set.css_transform();
        assert!(css.contains("translate("));
    }
}
