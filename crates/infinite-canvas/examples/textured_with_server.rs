//! Textured Canvas Server/Client Example
//!
//! This example demonstrates how to render GPUI elements to textures using a
//! separate server process, then load and display them in a client.
//!
//! This works around the arena lifecycle issues where elements cannot be passed
//! between different GPUI Application instances.
//!
//! Usage:
//!   # First, run the server to generate textures:
//!   cargo run -p infinite-canvas --example textured_with_server -- --server
//!
//!   # Then, run the client to display them:
//!   cargo run -p infinite-canvas --example textured_with_server

use gpui::*;
use std::path::PathBuf;

const TEXTURE_DIR: &str = "/tmp/infinite-canvas-textures";

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--server") {
        run_server();
    } else {
        run_client();
    }
}

// =============================================================================
// SERVER MODE
// =============================================================================

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
fn run_server() {
    use std::cell::RefCell;
    use std::fs;
    use std::rc::Rc;

    println!("=== Texture Server ===");
    println!("Output directory: {}", TEXTURE_DIR);

    // Create output directory
    fs::create_dir_all(TEXTURE_DIR).expect("Failed to create texture directory");

    // Define items to render
    let items = vec![
        ItemSpec {
            id: "hello".to_string(),
            width: 200,
            height: 80,
        },
        ItemSpec {
            id: "card-red".to_string(),
            width: 300,
            height: 150,
        },
        ItemSpec {
            id: "card-green".to_string(),
            width: 250,
            height: 180,
        },
        ItemSpec {
            id: "card-purple".to_string(),
            width: 350,
            height: 250,
        },
    ];

    // Track results
    let results: Rc<RefCell<Vec<(String, bool)>>> = Rc::new(RefCell::new(Vec::new()));

    for item in items {
        println!("\nRendering: {} ({}x{})", item.id, item.width, item.height);

        let item_id = item.id.clone();
        let results_clone = results.clone();

        let app = Application::textured();

        app.run(move |cx: &mut App| {
            let bounds = Bounds::centered(
                None,
                size(px(item.width as f32), px(item.height as f32)),
                cx,
            );

            let view_id = item_id.clone();
            let window_result = cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                move |_, cx| cx.new(|_| ServerRenderView::new(&view_id)),
            );

            match window_result {
                Ok(window) => {
                    let handle: AnyWindowHandle = window.into();
                    let id_for_spawn = item_id.clone();
                    let width = item.width;
                    let height = item.height;
                    let results_for_spawn = results_clone.clone();

                    cx.spawn(async move |cx| {
                        // Draw
                        let draw_result =
                            cx.update_window(handle, |_, window, cx| window.draw_and_present(cx));

                        let success = match draw_result {
                            Ok(true) => {
                                // Read pixels
                                let read_result =
                                    cx.update_window(handle, |_, window, _| window.read_pixels());

                                match read_result {
                                    Ok(Some(pixels)) => {
                                        // Save to PNG
                                        let path = format!("{}/{}.png", TEXTURE_DIR, id_for_spawn);
                                        match save_png(&pixels, width, height, &path) {
                                            Ok(()) => {
                                                println!("  ✓ Saved: {}", path);
                                                true
                                            }
                                            Err(e) => {
                                                println!("  ✗ Failed to save PNG: {}", e);
                                                false
                                            }
                                        }
                                    }
                                    Ok(None) => {
                                        println!("  ✗ read_pixels returned None");
                                        false
                                    }
                                    Err(e) => {
                                        println!("  ✗ Failed to read pixels: {}", e);
                                        false
                                    }
                                }
                            }
                            Ok(false) => {
                                println!("  ✗ Window not ready for drawing");
                                false
                            }
                            Err(e) => {
                                println!("  ✗ Draw failed: {}", e);
                                false
                            }
                        };

                        results_for_spawn
                            .borrow_mut()
                            .push((id_for_spawn, success));

                        let _ = cx.update(|cx| cx.quit());
                    })
                    .detach();
                }
                Err(e) => {
                    println!("  ✗ Failed to open window: {}", e);
                    results_clone.borrow_mut().push((item_id, false));
                    cx.quit();
                }
            }
        });
    }

    // Summary
    println!("\n=== Summary ===");
    let results = results.borrow();
    let success_count = results.iter().filter(|(_, ok)| *ok).count();
    println!(
        "Rendered {}/{} textures successfully",
        success_count,
        results.len()
    );

    // Write manifest
    let manifest_path = format!("{}/manifest.txt", TEXTURE_DIR);
    let manifest_content: String = results
        .iter()
        .filter(|(_, ok)| *ok)
        .map(|(id, _)| format!("{}.png", id))
        .collect::<Vec<_>>()
        .join("\n");

    if let Err(e) = std::fs::write(&manifest_path, manifest_content) {
        println!("Failed to write manifest: {}", e);
    } else {
        println!("Manifest written: {}", manifest_path);
    }

    println!("\nNow run without --server to view the textures.");
}

#[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
fn run_server() {
    eprintln!("Server mode requires Linux or FreeBSD (for Application::textured())");
    std::process::exit(1);
}

