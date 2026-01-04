//! Configuration options for the infinite canvas.
//!
//! This module provides the `CanvasOptions` struct which controls
//! various aspects of canvas behavior including zoom limits, pan/zoom
//! speeds, grid display, and input handling.

use gpui::{px, Pixels};
use serde::{Deserialize, Serialize};

/// Configuration options for an infinite canvas.
///
/// These options control the behavior of pan, zoom, grid display,
/// and other canvas features.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CanvasOptions {
    /// Minimum allowed zoom level (e.g., 0.1 = 10%).
    pub min_zoom: f32,

    /// Maximum allowed zoom level (e.g., 8.0 = 800%).
    pub max_zoom: f32,

    /// Discrete zoom steps for step-based zooming (zoom in/out actions).
    /// Should be sorted in ascending order.
    pub zoom_steps: Vec<f32>,

    /// Multiplier for pan speed (1.0 = normal).
    pub pan_speed: f32,

    /// Multiplier for zoom speed when using scroll wheel (1.0 = normal).
    pub zoom_speed: f32,

    /// Whether to show the background grid.
    pub show_grid: bool,

    /// Grid cell size in canvas units.
    pub grid_size: Pixels,

    /// Whether the camera is locked (prevents pan/zoom).
    pub locked: bool,

    /// Behavior when using the scroll wheel.
    pub wheel_behavior: WheelBehavior,

    /// Whether to enable inertial panning (momentum after releasing).
    pub inertia_enabled: bool,

    /// Friction coefficient for inertial panning (0.0-1.0, higher = more friction).
    pub inertia_friction: f32,
}

impl Default for CanvasOptions {
    fn default() -> Self {
        Self {
            min_zoom: 0.1,
            max_zoom: 8.0,
            zoom_steps: vec![0.1, 0.25, 0.5, 0.75, 1.0, 1.5, 2.0, 3.0, 4.0, 6.0, 8.0],
            pan_speed: 1.0,
            zoom_speed: 1.0,
            show_grid: true,
            grid_size: px(20.0),
            locked: false,
            wheel_behavior: WheelBehavior::default(),
            inertia_enabled: false,
            inertia_friction: 0.92,
        }
    }
}

impl CanvasOptions {
    /// Create new canvas options with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the minimum zoom level.
    pub fn min_zoom(mut self, min_zoom: f32) -> Self {
        self.min_zoom = min_zoom;
        self
    }

    /// Set the maximum zoom level.
    pub fn max_zoom(mut self, max_zoom: f32) -> Self {
        self.max_zoom = max_zoom;
        self
    }

    /// Set both min and max zoom levels.
    pub fn zoom_range(mut self, min: f32, max: f32) -> Self {
        self.min_zoom = min;
        self.max_zoom = max;
        self
    }

    /// Set the discrete zoom steps.
    pub fn zoom_steps(mut self, steps: Vec<f32>) -> Self {
        self.zoom_steps = steps;
        self
    }

    /// Set the pan speed multiplier.
    pub fn pan_speed(mut self, speed: f32) -> Self {
        self.pan_speed = speed;
        self
    }

    /// Set the zoom speed multiplier.
    pub fn zoom_speed(mut self, speed: f32) -> Self {
        self.zoom_speed = speed;
        self
    }

    /// Enable or disable the grid.
    pub fn show_grid(mut self, show: bool) -> Self {
        self.show_grid = show;
        self
    }

    /// Set the grid cell size.
    pub fn grid_size(mut self, size: Pixels) -> Self {
        self.grid_size = size;
        self
    }

    /// Lock or unlock the camera.
    pub fn locked(mut self, locked: bool) -> Self {
        self.locked = locked;
        self
    }

    /// Set the scroll wheel behavior.
    pub fn wheel_behavior(mut self, behavior: WheelBehavior) -> Self {
        self.wheel_behavior = behavior;
        self
    }

