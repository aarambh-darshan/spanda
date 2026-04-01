//! Pure-math drag state tracker and pointer data.
//!
//! `DragState` tracks pointer position, computes velocity, and applies
//! constraints (bounds, axis lock, grid snap). It works everywhere — no DOM
//! dependency. Call [`DragState::on_pointer_up`] to get an [`InertiaN`] for
//! momentum-based throw after release.
//!
//! For DOM binding see [`crate::integrations::draggable::Draggable`] (wasm-dom feature).
//!
//! # Example
//!
//! ```rust
//! use spanda::drag::{DragState, DragConstraints};
//!
//! let mut drag = DragState::new();
//! drag.on_pointer_down(10.0, 20.0);
//! drag.on_pointer_move(30.0, 25.0, 1.0 / 60.0);
//! drag.on_pointer_move(50.0, 30.0, 1.0 / 60.0);
//! assert!(drag.is_dragging());
//!
//! let inertia = drag.on_pointer_up();
//! // inertia carries momentum from the drag
//! ```

use crate::inertia::{InertiaConfig, InertiaN};

/// Unified pointer data — normalises mouse, touch, and pointer events.
#[derive(Debug, Clone, Default)]
pub struct PointerData {
    /// X position (client coordinates).
    pub x: f32,
    /// Y position (client coordinates).
    pub y: f32,
    /// Pointer pressure (0.0–1.0); 0.5 for mouse events.
    pub pressure: f32,
    /// Pointer ID (0 for mouse, touch identifier for touch events).
    pub pointer_id: i32,
}

/// Constraint options for dragging.
#[derive(Debug, Clone, Default)]
pub struct DragConstraints {
    /// Bounding rect: `[min_x, min_y, max_x, max_y]`. `None` = unconstrained.
    pub bounds: Option<[f32; 4]>,
    /// Lock movement to a single axis.
    pub axis_lock: Option<DragAxis>,
    /// Snap position to a grid during drag. `[grid_x, grid_y]`.
    pub snap_to_grid: Option<[f32; 2]>,
    /// Snap position to grid on release (different from live snap).
    pub snap_on_release: Option<[f32; 2]>,
}

/// Axis constraint for dragging.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragAxis {
    /// Allow only horizontal movement.
    X,
    /// Allow only vertical movement.
    Y,
}

/// Pure-math drag state tracker.
///
/// Tracks pointer position, computes velocity via exponential moving average,
/// and applies constraints. No DOM dependency.
pub struct DragState {
    position: [f32; 2],
    velocity: [f32; 2],
    dragging: bool,
    start_pointer: [f32; 2],
    start_position: [f32; 2],
    last_pointer: [f32; 2],
    constraints: DragConstraints,
    /// Callback fired when drag starts.
    #[cfg(feature = "std")]
    on_drag_start_cb: Option<Box<dyn FnMut([f32; 2])>>,
    /// Callback fired when drag ends.
    #[cfg(feature = "std")]
    on_drag_end_cb: Option<Box<dyn FnMut([f32; 2], [f32; 2])>>,
    /// Callback fired on click (tap without significant movement).
    #[cfg(feature = "std")]
    on_click_cb: Option<Box<dyn FnMut([f32; 2])>>,
    /// Callback fired each frame during inertia throw (if using manual update).
    #[cfg(feature = "std")]
    on_throw_update_cb: Option<Box<dyn FnMut([f32; 2], [f32; 2])>>,
    /// Click detection threshold (max distance moved to count as click).
    click_threshold: f32,
}

impl core::fmt::Debug for DragState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DragState")
            .field("position", &self.position)
            .field("velocity", &self.velocity)
            .field("dragging", &self.dragging)
            .field("start_pointer", &self.start_pointer)
            .field("start_position", &self.start_position)
            .field("last_pointer", &self.last_pointer)
            .field("constraints", &self.constraints)
            .field("click_threshold", &self.click_threshold)
            .finish()
    }
}

impl DragState {
    /// Create a new drag state with no constraints, positioned at origin.
    pub fn new() -> Self {
        Self {
            position: [0.0, 0.0],
            velocity: [0.0, 0.0],
            dragging: false,
            start_pointer: [0.0, 0.0],
            start_position: [0.0, 0.0],
            last_pointer: [0.0, 0.0],
            constraints: DragConstraints::default(),
            #[cfg(feature = "std")]
            on_drag_start_cb: None,
            #[cfg(feature = "std")]
            on_drag_end_cb: None,
            #[cfg(feature = "std")]
            on_click_cb: None,
            #[cfg(feature = "std")]
            on_throw_update_cb: None,
            click_threshold: 5.0,
        }
    }

    /// Set constraints (builder-style).
    pub fn with_constraints(mut self, constraints: DragConstraints) -> Self {
        self.constraints = constraints;
        self
    }

