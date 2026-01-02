mod app;
mod menu;
mod panels;

use gpui::*;
use gpui_component::{Root, TitleBar};

fn main() {
    let app = Application::new();

    app.run(move |cx| {
        // REQUIRED: Initialize gpui-component before using any features
        gpui_component::init(cx);

        // Register actions
        menu::register_actions(cx);

        cx.spawn(async move |cx| {
            let options = WindowOptions {
                titlebar: Some(TitleBar::title_bar_options()),
                window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                    Point::new(px(100.), px(100.)),
                    size(px(1200.), px(800.)),
                ))),
                ..Default::default()
            };

            cx.open_window(options, |window, cx| {
                let view = cx.new(|cx| app::ChangeologyApp::new(window, cx));
                // REQUIRED: Root must wrap the application view
                cx.new(|cx| Root::new(view, window, cx))
            })?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