struct ItemSpec {
    id: String,
    width: u32,
    height: u32,
}

/// View that renders content based on item ID
struct ServerRenderView {
    id: String,
}

impl ServerRenderView {
    fn new(id: &str) -> Self {
        Self { id: id.to_string() }
    }
}

impl Render for ServerRenderView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        match self.id.as_str() {
            "hello" => div()
                .size_full()
                .bg(rgb(0x3498db))
                .rounded_lg()
                .flex()
                .justify_center()
                .items_center()
                .child(div().text_xl().text_color(white()).child("Hello World"))
                .into_any_element(),

            "card-red" => div()
                .size_full()
                .p_4()
                .bg(rgb(0xe74c3c))
                .rounded_lg()
                .flex()
                .flex_col()
                .gap_2()
                .child(div().text_2xl().text_color(white()).child("Red Card"))
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0xffcccc))
                        .child("Rendered by server process"),
                )
                .into_any_element(),

            "card-green" => div()
                .size_full()
                .p_4()
                .bg(rgb(0x2ecc71))
                .rounded_lg()
                .flex()
                .flex_col()
                .gap_2()
                .child(div().text_xl().text_color(white()).child("Green Card"))
                .child(div().flex().gap_1().children((1..=3).map(|i| {
                    div()
                        .px_2()
                        .py_1()
                        .bg(rgb(0x27ae60))
                        .rounded_md()
                        .text_xs()
                        .text_color(white())
                        .child(format!("Tag {}", i))
                })))
                .into_any_element(),

            "card-purple" => div()
                .size_full()
                .p_6()
                .bg(rgb(0x9b59b6))
                .rounded_xl()
                .flex()
                .flex_col()
                .gap_3()
                .child(div().text_3xl().text_color(white()).child("Purple Box"))
                .child(
                    div()
                        .text_base()
                        .text_color(rgb(0xd6b4fc))
                        .child("A larger card with more padding"),
                )
                .child(
                    div()
                        .mt_2()
                        .p_3()
                        .bg(rgb(0x8e44ad))
                        .rounded_lg()
                        .text_sm()
                        .text_color(white())
                        .child("Nested content box"),
                )
                .into_any_element(),

            _ => div()
                .size_full()
                .bg(rgb(0x95a5a6))
                .flex()
                .justify_center()
                .items_center()
                .child(format!("Unknown: {}", self.id))
                .into_any_element(),
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
fn save_png(pixels: &[u8], width: u32, height: u32, path: &str) -> Result<(), String> {
    // Convert BGRA to RGBA
    let mut rgba = Vec::with_capacity(pixels.len());
    for chunk in pixels.chunks(4) {
        if chunk.len() == 4 {
            rgba.push(chunk[2]); // R (was B)
            rgba.push(chunk[1]); // G
            rgba.push(chunk[0]); // B (was R)
            rgba.push(chunk[3]); // A
        }
    }

    let img = image::RgbaImage::from_raw(width, height, rgba)
        .ok_or_else(|| "Failed to create image from pixels".to_string())?;

    img.save(path)
        .map_err(|e| format!("Failed to save image: {}", e))?;

    Ok(())
}

// =============================================================================
// CLIENT MODE
// =============================================================================

fn run_client() {
    println!("=== Texture Client ===");
    println!("Looking for textures in: {}", TEXTURE_DIR);

    Application::new().run(|cx: &mut App| {
        // Load available textures
        let textures = load_textures();

        if textures.is_empty() {
            eprintln!("No textures found! Run with --server first to generate them.");
            cx.quit();
            return;
        }

        println!("Loaded {} textures", textures.len());

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1200.0), px(800.0)),
                    cx,
                ))),
                titlebar: Some(TitlebarOptions {
                    title: Some("Textured Canvas Client".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(|_| ClientCanvasView::new(textures)),
        )
        .unwrap();
    });
}

/// Loaded texture info
#[allow(dead_code)]
struct LoadedTexture {
    id: String,
    #[allow(dead_code)]
    path: PathBuf,
    image: std::sync::Arc<RenderImage>,
    position: Point<Pixels>,
    size: Size<Pixels>,
}

fn load_textures() -> Vec<LoadedTexture> {
    use image::GenericImageView;

    let texture_dir = PathBuf::from(TEXTURE_DIR);
    let manifest_path = texture_dir.join("manifest.txt");

    let manifest = match std::fs::read_to_string(&manifest_path) {
        Ok(content) => content,
        Err(_) => {
            eprintln!("Could not read manifest at {:?}", manifest_path);
            return Vec::new();
        }
    };

    let mut textures = Vec::new();
    let mut x_offset = 50.0_f32;
    let mut y_offset = 100.0_f32;
    let mut row_height = 0.0_f32;

    for line in manifest.lines() {
        let filename = line.trim();
        if filename.is_empty() {
            continue;
        }

        let path = texture_dir.join(filename);
        let id = filename.trim_end_matches(".png").to_string();

        match image::open(&path) {
            Ok(img) => {
                let (width, height) = img.dimensions();
                let rgba = img.to_rgba8();

                // Create RenderImage
                let frame = image::Frame::new(rgba);
                let render_image = RenderImage::new(smallvec::smallvec![frame]);

                // Simple layout: place items in rows
                if x_offset + width as f32 > 1100.0 {
                    x_offset = 50.0;
                    y_offset += row_height + 20.0;
                    row_height = 0.0;
                }

                textures.push(LoadedTexture {
                    id,
                    path,
                    image: std::sync::Arc::new(render_image),
                    position: point(px(x_offset), px(y_offset)),
                    size: size(px(width as f32), px(height as f32)),
                });

                x_offset += width as f32 + 20.0;
                row_height = row_height.max(height as f32);

                println!("  ✓ Loaded: {} ({}x{})", filename, width, height);
            }
            Err(e) => {
                eprintln!("  ✗ Failed to load {}: {}", filename, e);
            }
        }
    }

    textures
}