    /// Enable or disable inertial panning.
    pub fn inertia_enabled(mut self, enabled: bool) -> Self {
        self.inertia_enabled = enabled;
        self
    }

    /// Set the inertia friction coefficient.
    pub fn inertia_friction(mut self, friction: f32) -> Self {
        self.inertia_friction = friction.clamp(0.0, 1.0);
        self
    }
}

/// Behavior when using the scroll wheel.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WheelBehavior {
    /// Scroll wheel zooms in/out (default).
    #[default]
    Zoom,

    /// Scroll wheel pans the canvas.
    Pan,

    /// Scroll wheel does nothing.
    None,
}

impl WheelBehavior {
    /// Returns true if the wheel should zoom.
    pub fn is_zoom(&self) -> bool {
        matches!(self, Self::Zoom)
    }

    /// Returns true if the wheel should pan.
    pub fn is_pan(&self) -> bool {
        matches!(self, Self::Pan)
    }

    /// Returns true if the wheel is disabled.
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

/// Constraints on camera movement.
///
/// These can be used to limit the camera to specific bounds
/// or behaviors.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CameraConstraints {
    /// Optional bounds that constrain camera movement.
    pub bounds: Option<ConstraintBounds>,

    /// How the constraints are applied.
    pub behavior: ConstraintBehavior,
}

impl CameraConstraints {
    /// Create unconstrained camera constraints.
    pub fn none() -> Self {
        Self::default()
    }

    /// Create constraints with specific bounds.
    pub fn with_bounds(bounds: ConstraintBounds, behavior: ConstraintBehavior) -> Self {
        Self {
            bounds: Some(bounds),
            behavior,
        }
    }
}

/// Bounds for constraining camera movement.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ConstraintBounds {
    /// Minimum X position in canvas space.
    pub min_x: f32,
    /// Maximum X position in canvas space.
    pub max_x: f32,
    /// Minimum Y position in canvas space.
    pub min_y: f32,
    /// Maximum Y position in canvas space.
    pub max_y: f32,
}

impl ConstraintBounds {
    /// Create new constraint bounds.
    pub fn new(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        Self {
            min_x,
            max_x,
            min_y,
            max_y,
        }
    }

    /// Create constraint bounds from origin and size.
    pub fn from_origin_size(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            min_x: x,
            max_x: x + width,
            min_y: y,
            max_y: y + height,
        }
    }
}

/// How camera constraints are applied.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintBehavior {
    /// Camera can move freely (no constraints).
    #[default]
    Free,

    /// Camera viewport must stay completely inside the bounds.
    Inside,

    /// Camera viewport must stay completely outside the bounds.
    Outside,

    /// Camera is fixed and cannot move.
    Fixed,

    /// Camera adjusts zoom to contain the bounds.
    Contain,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_options() {
        let options = CanvasOptions::default();
        assert_eq!(options.min_zoom, 0.1);
        assert_eq!(options.max_zoom, 8.0);
        assert!(options.show_grid);
        assert!(!options.locked);
    }

    #[test]
    fn test_builder_pattern() {
        let options = CanvasOptions::new()
            .min_zoom(0.5)
            .max_zoom(4.0)
            .show_grid(false)
            .locked(true)
            .pan_speed(2.0);

        assert_eq!(options.min_zoom, 0.5);
        assert_eq!(options.max_zoom, 4.0);
        assert!(!options.show_grid);
        assert!(options.locked);
        assert_eq!(options.pan_speed, 2.0);
    }

    #[test]
    fn test_wheel_behavior() {
        assert!(WheelBehavior::Zoom.is_zoom());
        assert!(WheelBehavior::Pan.is_pan());
        assert!(WheelBehavior::None.is_none());
    }

    #[test]
    fn test_inertia_friction_clamping() {
        let options = CanvasOptions::new().inertia_friction(1.5);
        assert_eq!(options.inertia_friction, 1.0);

        let options = CanvasOptions::new().inertia_friction(-0.5);
        assert_eq!(options.inertia_friction, 0.0);
    }
}
