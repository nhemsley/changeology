//! Diff UI - A GPUI application for displaying text diffs
//!
//! This is the entry point for the diff-ui application.
//! It creates a window and displays a diff view.

use gpui::{
    prelude::*, px, size, App, Application, Bounds,
    WindowBounds, WindowOptions,
};

mod diff_text_view;
mod theme;

use diff_text_view::{DiffTextView, RenderMode};

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(800.), px(600.)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                // Large diff for testing uniform_list performance
                // This generates a diff with 1000+ lines to demonstrate
                // that uniform_list only renders visible items
                let mut old_lines = Vec::new();
                let mut new_lines = Vec::new();

                // Add initial unchanged section
                for i in 1..=100 {
                    old_lines.push(format!("fn function_{}() {{", i));
                    old_lines.push(format!("    // Original implementation {}", i));
                    old_lines.push(format!("    println!(\"Function {}\");", i));
                    old_lines.push("}\n".to_string());

                    new_lines.push(format!("fn function_{}() {{", i));
                    new_lines.push(format!("    // Original implementation {}", i));
                    new_lines.push(format!("    println!(\"Function {}\");", i));
                    new_lines.push("}\n".to_string());
                }

                // Add some deleted lines
                for i in 101..=110 {
                    old_lines.push(format!("fn old_function_{}() {{", i));
                    old_lines.push("    // This will be deleted".to_string());
                    old_lines.push("}\n".to_string());
                }

                // Add some added lines
                for i in 111..=130 {
                    new_lines.push(format!("fn new_function_{}() {{", i));
                    new_lines.push("    // This is new code".to_string());
                    new_lines.push(format!("    let x = {};", i));
                    new_lines.push("}\n".to_string());
                }

                // Add another unchanged section
                for i in 131..=500 {
                    old_lines.push(format!("// Comment line {}", i));
                    new_lines.push(format!("// Comment line {}", i));
                }

                let old_text = old_lines.join("\n");
                let new_text = new_lines.join("\n");

                // Demo uses virtualized rendering by default
                // To use full buffer rendering instead, uncomment the line below:
                // cx.new(|_| DiffTextView::new(&old_text, &new_text).with_render_mode(RenderMode::FullBuffer))

                cx.new(|_| DiffTextView::new(&old_text, &new_text).with_render_mode(RenderMode::Virtualized))
            },
        )
        .unwrap();

        cx.activate(true);
    });
}
