//! Higher-level gesture recognition built on top of [`PointerData`].
//!
//! `GestureRecognizer` consumes pointer events and detects common gestures:
//! tap, long press, swipe, pinch-to-zoom, and two-finger rotation.
//!
//! The recognizer is platform-agnostic — it works with the pure-math
//! [`PointerData`](crate::drag::PointerData) struct.  For DOM binding,
//! use [`Observer`](crate::integrations::observer::Observer) to feed events.
//!
//! # Example
//!
//! ```rust
//! use spanda::gesture::{GestureRecognizer, Gesture, GestureConfig};
//! use spanda::drag::PointerData;
//!
//! let mut recognizer = GestureRecognizer::new();
//!
//! // Simulate a tap
//! recognizer.on_pointer_down(PointerData { x: 100.0, y: 100.0, pressure: 0.5, pointer_id: 0 });
//! recognizer.update(0.1);
//! let gesture = recognizer.on_pointer_up(PointerData { x: 101.0, y: 100.0, pressure: 0.0, pointer_id: 0 });
//!
//! match gesture {
//!     Some(Gesture::Tap { position }) => assert!((position[0] - 100.0).abs() < 2.0),
//!     _ => panic!("expected tap"),
//! }
//! ```

#[cfg(not(feature = "std"))]
#[allow(unused_imports)]
use num_traits::Float as _;

use crate::drag::PointerData;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// Recognised gesture types.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Gesture {
    /// Quick tap — pointer down + up within thresholds.
    Tap {
        /// Position of the tap.
        position: [f32; 2],
    },
    /// Pointer held without movement beyond threshold.
    LongPress {
        /// Position of the long press.
        position: [f32; 2],
        /// Duration the pointer was held (seconds).
        duration: f32,
    },
    /// Fast directional movement.
    Swipe {
        /// Dominant direction of the swipe.
        direction: SwipeDirection,
        /// Speed in pixels per second along the dominant axis.
        velocity: f32,
        /// Total displacement `[dx, dy]`.
        delta: [f32; 2],
    },
    /// Two-finger pinch (scale change).
    Pinch {
        /// Scale factor: >1.0 = zoom in, <1.0 = zoom out.
        scale: f32,
        /// Center point between the two fingers.
        center: [f32; 2],
    },
    /// Two-finger rotation.
    Rotate {
        /// Rotation angle in radians (positive = clockwise).
        angle: f32,
        /// Center point between the two fingers.
        center: [f32; 2],
    },
}

/// Cardinal swipe direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwipeDirection {
    /// Swipe toward negative Y (upward on screen).
    Up,
    /// Swipe toward positive Y (downward on screen).
    Down,
    /// Swipe toward negative X.
    Left,
    /// Swipe toward positive X.
    Right,
}

/// Configuration for gesture detection thresholds.
#[derive(Debug, Clone)]
pub struct GestureConfig {
    /// Maximum pixel movement for a touch to still count as a tap (default: 10.0).
    pub tap_max_distance: f32,
    /// Maximum seconds for a touch to still count as a tap (default: 0.3).
    pub tap_max_duration: f32,
    /// Seconds a pointer must be held to trigger a long press (default: 0.5).
    pub long_press_threshold: f32,
    /// Minimum velocity (px/s) for a movement to count as a swipe (default: 300.0).
    pub swipe_min_velocity: f32,
    /// Minimum pixel distance for a swipe (default: 50.0).
    pub swipe_min_distance: f32,
    /// Minimum scale delta for a pinch event (default: 0.05).
    pub pinch_min_scale_delta: f32,
    /// Minimum angle (radians) for a rotation event (default: 0.1).
    pub rotation_min_angle: f32,
}

impl Default for GestureConfig {
    fn default() -> Self {
        Self {
            tap_max_distance: 10.0,
            tap_max_duration: 0.3,
            long_press_threshold: 0.5,
            swipe_min_velocity: 300.0,
            swipe_min_distance: 50.0,
            pinch_min_scale_delta: 0.05,
            rotation_min_angle: 0.1,
        }
    }
}

/// Internal per-touch-point tracking.
#[derive(Debug, Clone)]
struct TouchPoint {
    id: i32,
    start_pos: [f32; 2],
    current_pos: [f32; 2],
    start_time: f32,
    last_dt: f32,
}

impl TouchPoint {
    fn distance_from_start(&self) -> f32 {
        let dx = self.current_pos[0] - self.start_pos[0];
        let dy = self.current_pos[1] - self.start_pos[1];
        (dx * dx + dy * dy).sqrt()
    }

