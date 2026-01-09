//! Textured Canvas Items Provider
//!
//! This module provides a canvas items provider that renders items to textures
//! using GPUI's `Application::textured()` feature. Items are rendered asynchronously
//! in background threads and cached for efficient pan/zoom.
//!
//! # Architecture
//!
//! - Items are defined with closures that return GPUI elements
//! - When a texture is requested, a background render thread is spawned
//! - Frames are streamed back via flume channels
//! - Textures are cached and displayed as images
//! - Supports flexible sizing via `ItemSizing` (fixed, measured height, explicit)
//!
//! # Example
//!
//! ```ignore
//! use infinite_canvas::textured_provider::TexturedCanvasItemsProvider;
//! use gpui::*;
//!
//! let mut provider = TexturedCanvasItemsProvider::new();
//!
//! provider.add_item("card-1", || {
//!     div()
//!         .p_4()
//!         .bg(rgb(0x3498db))
//!         .child("Hello World")
//!         .into_any_element()
//! });
//! ```
//!
//! # Platform Support
//!
//! Texture rendering requires `Application::textured()` support,
//! which is only available on Linux/FreeBSD. On other platforms, `tick()` will return
//! errors for render requests.

use gpui::{
    AnyElement, App, AppContext as _, Bounds, IntoElement, ItemSizing, ObjectFit, ParentElement,
    Pixels, Point, RenderImage, RenderOnce, Size, Styled, StyledImage, Window, div, img, point, px,
    size,
};

// ============================================================================
// Downscale Modes - for preserving syntax highlighting when zoomed out
// ============================================================================

/// Downscale mode for texture rendering.
///
/// When textures are displayed smaller than their rendered size, the downscale
/// mode determines how colors are preserved. This is especially important for
/// syntax-highlighted code where naive averaging washes out colors.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DownscaleMode {
    /// Standard linear interpolation (averaging).
    /// Fast but washes out syntax highlighting colors.
    #[default]
    Linear,

    /// Preserve the most saturated color in each block.
    /// Good for syntax highlighting - colorful tokens "win" over white/gray background.
    MostSaturated,

    /// Preserve colors furthest from background.
    /// Best when you know the background color.
    FurthestFromBackground,

    /// Min pooling - darkest pixel wins.
    /// Good for dark text on light background.
    MinPool,

    /// Max pooling - brightest pixel wins.
    /// Good for light text on dark background.
    MaxPool,
}

impl DownscaleMode {
    /// Get all available downscale modes for UI selection.
    pub fn all() -> &'static [DownscaleMode] {
        &[
            DownscaleMode::Linear,
            DownscaleMode::MostSaturated,
            DownscaleMode::FurthestFromBackground,
            DownscaleMode::MinPool,
            DownscaleMode::MaxPool,
        ]
    }

    /// Get display name for UI.
    pub fn display_name(&self) -> &'static str {
        match self {
            DownscaleMode::Linear => "Linear (Standard)",
            DownscaleMode::MostSaturated => "Most Saturated",
            DownscaleMode::FurthestFromBackground => "Furthest from BG",
            DownscaleMode::MinPool => "Min Pool (Darkest)",
            DownscaleMode::MaxPool => "Max Pool (Brightest)",
        }
    }
}

/// Apply downscaling to RGBA pixels.
///
/// # Arguments
/// * `pixels` - RGBA pixel data
/// * `width` - Source width
/// * `height` - Source height
/// * `scale` - Downscale factor (2 = half size, 4 = quarter size, etc.)
/// * `mode` - Downscale algorithm to use
/// * `bg_color` - Background color (used for FurthestFromBackground mode)
///
/// # Returns
/// Downscaled RGBA pixel data and new dimensions (width, height)
pub fn downscale_pixels(
    pixels: &[u8],
    width: u32,
    height: u32,
    scale: u32,
    mode: DownscaleMode,
    bg_color: [u8; 4],
) -> (Vec<u8>, u32, u32) {
    if scale <= 1 {
        return (pixels.to_vec(), width, height);
    }

    let dst_w = width / scale;
    let dst_h = height / scale;

    if dst_w == 0 || dst_h == 0 {
        return (pixels.to_vec(), width, height);
    }

    let mut result = vec![0u8; (dst_w * dst_h * 4) as usize];

    for dy in 0..dst_h {
        for dx in 0..dst_w {
            let color = match mode {
                DownscaleMode::Linear => {
                    average_block(pixels, dx * scale, dy * scale, scale, width)
                }
                DownscaleMode::MostSaturated => {
                    most_saturated_in_block(pixels, dx * scale, dy * scale, scale, width)
                }
                DownscaleMode::FurthestFromBackground => furthest_from_bg_in_block(
                    pixels,
                    dx * scale,
                    dy * scale,
                    scale,
                    width,
                    bg_color,
                ),
                DownscaleMode::MinPool => {
                    min_pool_block(pixels, dx * scale, dy * scale, scale, width)
                }
                DownscaleMode::MaxPool => {
                    max_pool_block(pixels, dx * scale, dy * scale, scale, width)
                }
            };

            let idx = ((dy * dst_w + dx) * 4) as usize;
            result[idx] = color[0];
            result[idx + 1] = color[1];
            result[idx + 2] = color[2];
            result[idx + 3] = color[3];
        }
    }

    (result, dst_w, dst_h)
}