/// Client canvas view
struct ClientCanvasView {
    textures: Vec<LoadedTexture>,
    camera_offset: Point<Pixels>,
    camera_zoom: f32,
    is_panning: bool,
    last_mouse_pos: Point<Pixels>,
}

impl ClientCanvasView {
    fn new(textures: Vec<LoadedTexture>) -> Self {
        Self {
            textures,
            camera_offset: point(px(0.0), px(0.0)),
            camera_zoom: 1.0,
            is_panning: false,
            last_mouse_pos: point(px(0.0), px(0.0)),
        }
    }
}

impl Render for ClientCanvasView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let viewport = window.viewport_size();
        let viewport_bounds = Bounds::new(point(px(0.0), px(0.0)), viewport);

        // Render texture items
        let mut items: Vec<AnyElement> = Vec::new();

        for tex in &self.textures {
            // Transform by camera
            let screen_pos = point(
                (tex.position.x + self.camera_offset.x) * self.camera_zoom,
                (tex.position.y + self.camera_offset.y) * self.camera_zoom,
            );
            let screen_size = size(
                tex.size.width * self.camera_zoom,
                tex.size.height * self.camera_zoom,
            );
            let screen_bounds = Bounds::new(screen_pos, screen_size);

            // Culling
            if !bounds_intersect(&screen_bounds, &viewport_bounds) {
                continue;
            }

            items.push(
                div()
                    .absolute()
                    .left(screen_bounds.origin.x)
                    .top(screen_bounds.origin.y)
                    .w(screen_bounds.size.width)
                    .h(screen_bounds.size.height)
                    .rounded_lg()
                    .overflow_hidden()
                    .child(
                        img(tex.image.clone())
                            .size_full()
                            .object_fit(ObjectFit::Fill),
                    )
                    .into_any_element(),
            );
        }

        div()
            .size_full()
            .bg(rgb(0xf0f0f0))
            .overflow_hidden()
            // Pan with middle mouse
            .on_mouse_down(MouseButton::Middle, {
                cx.listener(|this, event: &MouseDownEvent, _window, _cx| {
                    this.is_panning = true;
                    this.last_mouse_pos = event.position;
                })
            })
            .on_mouse_up(MouseButton::Middle, {
                cx.listener(|this, _event: &MouseUpEvent, _window, _cx| {
                    this.is_panning = false;
                })
            })
            .on_mouse_move({
                cx.listener(|this, event: &MouseMoveEvent, _window, cx| {
                    if this.is_panning {
                        let delta = point(
                            event.position.x - this.last_mouse_pos.x,
                            event.position.y - this.last_mouse_pos.y,
                        );
                        this.camera_offset.x += delta.x / this.camera_zoom;
                        this.camera_offset.y += delta.y / this.camera_zoom;
                        this.last_mouse_pos = event.position;
                        cx.notify();
                    }
                })
            })
            // Zoom with scroll
            .on_scroll_wheel({
                cx.listener(|this, event: &ScrollWheelEvent, _window, cx| {
                    let delta = event.delta.pixel_delta(px(1.0));
                    let delta_y: f32 = delta.y.into();
                    let zoom_factor = 1.0 + (delta_y / 500.0);
                    this.camera_zoom = (this.camera_zoom * zoom_factor).clamp(0.1, 10.0);
                    cx.notify();
                })
            })
            // Header
            .child(
                div()
                    .absolute()
                    .top_2()
                    .left_2()
                    .px_3()
                    .py_2()
                    .bg(rgb(0x333333))
                    .rounded_md()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_sm()
                            .text_color(white())
                            .child(format!(
                                "Textures: {} | Zoom: {:.0}%",
                                self.textures.len(),
                                self.camera_zoom * 100.0
                            )),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0xaaaaaa))
                            .child("Middle-drag to pan • Scroll to zoom"),
                    ),
            )
            // Canvas items
            .children(items)
    }
}

/// Check if two bounds intersect
fn bounds_intersect(a: &Bounds<Pixels>, b: &Bounds<Pixels>) -> bool {
    a.origin.x < b.origin.x + b.size.width
        && a.origin.x + a.size.width > b.origin.x
        && a.origin.y < b.origin.y + b.size.height
        && a.origin.y + a.size.height > b.origin.y
}