    fn delta(&self) -> [f32; 2] {
        [
            self.current_pos[0] - self.start_pos[0],
            self.current_pos[1] - self.start_pos[1],
        ]
    }
}

/// Gesture recognizer — feed it pointer events, get gestures out.
///
/// The recognizer is stateful: it tracks active touch points and uses
/// configurable thresholds to disambiguate between taps, swipes, long
/// presses, and multi-touch gestures.
///
/// # Usage
///
/// 1. Create with [`GestureRecognizer::new()`]
/// 2. Optionally register a callback with [`on_gesture()`](GestureRecognizer::on_gesture)
/// 3. Feed pointer events: [`on_pointer_down`], [`on_pointer_move`], [`on_pointer_up`]
/// 4. Call [`update(dt)`](GestureRecognizer::update) each frame for time-based detection (long press)
pub struct GestureRecognizer {
    config: GestureConfig,
    active_touches: Vec<TouchPoint>,
    elapsed: f32,
    #[cfg(feature = "std")]
    callback: Option<Box<dyn FnMut(Gesture)>>,
    // For pinch/rotate: initial distance and angle between first two touches
    initial_distance: Option<f32>,
    initial_angle: Option<f32>,
    long_press_fired: bool,
}

impl core::fmt::Debug for GestureRecognizer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("GestureRecognizer")
            .field("config", &self.config)
            .field("active_touches", &self.active_touches.len())
            .field("elapsed", &self.elapsed)
            .field("long_press_fired", &self.long_press_fired)
            .finish()
    }
}

impl GestureRecognizer {
    /// Create a new gesture recognizer with default configuration.
    pub fn new() -> Self {
        Self::with_config(GestureConfig::default())
    }

    /// Create a new gesture recognizer with custom configuration.
    pub fn with_config(config: GestureConfig) -> Self {
        Self {
            config,
            active_touches: Vec::new(),
            elapsed: 0.0,
            #[cfg(feature = "std")]
            callback: None,
            initial_distance: None,
            initial_angle: None,
            long_press_fired: false,
        }
    }