/// Average all pixels in a block (standard linear downscale).
fn average_block(pixels: &[u8], sx: u32, sy: u32, scale: u32, src_w: u32) -> [u8; 4] {
    let mut r_sum: u32 = 0;
    let mut g_sum: u32 = 0;
    let mut b_sum: u32 = 0;
    let mut a_sum: u32 = 0;
    let mut count: u32 = 0;

    for by in 0..scale {
        for bx in 0..scale {
            let x = sx + bx;
            let y = sy + by;
            if let Some(pixel) = get_pixel(pixels, x, y, src_w) {
                r_sum += pixel[0] as u32;
                g_sum += pixel[1] as u32;
                b_sum += pixel[2] as u32;
                a_sum += pixel[3] as u32;
                count += 1;
            }
        }
    }

    if count == 0 {
        return [0, 0, 0, 255];
    }

    [
        (r_sum / count) as u8,
        (g_sum / count) as u8,
        (b_sum / count) as u8,
        (a_sum / count) as u8,
    ]
}

/// Find the most saturated color in a block.
fn most_saturated_in_block(pixels: &[u8], sx: u32, sy: u32, scale: u32, src_w: u32) -> [u8; 4] {
    let mut best_color: Option<[u8; 4]> = None;
    let mut best_saturation: u32 = 0;

    // Also track average for fallback when no saturated pixels
    let mut r_sum: u32 = 0;
    let mut g_sum: u32 = 0;
    let mut b_sum: u32 = 0;
    let mut a_sum: u32 = 0;
    let mut count: u32 = 0;

    for by in 0..scale {
        for bx in 0..scale {
            let x = sx + bx;
            let y = sy + by;
            if let Some(pixel) = get_pixel(pixels, x, y, src_w) {
                // Track for average
                r_sum += pixel[0] as u32;
                g_sum += pixel[1] as u32;
                b_sum += pixel[2] as u32;
                a_sum += pixel[3] as u32;
                count += 1;

                let sat = color_saturation(&pixel);
                if sat > best_saturation {
                    best_saturation = sat;
                    best_color = Some(pixel);
                }
            }
        }
    }

    // If we found a saturated color, use it; otherwise fall back to average
    if let Some(color) = best_color {
        if best_saturation > 10 {
            // Threshold to avoid picking near-gray as "saturated"
            return color;
        }
    }

    // Fall back to average color (like Linear mode)
    if count > 0 {
        [
            (r_sum / count) as u8,
            (g_sum / count) as u8,
            (b_sum / count) as u8,
            (a_sum / count) as u8,
        ]
    } else {
        [0, 0, 0, 255]
    }
}

/// Find the color furthest from background in a block.
fn furthest_from_bg_in_block(
    pixels: &[u8],
    sx: u32,
    sy: u32,
    scale: u32,
    src_w: u32,
    bg_color: [u8; 4],
) -> [u8; 4] {
    let mut best_color = bg_color;
    let mut best_distance: u32 = 0;

    for by in 0..scale {
        for bx in 0..scale {
            let x = sx + bx;
            let y = sy + by;
            if let Some(pixel) = get_pixel(pixels, x, y, src_w) {
                let dist = color_distance(&pixel, &bg_color);
                if dist > best_distance {
                    best_distance = dist;
                    best_color = pixel;
                }
            }
        }
    }

    best_color
}

