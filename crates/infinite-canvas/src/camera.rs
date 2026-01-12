//! Camera system for the infinite canvas.
//!
//! The camera controls the viewport into the infinite canvas, handling:
//! - Pan offset (position on the canvas)
//! - Zoom level (scale factor)
//! - Coordinate conversion between screen space and canvas space

use gpui::{Bounds, Pixels, Point, Size};
use serde::{Deserialize, Serialize};

/// The camera state for an infinite canvas.
///
/// The camera defines the viewport into the infinite canvas space.
/// It has an offset (pan position) and a zoom level.
///
/// # Coordinate Systems
///
/// - **Screen Space**: Pixels relative to the viewport's top-left corner
/// - **Canvas Space**: Coordinates on the infinite canvas
///
/// The camera provides methods to convert between these coordinate systems.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Camera {
    /// Pan offset in screen pixels.
    /// This represents where canvas origin (0,0) appears on screen.
    pub offset: Point<Pixels>,

    /// Zoom level (1.0 = 100%, 2.0 = 200%, 0.5 = 50%).
    pub zoom: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            offset: Point::default(),
            zoom: 1.0,
        }
    }
}

impl Camera {
    /// Create a new camera with default settings (no offset, 100% zoom).
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a camera with a specific offset and zoom level.
    pub fn with_offset_and_zoom(offset: Point<Pixels>, zoom: f32) -> Self {
        Self { offset, zoom }
    }

    /// Convert a point from screen space to canvas space.
    pub fn screen_to_canvas(&self, screen_point: Point<Pixels>) -> Point<Pixels> {
        Point::new(
            (screen_point.x - self.offset.x) / self.zoom,
            (screen_point.y - self.offset.y) / self.zoom,
        )
    }

    /// Convert a point from canvas space to screen space.
    pub fn canvas_to_screen(&self, canvas_point: Point<Pixels>) -> Point<Pixels> {
        Point::new(
            canvas_point.x * self.zoom + self.offset.x,
            canvas_point.y * self.zoom + self.offset.y,
        )
    }

    /// Convert bounds from canvas space to screen space.
    pub fn canvas_to_screen_bounds(&self, canvas_bounds: Bounds<Pixels>) -> Bounds<Pixels> {
        Bounds::new(
            self.canvas_to_screen(canvas_bounds.origin),
            Size::new(
                canvas_bounds.size.width * self.zoom,
                canvas_bounds.size.height * self.zoom,
            ),
        )
    }

    /// Convert bounds from screen space to canvas space.
    pub fn screen_to_canvas_bounds(&self, screen_bounds: Bounds<Pixels>) -> Bounds<Pixels> {
        Bounds::new(
            self.screen_to_canvas(screen_bounds.origin),
            Size::new(
                screen_bounds.size.width / self.zoom,
                screen_bounds.size.height / self.zoom,
            ),
        )
    }

    /// Get the visible canvas bounds for a given viewport size.
    pub fn visible_canvas_bounds(&self, viewport_size: Size<Pixels>) -> Bounds<Pixels> {
        let origin = self.screen_to_canvas(Point::default());
        let size = Size::new(
            viewport_size.width / self.zoom,
            viewport_size.height / self.zoom,
        );
        Bounds::new(origin, size)
    }

    /// Pan the camera by a delta in screen pixels.
    pub fn pan(&mut self, delta: Point<Pixels>) {
        self.offset.x += delta.x;
        self.offset.y += delta.y;
    }

    /// Pan the camera to center on a specific canvas point.
    pub fn center_on(&mut self, canvas_point: Point<Pixels>, viewport_size: Size<Pixels>) {
        self.offset.x = viewport_size.width / 2.0 - canvas_point.x * self.zoom;
        self.offset.y = viewport_size.height / 2.0 - canvas_point.y * self.zoom;
    }

    /// Zoom the camera by a factor, keeping a specific screen point fixed.
    ///
    /// This is typically used for scroll-wheel zooming where the cursor
    /// position should remain at the same canvas location after zooming.
    pub fn zoom_around(
        &mut self,
        factor: f32,
        anchor: Point<Pixels>,
        min_zoom: f32,
        max_zoom: f32,
    ) {
        let canvas_point = self.screen_to_canvas(anchor);
        let new_zoom = (self.zoom * factor).clamp(min_zoom, max_zoom);

        if (new_zoom - self.zoom).abs() < f32::EPSILON {
            return;
        }

        self.zoom = new_zoom;

        let new_screen_point = self.canvas_to_screen(canvas_point);
        self.offset.x += anchor.x - new_screen_point.x;
        self.offset.y += anchor.y - new_screen_point.y;
    }

    /// Set the zoom level, keeping the viewport center fixed.
    pub fn set_zoom(
        &mut self,
        zoom: f32,
        viewport_size: Size<Pixels>,
        min_zoom: f32,
        max_zoom: f32,
    ) {
        let center = Point::new(viewport_size.width / 2.0, viewport_size.height / 2.0);
        let factor = zoom / self.zoom;
        self.zoom_around(factor, center, min_zoom, max_zoom);
    }

