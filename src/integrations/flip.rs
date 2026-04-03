//! FLIP animation technique — First, Last, Invert, Play.
//!
//! Capture element bounding rects before and after a layout change, then
//! animate the transform that brings the element from its old position/size
//! to its new one.
//!
//! Requires the `wasm-dom` feature for [`FlipState::capture`].
//! [`FlipState::diff`] is pure math.

use crate::easing::Easing;
use crate::traits::Update;
use crate::tween::Tween;

/// Captured element bounding rect.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlipState {
    /// Left edge X (px).
    pub x: f32,
    /// Top edge Y (px).
    pub y: f32,
    /// Width (px).
    pub width: f32,
    /// Height (px).
    pub height: f32,
}

impl FlipState {
    /// Create a FlipState from explicit values (works everywhere).
    pub fn from_rect(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Capture the current bounding rect of a DOM element.
    ///
    /// Calls `getBoundingClientRect()` on the element.
    pub fn capture(element: &web_sys::Element) -> Self {
        let rect = element.get_bounding_client_rect();
        Self {
            x: rect.x() as f32,
            y: rect.y() as f32,
            width: rect.width() as f32,
            height: rect.height() as f32,
        }
    }

    /// Compute the difference between two states and produce a FLIP animation.
    ///
    /// `first` is the state before the layout change, `last` is after.
    /// The resulting animation tweens the element's transform from the old
    /// position/size to the new one (identity transform).
    pub fn diff(first: &FlipState, last: &FlipState) -> FlipAnimationBuilder {
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

        FlipAnimationBuilder {
            dx,
            dy,
            sx,
            sy,
            duration: 0.3,
            easing: Easing::EaseOutCubic,
            #[cfg(all(feature = "std", not(feature = "bevy")))]
            on_enter: None,
            #[cfg(all(feature = "std", not(feature = "bevy")))]
            on_leave: None,
            #[cfg(all(feature = "std", not(feature = "bevy")))]
            on_complete: None,
        }
    }
}

/// Builder for [`FlipAnimation`].
pub struct FlipAnimationBuilder {
    dx: f32,
    dy: f32,
    sx: f32,
    sy: f32,
    duration: f32,
    easing: Easing,
    #[cfg(all(feature = "std", not(feature = "bevy")))]
    on_enter: Option<Box<dyn FnMut()>>,
    #[cfg(all(feature = "std", not(feature = "bevy")))]
    on_leave: Option<Box<dyn FnMut()>>,
    #[cfg(all(feature = "std", not(feature = "bevy")))]
    on_complete: Option<Box<dyn FnMut()>>,
}

impl core::fmt::Debug for FlipAnimationBuilder {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FlipAnimationBuilder")
            .field("dx", &self.dx)
            .field("dy", &self.dy)
            .field("sx", &self.sx)
            .field("sy", &self.sy)
            .field("duration", &self.duration)
            .field("easing", &self.easing)
            .finish()
    }
}

impl FlipAnimationBuilder {
    /// Set animation duration in seconds (default: 0.3).
    pub fn duration(mut self, d: f32) -> Self {
        self.duration = d;
        self
    }

    /// Set the easing curve (default: EaseOutCubic).
    pub fn easing(mut self, e: Easing) -> Self {
        self.easing = e;
        self
    }