/// Find the darkest pixel in a block (min pooling).
fn min_pool_block(pixels: &[u8], sx: u32, sy: u32, scale: u32, src_w: u32) -> [u8; 4] {
    let mut best_color = [255u8, 255, 255, 255];
    let mut best_luminance: u32 = u32::MAX;

    for by in 0..scale {
        for bx in 0..scale {
            let x = sx + bx;
            let y = sy + by;
            if let Some(pixel) = get_pixel(pixels, x, y, src_w) {
                let lum = pixel[0] as u32 + pixel[1] as u32 + pixel[2] as u32;
                if lum < best_luminance {
                    best_luminance = lum;
                    best_color = pixel;
                }
            }
        }
    }

    best_color
}

/// Find the brightest pixel in a block (max pooling).
fn max_pool_block(pixels: &[u8], sx: u32, sy: u32, scale: u32, src_w: u32) -> [u8; 4] {
    let mut best_color = [0u8, 0, 0, 255];
    let mut best_luminance: u32 = 0;

    for by in 0..scale {
        for bx in 0..scale {
            let x = sx + bx;
            let y = sy + by;
            if let Some(pixel) = get_pixel(pixels, x, y, src_w) {
                let lum = pixel[0] as u32 + pixel[1] as u32 + pixel[2] as u32;
                if lum > best_luminance {
                    best_luminance = lum;
                    best_color = pixel;
                }
            }
        }
    }

    best_color
}

/// Get a pixel from the buffer, returning None if out of bounds.
fn get_pixel(pixels: &[u8], x: u32, y: u32, width: u32) -> Option<[u8; 4]> {
    let idx = ((y * width + x) * 4) as usize;
    if idx + 3 < pixels.len() {
        Some([
            pixels[idx],
            pixels[idx + 1],
            pixels[idx + 2],
            pixels[idx + 3],
        ])
    } else {
        None
    }
}

/// Calculate color saturation (max - min of RGB).
fn color_saturation(pixel: &[u8; 4]) -> u32 {
    let max = pixel[0].max(pixel[1]).max(pixel[2]);
    let min = pixel[0].min(pixel[1]).min(pixel[2]);
    (max - min) as u32
}

/// Calculate color distance (sum of absolute RGB differences).
fn color_distance(a: &[u8; 4], b: &[u8; 4]) -> u32 {
    let dr = (a[0] as i32 - b[0] as i32).abs() as u32;
    let dg = (a[1] as i32 - b[1] as i32).abs() as u32;
    let db = (a[2] as i32 - b[2] as i32).abs() as u32;
    dr + dg + db
}
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::thread::JoinHandle;

use flume::Receiver;

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
use gpui::{AnyWindowHandle, Application, Context, Render, Timer, WindowBounds, WindowOptions};

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
use std::time::Duration;

/// String-based identifier for canvas items
pub type CanvasItemId = String;

/// Description of a canvas item for display
#[derive(Clone, Debug)]
pub struct CanvasItemDescriptor {
    /// Unique identifier
    pub id: CanvasItemId,
    /// Position and size in canvas world coordinates
    pub bounds: Bounds<Pixels>,
    /// Z-order for layering
    pub z_index: u32,
}

/// State of a texture
#[derive(Clone)]
pub enum TextureState {
    /// Not yet requested
    NotRequested,
    /// Rendering in background thread
    Rendering,
    /// Ready to display
    Ready {
        image: Arc<RenderImage>,
        size: Size<Pixels>,
    },
    /// Rendering failed
    Failed(String),
}

impl std::fmt::Debug for TextureState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextureState::NotRequested => write!(f, "NotRequested"),
            TextureState::Rendering => write!(f, "Rendering"),
            TextureState::Ready { size, .. } => write!(f, "Ready({:?})", size),
            TextureState::Failed(e) => write!(f, "Failed({})", e),
        }
    }
}

/// Frame data sent from background render thread
#[derive(Debug)]
struct RenderedFrame {
    /// Raw pixel data (already converted to RGBA)
    pixels: Vec<u8>,
    /// Frame width in pixels
    width: u32,
    /// Frame height in pixels
    height: u32,
}

/// Active render job for an item
struct RenderJob {
    /// Channel to receive rendered frames
    receiver: Receiver<RenderedFrame>,
    /// Handle to the background thread (kept for cleanup)
    #[allow(dead_code)]
    thread_handle: JoinHandle<()>,
}

