# Changeology UI Implementation Template

This document provides concrete starter code for implementing the changeology UI with the **vendored** gpui-component library.

## File Structure

```
crates/changeology/
├── Cargo.toml
└── src/
    ├── changeology.rs    (main entry point)
    ├── app.rs            (main application state and render)
    ├── panels/
    │   ├── mod.rs
    │   ├── file_tree.rs  (file tree panel)
    │   └── history.rs    (commit history panel)
    └── menu.rs           (menu bar actions)
```

---

## Cargo.toml Updates

```toml
[package]
name = "changeology"
version = "0.1.0"
edition = "2021"
publish = false

[[bin]]
name = "changeology"
path = "src/changeology.rs"

[dependencies]
# Workspace dependencies
buffer-diff = { path = "../diff" }
git = { path = "../git" }

# GPUI (from vendored zed)
gpui = { path = "../../../vendor/zed/crates/gpui" }

# GPUI Component (vendored)
gpui-component = { path = "../../../vendor/gpui-component/crates/ui" }

# General
anyhow = "1.0"
```

---

## Main Entry Point (src/changeology.rs)

```rust
mod app;
mod menu;
mod panels;

use anyhow::Result;
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
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1200.), px(800.)),
                    cx.as_ref(),
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
```

---

## Menu Actions (src/menu.rs)

```rust
use gpui::*;

// Define actions using the actions! macro
actions!(
    changeology,
    [
        OpenRepository,
        CloseRepository,
        Refresh,
        Quit,
        ToggleSidebar,
        ShowHistory,
        ShowFileTree,
    ]
);

pub fn register_actions(cx: &mut App) {
    // Register global action handlers
    cx.on_action(|_: &Quit, cx| {
        cx.quit();
    });
}
```

---

## Main Application (src/app.rs)