    /// Reset the camera to default state (no pan, 100% zoom).
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Zoom to fit the given canvas bounds within the viewport.
    pub fn zoom_to_fit(
        &mut self,
        canvas_bounds: Bounds<Pixels>,
        viewport_size: Size<Pixels>,
        padding: Pixels,
        min_zoom: f32,
        max_zoom: f32,
    ) {
        let bounds_width: f32 = canvas_bounds.size.width.into();
        let bounds_height: f32 = canvas_bounds.size.height.into();

        if bounds_width <= 0.0 || bounds_height <= 0.0 {
            return;
        }

        let available_width = viewport_size.width - padding * 2.0;
        let available_height = viewport_size.height - padding * 2.0;

        let avail_w: f32 = available_width.into();
        let avail_h: f32 = available_height.into();

        if avail_w <= 0.0 || avail_h <= 0.0 {
            return;
        }

        let zoom_x = avail_w / bounds_width;
        let zoom_y = avail_h / bounds_height;
        let zoom = zoom_x.min(zoom_y).clamp(min_zoom, max_zoom);

        self.zoom = zoom;

        let bounds_center = Point::new(
            canvas_bounds.origin.x + canvas_bounds.size.width / 2.0,
            canvas_bounds.origin.y + canvas_bounds.size.height / 2.0,
        );

        self.center_on(bounds_center, viewport_size);
    }

    /// Get the next discrete zoom level (for stepping zoom in).
    pub fn next_zoom_step(&self, zoom_steps: &[f32]) -> f32 {
        for &step in zoom_steps {
            if step > self.zoom + 0.001 {
                return step;
            }
        }
        self.zoom
    }

    /// Get the previous discrete zoom level (for stepping zoom out).
    pub fn prev_zoom_step(&self, zoom_steps: &[f32]) -> f32 {
        for &step in zoom_steps.iter().rev() {
            if step < self.zoom - 0.001 {
                return step;
            }
        }
        self.zoom
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{point, px, size};

    #[test]
    fn test_default_camera() {
        let camera = Camera::default();
        assert_eq!(camera.offset, Point::default());
        assert_eq!(camera.zoom, 1.0);
    }

    #[test]
    fn test_screen_to_canvas_no_transform() {
        let camera = Camera::default();
        let screen_point = point(px(100.), px(200.));
        let canvas_point = camera.screen_to_canvas(screen_point);

        assert_eq!(canvas_point.x, px(100.));
        assert_eq!(canvas_point.y, px(200.));
    }

    #[test]
    fn test_screen_to_canvas_with_offset() {
        let camera = Camera::with_offset_and_zoom(point(px(50.), px(100.)), 1.0);
        let screen_point = point(px(150.), px(200.));
        let canvas_point = camera.screen_to_canvas(screen_point);

        assert_eq!(canvas_point.x, px(100.));
        assert_eq!(canvas_point.y, px(100.));
    }

    #[test]
    fn test_screen_to_canvas_with_zoom() {
        let camera = Camera::with_offset_and_zoom(point(px(0.), px(0.)), 2.0);
        let screen_point = point(px(200.), px(100.));
        let canvas_point = camera.screen_to_canvas(screen_point);

        assert_eq!(canvas_point.x, px(100.));
        assert_eq!(canvas_point.y, px(50.));
    }

    #[test]
    fn test_roundtrip_conversion() {
        let camera = Camera::with_offset_and_zoom(point(px(100.), px(50.)), 1.5);
        let original = point(px(200.), px(300.));

        let canvas_point = camera.screen_to_canvas(original);
        let back_to_screen = camera.canvas_to_screen(canvas_point);

        let back_x: f32 = back_to_screen.x.into();
        let back_y: f32 = back_to_screen.y.into();
        let orig_x: f32 = original.x.into();
        let orig_y: f32 = original.y.into();

        assert!((back_x - orig_x).abs() < 0.001);
        assert!((back_y - orig_y).abs() < 0.001);
    }

    #[test]
    fn test_pan() {
        let mut camera = Camera::default();
        camera.pan(point(px(10.), px(20.)));

        assert_eq!(camera.offset.x, px(10.));
        assert_eq!(camera.offset.y, px(20.));
    }

    #[test]
    fn test_visible_canvas_bounds() {
        let camera = Camera::with_offset_and_zoom(point(px(0.), px(0.)), 2.0);
        let viewport_size = size(px(800.), px(600.));
        let visible = camera.visible_canvas_bounds(viewport_size);

        assert_eq!(visible.origin.x, px(0.));
        assert_eq!(visible.origin.y, px(0.));
        assert_eq!(visible.size.width, px(400.));
        assert_eq!(visible.size.height, px(300.));
    }

    #[test]
    fn test_zoom_steps() {
        let camera = Camera::with_offset_and_zoom(point(px(0.), px(0.)), 1.0);
        let steps = vec![0.25, 0.5, 1.0, 2.0, 4.0];

        assert_eq!(camera.next_zoom_step(&steps), 2.0);
        assert_eq!(camera.prev_zoom_step(&steps), 0.5);
    }
}