/// Internal item definition
struct ItemDefinition {
    id: CanvasItemId,
    origin: Point<Pixels>,
    size: Option<Size<Pixels>>,
    z_index: u32,
    factory: Arc<dyn Fn() -> AnyElement + Send + Sync>,
}

/// Trait for providing canvas items and their textures
pub trait CanvasItemsProvider {
    /// Get all items to display
    fn items(&self) -> Vec<CanvasItemDescriptor>;

    /// Get texture state for an item
    fn texture_state(&self, id: &str) -> TextureState;

    /// Request texture rendering (queues it)
    fn request_texture(&mut self, id: &str);

    /// Process render queue (call once per frame)
    /// Returns true if any work was done
    fn tick(&mut self) -> bool;
}

/// A canvas items provider that renders items to textures asynchronously
pub struct TexturedCanvasItemsProvider {
    /// Item definitions
    items: HashMap<CanvasItemId, ItemDefinition>,
    /// Texture states
    textures: HashMap<CanvasItemId, TextureState>,
    /// Pending render requests (not yet started)
    render_queue: VecDeque<CanvasItemId>,
    /// Active render jobs (background threads running)
    active_jobs: HashMap<CanvasItemId, RenderJob>,
    /// Max concurrent render jobs
    max_concurrent_renders: usize,
    /// Sizing strategy for items
    sizing: ItemSizing,
}