```rust
use gpui::*;
use gpui_component::{
    button::Button,
    h_flex, v_flex,
    list::ListItem,
    resizable::{h_resizable, resizable_panel},
    tab::{Tab, TabBar},
    tree::{tree, TreeState, TreeItem},
    ActiveTheme, Icon, IconName, Root, TitleBar,
};

use crate::menu::*;
use crate::panels::{file_tree::FileTreePanel, history::HistoryPanel};
use git::Repository;

/// Which panel is currently shown in the left sidebar
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum ActivePanel {
    History,
    #[default]
    FileTree,
}

pub struct ChangeologyApp {
    /// The git repository (if opened)
    repository: Option<Repository>,

    /// Current working directory path
    cwd: Option<String>,

    /// Whether the sidebar is collapsed
    sidebar_collapsed: bool,

    /// Which panel is active
    active_panel: ActivePanel,

    /// File tree state
    file_tree_state: Entity<TreeState>,

    /// Selected file path
    selected_file: Option<String>,
}

impl ChangeologyApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Try to open repository at current directory
        let cwd = std::env::current_dir()
            .ok()
            .map(|p| p.display().to_string());
        let repository = cwd
            .as_ref()
            .and_then(|path| Repository::open(path).ok());

        // Create tree state
        let file_tree_state = cx.new(|cx| TreeState::new(cx));

        let mut app = Self {
            repository,
            cwd,
            sidebar_collapsed: false,
            active_panel: ActivePanel::FileTree,
            file_tree_state,
            selected_file: None,
        };

        // Load initial data
        app.refresh(window, cx);

        app
    }

    pub fn refresh(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(repo) = &self.repository {
            if let Ok(status) = repo.status() {
                let items = Self::build_tree_items(&status);
                self.file_tree_state.update(cx, |state, cx| {
                    state.set_items(items, cx);
                });
            }
        }
        cx.notify();
    }

    fn build_tree_items(status: &git::StatusList) -> Vec<TreeItem> {
        // Simple flat list for now - can be enhanced to show directory hierarchy
        status
            .entries
            .iter()
            .map(|entry| {
                let filename = entry
                    .path
                    .split('/')
                    .last()
                    .unwrap_or(&entry.path);
                TreeItem::new(&entry.path, filename)
            })
            .collect()
    }

    fn render_title_bar(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        TitleBar::new()
            .child(
                h_flex()
                    .gap_1()
                    .child(
                        Button::new("file-menu")
                            .ghost()
                            .label("File")
                            .dropdown_menu(|menu, _, _| {
                                menu.menu("Open Repository...", Box::new(OpenRepository))
                                    .menu("Close Repository", Box::new(CloseRepository))
                                    .separator()
                                    .menu("Refresh", Box::new(Refresh))
                                    .separator()
                                    .menu("Quit", Box::new(Quit))
                            }),
                    )
                    .child(
                        Button::new("view-menu")
                            .ghost()
                            .label("View")
                            .dropdown_menu(|menu, _, _| {
                                menu.menu("Toggle Sidebar", Box::new(ToggleSidebar))
                                    .separator()
                                    .menu("History", Box::new(ShowHistory))
                                    .menu("File Tree", Box::new(ShowFileTree))
                            }),
                    ),
            )
            .child(
                // Spacer + repo name centered
                h_flex()
                    .flex_1()
                    .justify_center()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(
                        self.cwd
                            .as_ref()
                            .and_then(|p| std::path::Path::new(p).file_name())
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| "No Repository".to_string()),
                    ),
            )
    }

    fn render_panel_tabs(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let selected = match self.active_panel {
            ActivePanel::History => 0,
            ActivePanel::FileTree => 1,
        };

        h_flex()
            .w_full()
            .px_2()
            .py_1()
            .gap_1()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(
                TabBar::new("panel-tabs")
                    .segmented()
                    .small()
                    .selected_index(selected)
                    .child(Tab::new().icon(IconName::GitCommit))
                    .child(Tab::new().icon(IconName::Folder))
                    .on_click(cx.listener(|this, index, _, cx| {
                        this.active_panel = match index {
                            0 => ActivePanel::History,
                            _ => ActivePanel::FileTree,
                        };
                        cx.notify();
                    })),
            )
    }

    fn render_file_tree(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tree_state = self.file_tree_state.clone();

        div()
            .size_full()
            .p_2()
            .child(tree(&tree_state, |ix, entry, selected, _window, cx| {
                let item = entry.item();
                let icon = if entry.is_folder() {
                    if entry.is_expanded() {
                        IconName::FolderOpen
                    } else {
                        IconName::Folder
                    }
                } else {
                    IconName::File
                };

                ListItem::new(ix)
                    .selected(selected)
                    .py_0p5()
                    .pl(px(16.) * entry.depth() as f32 + px(4.))
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                Icon::new(icon)
                                    .small()
                                    .text_color(cx.theme().muted_foreground),
                            )
                            .child(item.label.clone()),
                    )
            }))
    }

    fn render_history_panel(
        &self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        v_flex()
            .size_full()
            .p_4()
            .items_center()
            .justify_center()
            .text_color(cx.theme().muted_foreground)
            .child("Commit history coming soon...")
            .child(div().text_xs().pt_2().child("This will show recent commits"))
    }

    fn render_sidebar(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .bg(cx.theme().sidebar)
            .text_color(cx.theme().sidebar_foreground)
            .child(self.render_panel_tabs(window, cx))
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .child(match self.active_panel {
                        ActivePanel::History => {
                            self.render_history_panel(window, cx).into_any_element()
                        }
                        ActivePanel::FileTree => {
                            self.render_file_tree(window, cx).into_any_element()
                        }
                    }),
            )
    }

    fn render_content_area(
        &self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        // Placeholder for diff view
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(cx.theme().background)
            .text_color(cx.theme().muted_foreground)
            .child("Select a file to view diff")
    }
}

impl Render for ChangeologyApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(self.render_title_bar(window, cx))
            .child(
                h_resizable("main-layout")
                    .flex_1()
                    .child(
                        resizable_panel()
                            .size(px(260.))
                            .size_range(px(180.)..px(450.))
                            .child(self.render_sidebar(window, cx)),
                    )
                    .child(resizable_panel().child(self.render_content_area(window, cx))),
            )
            // Required: Render overlay layers for dialogs/notifications
            .children(Root::render_dialog_layer(window, cx))
            .children(Root::render_notification_layer(window, cx))
    }
}
```

---

## Panels Module (src/panels/mod.rs)

```rust
pub mod file_tree;
pub mod history;
```

---

## File Tree Panel (src/panels/file_tree.rs)

