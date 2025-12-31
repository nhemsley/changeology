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

use diff_text_view::DiffTextView;

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(800.), px(600.)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                // Sample diff for testing
                let old_text = r#"fn main() {
    println!("Hello, world!");
}
"#;
                let new_text = r#"fn main() {
    // Print a greeting
    let name = "Rust";
    println!("Hello, {}!", name);
}
"#;

                cx.new(|_| DiffTextView::new(old_text, new_text))
            },
        )
        .unwrap();

        cx.activate(true);
    });
}