impl TexturedCanvasItemsProvider {
    /// Create a new textured canvas items provider with default sizing
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            textures: HashMap::new(),
            render_queue: VecDeque::new(),
            active_jobs: HashMap::new(),
            max_concurrent_renders: 4,
            sizing: ItemSizing::default(),
        }
    }

    /// Create a new provider with specific sizing
    pub fn with_sizing(sizing: ItemSizing) -> Self {
        Self {
            items: HashMap::new(),
            textures: HashMap::new(),
            render_queue: VecDeque::new(),
            active_jobs: HashMap::new(),
            max_concurrent_renders: 4,
            sizing,
        }
    }

    /// Set the sizing strategy for all items
    pub fn set_sizing(&mut self, sizing: ItemSizing) {
        self.sizing = sizing;
    }

    /// Get the current sizing strategy
    pub fn sizing(&self) -> &ItemSizing {
        &self.sizing
    }

    /// Add an item (auto-positioned at origin)
    pub fn add_item<F>(&mut self, id: impl Into<String>, factory: F)
    where
        F: Fn() -> AnyElement + Send + Sync + 'static,
    {
        self.add_item_at(id, point(px(0.0), px(0.0)), factory);
    }

    /// Add an item at a specific position
    pub fn add_item_at<F>(&mut self, id: impl Into<String>, origin: Point<Pixels>, factory: F)
    where
        F: Fn() -> AnyElement + Send + Sync + 'static,
    {
        let id = id.into();

        self.items.insert(
            id.clone(),
            ItemDefinition {
                id: id.clone(),
                origin,
                size: None,
                z_index: 0,
                factory: Arc::new(factory),
            },
        );

        self.textures.insert(id, TextureState::NotRequested);
    }

    /// Set item position
    pub fn set_position(&mut self, id: &str, origin: Point<Pixels>) {
        if let Some(item) = self.items.get_mut(id) {
            item.origin = origin;
        }
    }

    /// Set item z-index
    pub fn set_z_index(&mut self, id: &str, z_index: u32) {
        if let Some(item) = self.items.get_mut(id) {
            item.z_index = z_index;
        }
    }

    /// Get item bounds (None if not yet measured)
    pub fn bounds(&self, id: &str) -> Option<Bounds<Pixels>> {
        self.items
            .get(id)
            .and_then(|item| item.size.map(|size| Bounds::new(item.origin, size)))
    }

    /// Remove an item
    pub fn remove_item(&mut self, id: &str) {
        self.items.remove(id);
        self.textures.remove(id);
        self.render_queue.retain(|i| i != id);
        self.active_jobs.remove(id);
    }

    /// Invalidate an item (force re-render)
    pub fn invalidate(&mut self, id: &str) {
        if self.items.contains_key(id) {
            self.active_jobs.remove(id);
            self.textures
                .insert(id.to_string(), TextureState::NotRequested);
        }
    }

    /// Invalidate all items
    pub fn invalidate_all(&mut self) {
        self.active_jobs.clear();
        for id in self.items.keys() {
            self.textures.insert(id.clone(), TextureState::NotRequested);
        }
    }

    /// Set max concurrent renders
    pub fn set_max_concurrent_renders(&mut self, count: usize) {
        self.max_concurrent_renders = count.max(1);
    }

    /// Get number of pending renders (queued but not started)
    pub fn pending_count(&self) -> usize {
        self.render_queue.len()
    }

    /// Get number of active renders (background threads running)
    pub fn active_count(&self) -> usize {
        self.active_jobs.len()
    }

    /// Check if all textures are ready
    pub fn all_ready(&self) -> bool {
        self.textures
            .values()
            .all(|state| matches!(state, TextureState::Ready { .. }))
    }

    /// Get item IDs
    pub fn item_ids(&self) -> impl Iterator<Item = &str> {
        self.items.keys().map(|s| s.as_str())
    }

    /// Start a render job for an item (spawns background thread)
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    fn start_render_job(&mut self, id: &str) {
        if let Some(item) = self.items.get(id) {
            let factory = item.factory.clone();
            let sizing = self.sizing.clone();

            let (sender, receiver) = flume::bounded(2);

            let thread_handle = std::thread::spawn(move || {
                run_textured_renderer(sizing, factory, sender);
            });

            self.active_jobs.insert(
                id.to_string(),
                RenderJob {
                    receiver,
                    thread_handle,
                },
            );

            self.textures
                .insert(id.to_string(), TextureState::Rendering);
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
    fn start_render_job(&mut self, id: &str) {
        self.textures.insert(
            id.to_string(),
            TextureState::Failed("Textured rendering only supported on Linux/FreeBSD".into()),
        );
    }

    /// Poll active jobs for completed frames
    fn poll_active_jobs(&mut self) -> bool {
        let mut work_done = false;
        let mut completed_jobs = Vec::new();

        for (id, job) in &self.active_jobs {
            match job.receiver.try_recv() {
                Ok(frame) => {
                    // Convert pixels to RenderImage
                    if let Some(image) =
                        pixels_to_render_image(&frame.pixels, frame.width, frame.height)
                    {
                        let measured_size = size(px(frame.width as f32), px(frame.height as f32));

                        // Update item's measured size
                        if let Some(item) = self.items.get_mut(id) {
                            item.size = Some(measured_size);
                        }

                        self.textures.insert(
                            id.clone(),
                            TextureState::Ready {
                                image: Arc::new(image),
                                size: measured_size,
                            },
                        );
                    } else {
                        self.textures.insert(
                            id.clone(),
                            TextureState::Failed("Failed to convert pixels to image".into()),
                        );
                    }
                    completed_jobs.push(id.clone());
                    work_done = true;
                }
                Err(flume::TryRecvError::Empty) => {
                    // Still rendering, keep waiting
                }
                Err(flume::TryRecvError::Disconnected) => {
                    // Thread finished without sending a frame (error or panic)
                    self.textures.insert(
                        id.clone(),
                        TextureState::Failed("Render thread terminated unexpectedly".into()),
                    );
                    completed_jobs.push(id.clone());
                    work_done = true;
                }
            }
        }

        // Remove completed jobs
        for id in completed_jobs {
            self.active_jobs.remove(&id);
        }

        work_done
    }
}

impl Default for TexturedCanvasItemsProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl CanvasItemsProvider for TexturedCanvasItemsProvider {
    fn items(&self) -> Vec<CanvasItemDescriptor> {
        self.items
            .values()
            .map(|item| {
                // Use measured size, or initial size from sizing strategy
                let item_size = item.size.unwrap_or_else(|| self.sizing.initial_size());
                CanvasItemDescriptor {
                    id: item.id.clone(),
                    bounds: Bounds::new(item.origin, item_size),
                    z_index: item.z_index,
                }
            })
            .collect()
    }

    fn texture_state(&self, id: &str) -> TextureState {
        self.textures
            .get(id)
            .cloned()
            .unwrap_or(TextureState::NotRequested)
    }

    fn request_texture(&mut self, id: &str) {
        if self.items.contains_key(id) {
            if let Some(TextureState::NotRequested) = self.textures.get(id) {
                self.render_queue.push_back(id.to_string());
            }
        }
    }

    fn tick(&mut self) -> bool {
        let mut work_done = false;

        // Poll active jobs for completed frames
        if self.poll_active_jobs() {
            work_done = true;
        }

        // Start new render jobs if we have capacity
        while self.active_jobs.len() < self.max_concurrent_renders {
            if let Some(id) = self.render_queue.pop_front() {
                self.start_render_job(&id);
                work_done = true;
            } else {
                break;
            }
        }

        work_done
    }
}

/// Convert RGBA pixels to RenderImage
fn pixels_to_render_image(pixels: &[u8], width: u32, height: u32) -> Option<RenderImage> {
    use image::{Frame, RgbaImage};
    use smallvec::smallvec;

    let image = RgbaImage::from_raw(width, height, pixels.to_vec())?;
    Some(RenderImage::new(smallvec![Frame::new(image)]))
}

// ============================================================================
// Background Rendering (Linux/FreeBSD only)
// ============================================================================

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
/// Maximum texture height to prevent excessive memory usage
const MAX_TEXTURE_HEIGHT: f32 = 2048.0;

fn run_textured_renderer(
    sizing: ItemSizing,
    factory: Arc<dyn Fn() -> AnyElement + Send + Sync>,
    sender: flume::Sender<RenderedFrame>,
) {
    let app = Application::textured();

    app.run(move |cx: &mut App| {
        // For FixedWidth sizing, use a large initial height to allow content to expand
        let initial_size = match &sizing {
            ItemSizing::FixedWidth { width, estimated_height } => {
                // Use 1.5x estimated height to allow some expansion, capped at MAX_TEXTURE_HEIGHT
                let est_height: f32 = (*estimated_height).into();
                let height = (est_height * 1.5).min(MAX_TEXTURE_HEIGHT);
                size(*width, px(height))
            }
            _ => sizing.initial_size(),
        };
        let bounds = Bounds::centered(None, initial_size, cx);

        let window_result = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| BackgroundRenderer {
                    factory: factory.clone(),
                    sizing: sizing.clone(),
                    sender: sender.clone(),
                    window_handle: None,
                    phase: RenderPhase::FirstRender,
                    did_resize: false,
                })
            },
        );

        if window_result.is_err() {
            cx.quit();
        }
    });
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
#[derive(Debug, Clone, PartialEq)]
enum RenderPhase {
    /// First render - may need measurement
    FirstRender,
    /// Ready to paint and capture
    ReadyToPaint,
    /// Painted, waiting for completion
    Painted,
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
struct BackgroundRenderer {
    factory: Arc<dyn Fn() -> AnyElement + Send + Sync>,
    sizing: ItemSizing,
    sender: flume::Sender<RenderedFrame>,
    window_handle: Option<AnyWindowHandle>,
    phase: RenderPhase,
    did_resize: bool,
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
impl Render for BackgroundRenderer {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Store window handle on first render
        if self.window_handle.is_none() {
            self.window_handle = Some(window.window_handle());
        }