    /// Set initial position (builder-style).
    pub fn with_position(mut self, position: [f32; 2]) -> Self {
        self.position = position;
        self
    }

    /// Set click detection threshold (builder-style).
    ///
    /// If pointer moves less than this distance, it's considered a click.
    pub fn with_click_threshold(mut self, threshold: f32) -> Self {
        self.click_threshold = threshold;
        self
    }

    /// Register a callback fired when drag starts.
    #[cfg(feature = "std")]
    pub fn on_drag_start<F: FnMut([f32; 2]) + 'static>(&mut self, f: F) -> &mut Self {
        self.on_drag_start_cb = Some(Box::new(f));
        self
    }

    /// Register a callback fired when drag ends.
    ///
    /// Callback receives `(final_position, velocity)`.
    #[cfg(feature = "std")]
    pub fn on_drag_end<F: FnMut([f32; 2], [f32; 2]) + 'static>(&mut self, f: F) -> &mut Self {
        self.on_drag_end_cb = Some(Box::new(f));
        self
    }

    /// Register a callback fired on click (tap without significant movement).
    ///
    /// Callback receives the click position.
    #[cfg(feature = "std")]
    pub fn on_click<F: FnMut([f32; 2]) + 'static>(&mut self, f: F) -> &mut Self {
        self.on_click_cb = Some(Box::new(f));
        self
    }

    /// Register a callback fired during inertia throw updates.
    ///
    /// Callback receives `(position, velocity)` each frame.
    #[cfg(feature = "std")]
    pub fn on_throw_update<F: FnMut([f32; 2], [f32; 2]) + 'static>(&mut self, f: F) -> &mut Self {
        self.on_throw_update_cb = Some(Box::new(f));
        self
    }

    /// Called when a pointer/mouse/touch press begins.
    pub fn on_pointer_down(&mut self, x: f32, y: f32) {
        self.dragging = true;
        self.start_pointer = [x, y];
        self.start_position = self.position;
        self.last_pointer = [x, y];
        self.velocity = [0.0, 0.0];

        // Fire on_drag_start callback
        #[cfg(feature = "std")]
        {
            if let Some(ref mut cb) = self.on_drag_start_cb {
                cb(self.position);
            }
        }
    }

    /// Called each frame while the pointer is held down.
    ///
    /// `dt` is the time since the last move event (for velocity calculation).
    pub fn on_pointer_move(&mut self, x: f32, y: f32, dt: f32) {
        if !self.dragging {
            return;
        }

        let dx = x - self.start_pointer[0];
        let dy = y - self.start_pointer[1];

        let mut new_pos = [
            self.start_position[0] + dx,
            self.start_position[1] + dy,
        ];

        new_pos = self.apply_constraints(new_pos);

        // Compute instantaneous velocity and smooth with EMA
        if dt > 1e-6 {
            let inst_vx = (x - self.last_pointer[0]) / dt;
            let inst_vy = (y - self.last_pointer[1]) / dt;
            self.velocity[0] = 0.8 * inst_vx + 0.2 * self.velocity[0];
            self.velocity[1] = 0.8 * inst_vy + 0.2 * self.velocity[1];
        }

        self.position = new_pos;
        self.last_pointer = [x, y];
    }

    /// Called when the pointer is released. Returns an [`InertiaN`] carrying
    /// the momentum from the drag.
    pub fn on_pointer_up(&mut self) -> InertiaN<[f32; 2]> {
        self.dragging = false;

        // Calculate distance moved for click detection
        let dx = self.last_pointer[0] - self.start_pointer[0];
        let dy = self.last_pointer[1] - self.start_pointer[1];
        let distance = (dx * dx + dy * dy).sqrt();

        // Apply snap on release if configured
        if let Some(grid) = &self.constraints.snap_on_release {
            if grid[0] > 0.0 {
                self.position[0] = (self.position[0] / grid[0]).round() * grid[0];
            }
            if grid[1] > 0.0 {
                self.position[1] = (self.position[1] / grid[1]).round() * grid[1];
            }
        }

        // Fire callbacks
        #[cfg(feature = "std")]
        {
            // Check if this was a click (minimal movement)
            if distance < self.click_threshold {
                if let Some(ref mut cb) = self.on_click_cb {
                    cb(self.position);
                }
            }

            // Always fire on_drag_end
            if let Some(ref mut cb) = self.on_drag_end_cb {
                cb(self.position, self.velocity);
            }
        }

        InertiaN::new(InertiaConfig::default_flick(), self.position)
            .with_velocity(self.velocity)
    }

    /// Current drag position.
    pub fn position(&self) -> [f32; 2] {
        self.position
    }

    /// Current velocity (smoothed).
    pub fn velocity(&self) -> [f32; 2] {
        self.velocity
    }

    /// Whether the pointer is currently held down.
    pub fn is_dragging(&self) -> bool {
        self.dragging
    }

    fn apply_constraints(&self, mut pos: [f32; 2]) -> [f32; 2] {
        // Axis lock
        if let Some(axis) = &self.constraints.axis_lock {
            match axis {
                DragAxis::X => pos[1] = self.start_position[1],
                DragAxis::Y => pos[0] = self.start_position[0],
            }
        }

        // Bounds clamping
        if let Some(bounds) = &self.constraints.bounds {
            pos[0] = pos[0].clamp(bounds[0], bounds[2]);
            pos[1] = pos[1].clamp(bounds[1], bounds[3]);
        }

        // Grid snapping
        if let Some(grid) = &self.constraints.snap_to_grid {
            if grid[0] > 0.0 {
                pos[0] = (pos[0] / grid[0]).round() * grid[0];
            }
            if grid[1] > 0.0 {
                pos[1] = (pos[1] / grid[1]).round() * grid[1];
            }
        }

        pos
    }
}