    /// Set callback fired when animation starts (first update).
    ///
    /// GSAP equivalent: `onEnter` in Flip.from/to.
    #[cfg(all(feature = "std", not(feature = "bevy")))]
    pub fn on_enter<F: FnMut() + 'static>(mut self, f: F) -> Self {
        self.on_enter = Some(Box::new(f));
        self
    }

    /// Set callback fired when animation completes.
    ///
    /// GSAP equivalent: `onLeave` in Flip.from/to.
    #[cfg(all(feature = "std", not(feature = "bevy")))]
    pub fn on_leave<F: FnMut() + 'static>(mut self, f: F) -> Self {
        self.on_leave = Some(Box::new(f));
        self
    }

    /// Set callback fired on animation completion.
    ///
    /// GSAP equivalent: `onComplete` in FLIP animations.
    #[cfg(all(feature = "std", not(feature = "bevy")))]
    pub fn on_complete<F: FnMut() + 'static>(mut self, f: F) -> Self {
        self.on_complete = Some(Box::new(f));
        self
    }

    /// Build the FLIP animation.
    pub fn build(self) -> FlipAnimation {
        FlipAnimation {
            translate_x: Tween::new(self.dx, 0.0)
                .duration(self.duration)
                .easing(self.easing.clone())
                .build(),
            translate_y: Tween::new(self.dy, 0.0)
                .duration(self.duration)
                .easing(self.easing.clone())
                .build(),
            scale_x: Tween::new(self.sx, 1.0)
                .duration(self.duration)
                .easing(self.easing.clone())
                .build(),
            scale_y: Tween::new(self.sy, 1.0)
                .duration(self.duration)
                .easing(self.easing)
                .build(),
            #[cfg(all(feature = "std", not(feature = "bevy")))]
            on_enter_cb: self.on_enter,
            #[cfg(all(feature = "std", not(feature = "bevy")))]
            on_leave_cb: self.on_leave,
            #[cfg(all(feature = "std", not(feature = "bevy")))]
            on_complete_cb: self.on_complete,
            #[cfg(all(feature = "std", not(feature = "bevy")))]
            started: false,
        }
    }
}

/// FLIP animation — 4 tweens for translate X/Y and scale X/Y.
///
/// Read each tween's `.value()` and apply as a CSS transform:
/// ```text
/// transform: translate({tx}px, {ty}px) scale({sx}, {sy})
/// ```
pub struct FlipAnimation {
    /// Horizontal translation tween (px).
    pub translate_x: Tween<f32>,
    /// Vertical translation tween (px).
    pub translate_y: Tween<f32>,
    /// Horizontal scale tween.
    pub scale_x: Tween<f32>,
    /// Vertical scale tween.
    pub scale_y: Tween<f32>,
    /// Callback fired when animation starts.
    #[cfg(all(feature = "std", not(feature = "bevy")))]
    on_enter_cb: Option<Box<dyn FnMut()>>,
    /// Callback fired when animation exits viewport (completes).
    #[cfg(all(feature = "std", not(feature = "bevy")))]
    on_leave_cb: Option<Box<dyn FnMut()>>,
    /// Callback fired on completion.
    #[cfg(all(feature = "std", not(feature = "bevy")))]
    on_complete_cb: Option<Box<dyn FnMut()>>,
    /// Whether the animation has started (for on_enter).
    #[cfg(all(feature = "std", not(feature = "bevy")))]
    started: bool,
}

impl core::fmt::Debug for FlipAnimation {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FlipAnimation")
            .field("translate_x", &self.translate_x)
            .field("translate_y", &self.translate_y)
            .field("scale_x", &self.scale_x)
            .field("scale_y", &self.scale_y)
            .finish()
    }
}

impl FlipAnimation {
    /// Whether all 4 tweens have completed.
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

impl Update for FlipAnimation {
    fn update(&mut self, dt: f32) -> bool {
        // Fire on_enter on first update
        #[cfg(all(feature = "std", not(feature = "bevy")))]
        if !self.started {
            self.started = true;
            if let Some(ref mut cb) = self.on_enter_cb {
                cb();
            }
        }

        let _was_complete = self.is_complete();

        let a = self.translate_x.update(dt);
        let b = self.translate_y.update(dt);
        let c = self.scale_x.update(dt);
        let d = self.scale_y.update(dt);

        let running = a || b || c || d;

        // Fire callbacks on completion
        #[cfg(all(feature = "std", not(feature = "bevy")))]
        if !_was_complete && self.is_complete() {
            if let Some(ref mut cb) = self.on_leave_cb {
                cb();
            }
            if let Some(ref mut cb) = self.on_complete_cb {
                cb();
            }
        }

        running
    }
}