        let window_handle = self.window_handle.unwrap();

        match self.phase {
            RenderPhase::FirstRender => {
                // For FixedWidth mode, the window is already sized large enough
                // Just mark as did_resize to skip unnecessary resize logic
                if self.sizing.needs_measurement() && !self.did_resize {
                    self.did_resize = true;
                }

                self.phase = RenderPhase::ReadyToPaint;

                // Schedule the actual paint and capture
                let sender = self.sender.clone();

                window
                    .spawn(cx, async move |cx| {
                        // Wait for render to complete
                        Timer::after(Duration::from_millis(20)).await;

                        let _ = cx.update_window(window_handle, |_, window: &mut Window, cx| {
                            window.draw_and_present(cx);

                            if let Some(pixels) = window.read_pixels() {
                                let bounds = window.bounds();
                                let width: u32 = bounds.size.width.into();
                                let height: u32 = bounds.size.height.into();

                                // Convert BGRA to RGBA
                                let rgba = convert_bgra_to_rgba(&pixels);

                                sender
                                    .send(RenderedFrame {
                                        pixels: rgba,
                                        width,
                                        height,
                                    })
                                    .ok();
                            }
                        });

                        // Quit after rendering
                        Timer::after(Duration::from_millis(50)).await;
                        let _ = cx.update(|_, cx| cx.quit());
                    })
                    .detach();
            }

            RenderPhase::ReadyToPaint => {
                self.phase = RenderPhase::Painted;
            }

            RenderPhase::Painted => {
                // Done, waiting for async task to complete
            }
        }