impl Default for DragState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::Update;

    #[test]
    fn drag_basic_movement() {
        let mut drag = DragState::new().with_position([100.0, 100.0]);
        drag.on_pointer_down(50.0, 50.0);
        drag.on_pointer_move(70.0, 60.0, 1.0 / 60.0);
        assert_eq!(drag.position(), [120.0, 110.0]);
    }

    #[test]
    fn drag_axis_lock_x() {
        let mut drag = DragState::new()
            .with_constraints(DragConstraints {
                axis_lock: Some(DragAxis::X),
                ..Default::default()
            });
        drag.on_pointer_down(0.0, 0.0);
        drag.on_pointer_move(50.0, 30.0, 1.0 / 60.0);
        let pos = drag.position();
        assert!((pos[0] - 50.0).abs() < 1e-6);
        assert!((pos[1]).abs() < 1e-6, "Y should be locked: {}", pos[1]);
    }

    #[test]
    fn drag_axis_lock_y() {
        let mut drag = DragState::new()
            .with_constraints(DragConstraints {
                axis_lock: Some(DragAxis::Y),
                ..Default::default()
            });
        drag.on_pointer_down(0.0, 0.0);
        drag.on_pointer_move(50.0, 30.0, 1.0 / 60.0);
        let pos = drag.position();
        assert!((pos[0]).abs() < 1e-6, "X should be locked: {}", pos[0]);
        assert!((pos[1] - 30.0).abs() < 1e-6);
    }

    #[test]
    fn drag_bounds_clamping() {
        let mut drag = DragState::new()
            .with_constraints(DragConstraints {
                bounds: Some([0.0, 0.0, 100.0, 100.0]),
                ..Default::default()
            });
        drag.on_pointer_down(50.0, 50.0);
        drag.on_pointer_move(200.0, 200.0, 1.0 / 60.0);
        let pos = drag.position();
        assert!(pos[0] <= 100.0, "X should be clamped: {}", pos[0]);
        assert!(pos[1] <= 100.0, "Y should be clamped: {}", pos[1]);
    }

    #[test]
    fn drag_grid_snapping() {
        let mut drag = DragState::new()
            .with_constraints(DragConstraints {
                snap_to_grid: Some([10.0, 10.0]),
                ..Default::default()
            });
        drag.on_pointer_down(0.0, 0.0);
        drag.on_pointer_move(17.0, 23.0, 1.0 / 60.0);
        let pos = drag.position();
        assert!((pos[0] - 20.0).abs() < 1e-6, "X should snap to 20: {}", pos[0]);
        assert!((pos[1] - 20.0).abs() < 1e-6, "Y should snap to 20: {}", pos[1]);
    }

    #[test]
    fn drag_velocity_tracking() {
        let mut drag = DragState::new();
        drag.on_pointer_down(0.0, 0.0);
        // Move 100px in 1/60s = 6000 px/s
        drag.on_pointer_move(100.0, 0.0, 1.0 / 60.0);
        let vel = drag.velocity();
        assert!(vel[0] > 1000.0, "Expected large X velocity: {}", vel[0]);
    }

    #[test]
    fn drag_pointer_up_returns_inertia() {
        let mut drag = DragState::new();
        drag.on_pointer_down(0.0, 0.0);
        drag.on_pointer_move(50.0, 0.0, 1.0 / 60.0);
        let mut inertia = drag.on_pointer_up();
        assert!(!drag.is_dragging());

        // The inertia should carry momentum
        let pos_before = inertia.position();
        inertia.update(1.0 / 60.0);
        let pos_after = inertia.position();
        assert!(pos_after[0] > pos_before[0], "Inertia should continue moving");
    }

    #[test]
    fn drag_not_dragging_ignores_moves() {
        let mut drag = DragState::new();
        drag.on_pointer_move(100.0, 100.0, 1.0 / 60.0);
        assert_eq!(drag.position(), [0.0, 0.0]);
    }
}