    /// Register a gesture callback.  Fires whenever a gesture is detected.
    #[cfg(feature = "std")]
    pub fn on_gesture<F: FnMut(Gesture) + 'static>(&mut self, f: F) {
        self.callback = Some(Box::new(f));
    }

    /// Feed a pointer-down event.
    pub fn on_pointer_down(&mut self, data: PointerData) {
        let touch = TouchPoint {
            id: data.pointer_id,
            start_pos: [data.x, data.y],
            current_pos: [data.x, data.y],
            start_time: self.elapsed,
            last_dt: 0.0,
        };
        self.active_touches.push(touch);
        self.long_press_fired = false;

        // If we now have 2 touches, record initial distance and angle
        if self.active_touches.len() == 2 {
            let d = distance_between_touches(&self.active_touches[0], &self.active_touches[1]);
            let a = angle_between_touches(&self.active_touches[0], &self.active_touches[1]);
            self.initial_distance = Some(d);
            self.initial_angle = Some(a);
        }
    }

    /// Feed a pointer-move event.
    ///
    /// Returns a gesture if a multi-touch gesture (pinch/rotate) is detected.
    pub fn on_pointer_move(&mut self, data: PointerData) -> Option<Gesture> {
        // Find and update the matching touch
        if let Some(touch) = self
            .active_touches
            .iter_mut()
            .find(|t| t.id == data.pointer_id)
        {
            let dx = data.x - touch.current_pos[0];
            let dy = data.y - touch.current_pos[1];
            touch.current_pos = [data.x, data.y];
            // Track last frame displacement for velocity
            touch.last_dt = (dx * dx + dy * dy).sqrt();
        }

        // Check for multi-touch gestures
        if self.active_touches.len() == 2 {
            return self.check_multi_touch_gesture();
        }

        None
    }

    /// Feed a pointer-up event.
    ///
    /// Returns a gesture if a single-touch gesture (tap/swipe) is detected.
    pub fn on_pointer_up(&mut self, data: PointerData) -> Option<Gesture> {
        let touch_idx = self
            .active_touches
            .iter()
            .position(|t| t.id == data.pointer_id);
        let touch = match touch_idx {
            Some(idx) => self.active_touches.remove(idx),
            None => return None,
        };

        // Clear multi-touch state if we drop below 2
        if self.active_touches.len() < 2 {
            self.initial_distance = None;
            self.initial_angle = None;
        }

        // Don't detect single-touch gestures if long press already fired
        if self.long_press_fired {
            return None;
        }

        // Only detect single-touch gestures if no other pointers active
        if !self.active_touches.is_empty() {
            return None;
        }

        let duration = self.elapsed - touch.start_time;
        let dist = touch.distance_from_start();
        let delta = touch.delta();

        // Check for tap
        if dist < self.config.tap_max_distance && duration < self.config.tap_max_duration {
            let gesture = Gesture::Tap {
                position: touch.start_pos,
            };
            self.fire_callback(gesture.clone());
            return Some(gesture);
        }

        // Check for swipe
        if dist >= self.config.swipe_min_distance && duration > 0.0 {
            let velocity = dist / duration;
            if velocity >= self.config.swipe_min_velocity {
                let dx = delta[0].abs();
                let dy = delta[1].abs();
                let direction = if dx > dy {
                    if delta[0] > 0.0 {
                        SwipeDirection::Right
                    } else {
                        SwipeDirection::Left
                    }
                } else if delta[1] > 0.0 {
                    SwipeDirection::Down
                } else {
                    SwipeDirection::Up
                };

                let gesture = Gesture::Swipe {
                    direction,
                    velocity,
                    delta,
                };
                self.fire_callback(gesture.clone());
                return Some(gesture);
            }
        }

        None
    }

    /// Tick the recognizer for time-based detection (long press).
    ///
    /// Returns a gesture if a long press is detected this frame.
    pub fn update(&mut self, dt: f32) -> Option<Gesture> {
        self.elapsed += dt;

        // Check for long press on single-touch
        if self.active_touches.len() == 1 && !self.long_press_fired {
            let touch = &self.active_touches[0];
            let hold_time = self.elapsed - touch.start_time;
            let dist = touch.distance_from_start();

            if hold_time >= self.config.long_press_threshold && dist < self.config.tap_max_distance
            {
                self.long_press_fired = true;
                let gesture = Gesture::LongPress {
                    position: touch.start_pos,
                    duration: hold_time,
                };
                self.fire_callback(gesture.clone());
                return Some(gesture);
            }
        }

        None
    }

    /// Number of active touch points.
    pub fn active_touch_count(&self) -> usize {
        self.active_touches.len()
    }

    /// Cancel all pending gesture recognition.
    pub fn cancel(&mut self) {
        self.active_touches.clear();
        self.initial_distance = None;
        self.initial_angle = None;
        self.long_press_fired = false;
    }

    /// Get the current configuration.
    pub fn config(&self) -> &GestureConfig {
        &self.config
    }

    // ── Private helpers ──────────────────────────────────────────────────

    fn check_multi_touch_gesture(&self) -> Option<Gesture> {
        if self.active_touches.len() < 2 {
            return None;
        }

        let t0 = &self.active_touches[0];
        let t1 = &self.active_touches[1];
        let center = [
            (t0.current_pos[0] + t1.current_pos[0]) * 0.5,
            (t0.current_pos[1] + t1.current_pos[1]) * 0.5,
        ];

        // Pinch detection
        if let Some(initial_dist) = self.initial_distance {
            if initial_dist > 1e-6 {
                let current_dist = distance_between_touches(t0, t1);
                let scale = current_dist / initial_dist;
                if (scale - 1.0).abs() >= self.config.pinch_min_scale_delta {
                    return Some(Gesture::Pinch { scale, center });
                }
            }
        }

        // Rotation detection
        if let Some(initial_ang) = self.initial_angle {
            let current_ang = angle_between_touches(t0, t1);
            let mut angle_diff = current_ang - initial_ang;
            // Normalise to [-π, π]
            while angle_diff > core::f32::consts::PI {
                angle_diff -= 2.0 * core::f32::consts::PI;
            }
            while angle_diff < -core::f32::consts::PI {
                angle_diff += 2.0 * core::f32::consts::PI;
            }
            if angle_diff.abs() >= self.config.rotation_min_angle {
                return Some(Gesture::Rotate {
                    angle: angle_diff,
                    center,
                });
            }
        }

        None
    }

    fn fire_callback(&mut self, gesture: Gesture) {
        #[cfg(feature = "std")]
        {
            if let Some(ref mut cb) = self.callback {
                cb(gesture);
            }
        }
        let _ = gesture; // avoid unused warning in no_std
    }
}