```rust
// Optional: Can extract file tree logic here if app.rs gets too large

use gpui::*;
use gpui_component::{
    h_flex,
    list::ListItem,
    tree::{tree, TreeItem, TreeState},
    ActiveTheme, Icon, IconName,
};
use git::{StatusKind, StatusList};

/// Get icon based on file/folder state
pub fn get_file_icon(is_folder: bool, is_expanded: bool) -> IconName {
    if is_folder {
        if is_expanded {
            IconName::FolderOpen
        } else {
            IconName::Folder
        }
    } else {
        IconName::File
    }
}

/// Get color based on git status
pub fn status_color(kind: StatusKind, cx: &App) -> Hsla {
    match kind {
        StatusKind::Added => cx.theme().green,
        StatusKind::Modified => cx.theme().yellow,
        StatusKind::Deleted => cx.theme().red,
        StatusKind::Renamed => cx.theme().blue,
        StatusKind::Untracked => cx.theme().muted_foreground,
        _ => cx.theme().foreground,
    }
}

/// Build tree items from git status (flat list)
pub fn build_flat_tree(status: &StatusList) -> Vec<TreeItem> {
    status
        .entries
        .iter()
        .map(|entry| {
            let filename = entry.path.split('/').last().unwrap_or(&entry.path);
            TreeItem::new(&entry.path, filename)
        })
        .collect()
}

/// Build tree items with directory hierarchy
pub fn build_nested_tree(status: &StatusList) -> Vec<TreeItem> {
    use std::collections::HashMap;

    let mut root_items: Vec<TreeItem> = Vec::new();
    let mut dir_map: HashMap<String, TreeItem> = HashMap::new();

    for entry in &status.entries {
        let parts: Vec<&str> = entry.path.split('/').collect();

        if parts.len() == 1 {
            // Root level file
            root_items.push(TreeItem::new(&entry.path, parts[0]));
        } else {
            // File in subdirectory - simplified: just show full path for now
            // A complete implementation would build the directory hierarchy
            let filename = parts.last().unwrap_or(&entry.path.as_str());
            root_items.push(TreeItem::new(&entry.path, *filename));
        }
    }

    root_items
}
```

---

## History Panel (src/panels/history.rs)

```rust
// Placeholder for commit history panel

use gpui::*;
use gpui_component::{v_flex, ActiveTheme};

pub struct HistoryPanel {
    // Future: commits data, list state
}

impl HistoryPanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(&self, _window: &mut Window, cx: &App) -> impl IntoElement {
        v_flex()
            .size_full()
            .p_4()
            .items_center()
            .justify_center()
            .text_color(cx.theme().muted_foreground)
            .child("Commit history coming soon...")
    }
}
```

---

## Running the Application

```bash
cd changeology
cargo run --bin changeology
```

---

## Key Patterns Summary

### 1. Initialization
```rust
gpui_component::init(cx);  // Must call before using components
```

### 2. Root Wrapper
```rust
cx.new(|cx| Root::new(view, window, cx))  // Required for overlays
```

### 3. Entity State Management
```rust
// Create entity
let tree_state = cx.new(|cx| TreeState::new(cx));

// Update entity
tree_state.update(cx, |state, cx| {
    state.set_items(items, cx);
});

// Read entity
let item = tree_state.read(cx).selected_item();
```

### 4. Event Listeners
```rust
.on_click(cx.listener(|this, event, window, cx| {
    this.handle_click(event, window, cx);
    cx.notify();  // Trigger re-render
}))
```

### 5. Conditional Rendering
```rust
.child(match self.active_panel {
    ActivePanel::History => render_history().into_any_element(),
    ActivePanel::FileTree => render_tree().into_any_element(),
})
```

---

## Next Steps After Basic Shell Works

1. **Wire up file selection** - Subscribe to tree selection changes
2. **Show diff content** - Display actual diff in content area using diff-ui crate
3. **Implement commit history** - Add `ListDelegate` for commits, extend git crate
4. **Add context menus** - Right-click on files for operations
5. **Keyboard shortcuts** - Add keybindings for common actions
6. **Improve file tree** - Show nested directories properly with expand/collapse
7. **Status indicators** - Color-code files by git status (added/modified/deleted)