        // Render the actual content
        let element = (self.factory)();

        // For FixedWidth sizing, only constrain width - let height be determined by content
        // The window is sized large enough to accommodate the content
        match &self.sizing {
            ItemSizing::FixedWidth { width, .. } => {
                div()
                    .w(*width)
                    .flex()
                    .flex_col()
                    .child(element)
            }
            _ => {
                let size = self.sizing.initial_size();
                div()
                    .w(size.width)
                    .h(size.height)
                    .overflow_hidden()
                    .child(element)
            }
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
fn convert_bgra_to_rgba(bgra: &[u8]) -> Vec<u8> {
    let mut rgba = Vec::with_capacity(bgra.len());
    for chunk in bgra.chunks(4) {
        if chunk.len() == 4 {
            rgba.push(chunk[2]); // R (was B)
            rgba.push(chunk[1]); // G
            rgba.push(chunk[0]); // B (was R)
            rgba.push(chunk[3]); // A
        }
    }
    rgba
}

// ============================================================================
// TexturedItemElement - for rendering textured items in the canvas
// ============================================================================

/// Element for rendering a textured canvas item
pub struct TexturedItemElement {
    image: Arc<RenderImage>,
    #[allow(dead_code)]
    bounds: Bounds<Pixels>,
}

impl TexturedItemElement {
    pub fn new(image: Arc<RenderImage>, bounds: Bounds<Pixels>) -> Self {
        Self { image, bounds }
    }
}

impl RenderOnce for TexturedItemElement {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        img(self.image).size_full().object_fit(ObjectFit::Fill)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::div;

    #[test]
    fn test_provider_add_item() {
        let mut provider = TexturedCanvasItemsProvider::new();

        provider.add_item("test-1", || div().into_any_element());

        assert_eq!(provider.items().len(), 1);
        assert!(matches!(
            provider.texture_state("test-1"),
            TextureState::NotRequested
        ));
    }

    #[test]
    fn test_provider_request_texture() {
        let mut provider = TexturedCanvasItemsProvider::new();

        provider.add_item("test-1", || div().into_any_element());
        provider.request_texture("test-1");

        assert_eq!(provider.pending_count(), 1);
    }

    #[test]
    fn test_provider_remove_item() {
        let mut provider = TexturedCanvasItemsProvider::new();

        provider.add_item("test-1", || div().into_any_element());
        provider.remove_item("test-1");

        assert_eq!(provider.items().len(), 0);
    }

    #[test]
    fn test_provider_invalidate() {
        let mut provider = TexturedCanvasItemsProvider::new();

        provider.add_item("test-1", || div().into_any_element());
        provider.request_texture("test-1");

        assert_eq!(provider.pending_count(), 1);

        // Invalidate puts it back to NotRequested
        provider.invalidate("test-1");

        assert!(matches!(
            provider.texture_state("test-1"),
            TextureState::NotRequested
        ));
    }

    #[test]
    fn test_provider_set_position() {
        let mut provider = TexturedCanvasItemsProvider::new();

        provider.add_item("test-1", || div().into_any_element());
        provider.set_position("test-1", point(px(100.0), px(200.0)));

        let items = provider.items();
        assert_eq!(items[0].bounds.origin.x, px(100.0));
        assert_eq!(items[0].bounds.origin.y, px(200.0));
    }

    #[test]
    fn test_provider_with_sizing() {
        let provider = TexturedCanvasItemsProvider::with_sizing(ItemSizing::FixedWidth {
            width: px(400.0),
            estimated_height: px(300.0),
        });

        assert!(matches!(provider.sizing(), ItemSizing::FixedWidth { .. }));
    }

    #[test]
    fn test_provider_max_concurrent() {
        let mut provider = TexturedCanvasItemsProvider::new();
        provider.set_max_concurrent_renders(8);

        // Add multiple items
        for i in 0..10 {
            provider.add_item(format!("test-{}", i), || div().into_any_element());
            provider.request_texture(&format!("test-{}", i));
        }

        assert_eq!(provider.pending_count(), 10);
    }
}