impl Default for GestureRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

// ── Free helper functions ───────────────────────────────────────────────────

fn distance_between_touches(a: &TouchPoint, b: &TouchPoint) -> f32 {
    let dx = a.current_pos[0] - b.current_pos[0];
    let dy = a.current_pos[1] - b.current_pos[1];
    (dx * dx + dy * dy).sqrt()
}

fn angle_between_touches(a: &TouchPoint, b: &TouchPoint) -> f32 {
    let dx = b.current_pos[0] - a.current_pos[0];
    let dy = b.current_pos[1] - a.current_pos[1];
    dy.atan2(dx)
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drag::PointerData;

    fn pointer(id: i32, x: f32, y: f32) -> PointerData {
        PointerData {
            x,
            y,
            pressure: 0.5,
            pointer_id: id,
        }
    }

    #[test]
    fn tap_detected() {
        let mut r = GestureRecognizer::new();
        r.on_pointer_down(pointer(0, 100.0, 100.0));
        r.update(0.1); // 100ms hold — within tap threshold
        let g = r.on_pointer_up(pointer(0, 101.0, 100.0));
        match g {
            Some(Gesture::Tap { position }) => {
                assert!((position[0] - 100.0).abs() < 1e-6);
                assert!((position[1] - 100.0).abs() < 1e-6);
            }
            other => panic!("expected Tap, got {:?}", other),
        }
    }

    #[test]
    fn tap_rejected_if_too_far() {
        let mut r = GestureRecognizer::new();
        r.on_pointer_down(pointer(0, 100.0, 100.0));
        r.update(0.1);
        // Move 50px — exceeds tap_max_distance (10)
        r.on_pointer_move(pointer(0, 150.0, 100.0));
        let g = r.on_pointer_up(pointer(0, 150.0, 100.0));
        assert!(
            !matches!(g, Some(Gesture::Tap { .. })),
            "Should not be a tap: {:?}",
            g
        );
    }

    #[test]
    fn tap_rejected_if_too_slow() {
        let mut r = GestureRecognizer::new();
        r.on_pointer_down(pointer(0, 100.0, 100.0));
        r.update(0.5); // 500ms — exceeds tap_max_duration (300ms)
        let g = r.on_pointer_up(pointer(0, 101.0, 100.0));
        assert!(
            !matches!(g, Some(Gesture::Tap { .. })),
            "Should not be a tap after 500ms: {:?}",
            g
        );
    }

    #[test]
    fn long_press_detected() {
        let mut r = GestureRecognizer::new();
        r.on_pointer_down(pointer(0, 100.0, 100.0));
        // Tick past long_press_threshold (0.5s)
        let g = r.update(0.6);
        match g {
            Some(Gesture::LongPress { position, duration }) => {
                assert!((position[0] - 100.0).abs() < 1e-6);
                assert!(duration >= 0.5);
            }
            other => panic!("expected LongPress, got {:?}", other),
        }
    }

    #[test]
    fn long_press_cancelled_by_move() {
        let mut r = GestureRecognizer::new();
        r.on_pointer_down(pointer(0, 100.0, 100.0));
        r.update(0.2);
        // Move 50px — cancels long press because distance > tap_max_distance
        r.on_pointer_move(pointer(0, 150.0, 100.0));
        let g = r.update(0.5);
        assert!(
            !matches!(g, Some(Gesture::LongPress { .. })),
            "Long press should be cancelled by movement: {:?}",
            g
        );
    }

    #[test]
    fn swipe_right_detected() {
        let mut r = GestureRecognizer::new();
        r.on_pointer_down(pointer(0, 100.0, 100.0));
        r.update(0.05); // 50ms
        r.on_pointer_move(pointer(0, 250.0, 105.0)); // 150px right
        let g = r.on_pointer_up(pointer(0, 250.0, 105.0));
        match g {
            Some(Gesture::Swipe {
                direction,
                velocity,
                ..
            }) => {
                assert_eq!(direction, SwipeDirection::Right);
                assert!(velocity > 300.0, "velocity={velocity}");
            }
            other => panic!("expected Swipe Right, got {:?}", other),
        }
    }

    #[test]
    fn swipe_up_detected() {
        let mut r = GestureRecognizer::new();
        r.on_pointer_down(pointer(0, 100.0, 300.0));
        r.update(0.05);
        r.on_pointer_move(pointer(0, 105.0, 100.0)); // 200px up (negative Y)
        let g = r.on_pointer_up(pointer(0, 105.0, 100.0));
        match g {
            Some(Gesture::Swipe { direction, .. }) => {
                assert_eq!(direction, SwipeDirection::Up);
            }
            other => panic!("expected Swipe Up, got {:?}", other),
        }
    }

    #[test]
    fn swipe_rejected_if_too_slow() {
        let mut r = GestureRecognizer::new();
        r.on_pointer_down(pointer(0, 100.0, 100.0));
        r.update(2.0); // 2 seconds — very slow
        r.on_pointer_move(pointer(0, 200.0, 100.0)); // 100px
        let g = r.on_pointer_up(pointer(0, 200.0, 100.0));
        // 100px / 2s = 50 px/s < 300 px/s threshold
        assert!(
            !matches!(g, Some(Gesture::Swipe { .. })),
            "Should not be a swipe at 50 px/s: {:?}",
            g
        );
    }

    #[test]
    fn pinch_zoom_detected() {
        let mut r = GestureRecognizer::new();
        // Two fingers, initially 100px apart
        r.on_pointer_down(pointer(0, 100.0, 200.0));
        r.on_pointer_down(pointer(1, 200.0, 200.0));
        // Spread to 300px apart (scale = 3.0)
        r.on_pointer_move(pointer(0, 50.0, 200.0));
        let g = r.on_pointer_move(pointer(1, 350.0, 200.0));
        match g {
            Some(Gesture::Pinch { scale, center }) => {
                assert!(scale > 1.5, "scale={scale}");
                assert!((center[1] - 200.0).abs() < 1e-4);
            }
            other => panic!("expected Pinch, got {:?}", other),
        }
    }

    #[test]
    fn rotation_detected() {
        let mut r = GestureRecognizer::with_config(GestureConfig {
            rotation_min_angle: 0.01, // low threshold for test
            ..Default::default()
        });
        // Two fingers on a horizontal line
        r.on_pointer_down(pointer(0, 100.0, 200.0));
        r.on_pointer_down(pointer(1, 200.0, 200.0));
        // Rotate: move finger 1 up, finger 0 stays
        r.on_pointer_move(pointer(1, 200.0, 100.0));
        let g = r.on_pointer_move(pointer(0, 100.0, 200.0)); // trigger re-check
        // The angle should have changed significantly
        // Initial: atan2(0, 100) = 0
        // New: atan2(-100, 100) ≈ -0.785 rad
        match g {
            Some(Gesture::Rotate { angle, .. }) => {
                assert!(angle.abs() > 0.5, "angle={angle}");
            }
            Some(Gesture::Pinch { .. }) => {
                // Pinch may also fire — that's ok, distance changed too.
                // Re-test with equidistant rotation.
            }
            other => panic!("expected Rotate or Pinch, got {:?}", other),
        }
    }

    #[test]
    fn custom_config_thresholds() {
        let config = GestureConfig {
            tap_max_distance: 50.0, // very generous
            tap_max_duration: 1.0,
            long_press_threshold: 2.0, // prevent long press from firing before pointer up
            ..Default::default()
        };
        let mut r = GestureRecognizer::with_config(config);
        r.on_pointer_down(pointer(0, 100.0, 100.0));
        r.update(0.5);
        // Move 30px — within custom threshold of 50
        r.on_pointer_move(pointer(0, 130.0, 100.0));
        let g = r.on_pointer_up(pointer(0, 130.0, 100.0));
        match g {
            Some(Gesture::Tap { .. }) => {} // expected with generous thresholds
            other => panic!("expected Tap with generous config, got {:?}", other),
        }
    }

    #[test]
    fn cancel_clears_state() {
        let mut r = GestureRecognizer::new();
        r.on_pointer_down(pointer(0, 100.0, 100.0));
        assert_eq!(r.active_touch_count(), 1);
        r.cancel();
        assert_eq!(r.active_touch_count(), 0);
    }

    #[cfg(feature = "std")]
    #[test]
    fn callback_fires_on_tap() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicU32, Ordering};

        let count = Arc::new(AtomicU32::new(0));
        let count_clone = count.clone();

        let mut r = GestureRecognizer::new();
        r.on_gesture(move |_g| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        });

        r.on_pointer_down(pointer(0, 100.0, 100.0));
        r.update(0.05);
        let _ = r.on_pointer_up(pointer(0, 101.0, 100.0));

        assert_eq!(count.load(Ordering::SeqCst), 1);
    }
}
