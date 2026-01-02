# GPUI Component UI Research for Changeology

## Overview

This document researches the **vendored** gpui-component library (located at `vendor/gpui-component/`) components needed to build the Changeology git diff viewer UI with:
- **Menu bar** along the top (TitleBar with dropdown menus)
- **Left panel** with History and File Tree panels (switchable via tabs)
- **Main content area** for diff viewing
- **Panel toggle icons** at bottom of sidebar

## Reference Image Analysis

The reference shows a typical IDE/editor layout:
- Left sidebar with file tree (expandable/collapsible folders and files)
- Icons in the bottom-left for switching panel modes
- Main editor area taking remaining space
- Clean, modern dark theme appearance

---

## Vendored Library Location

```
changeology/vendor/gpui-component/
├── crates/
│   ├── ui/           # Main component library
│   ├── assets/       # Icon assets
│   ├── macros/       # Derive macros
│   └── ...
├── examples/         # Reference implementations
└── themes/           # Theme JSON files
```

The main components are in `vendor/gpui-component/crates/ui/src/`.

---

## Required Components

### 1. TitleBar (Menu Bar Area)

**Location:** `vendor/gpui-component/crates/ui/src/title_bar.rs`

**Import:**
```rust
use gpui_component::TitleBar;
```

**Key Features:**
- Platform-specific window controls (macOS traffic lights, Windows/Linux buttons)
- Custom content support via `.child()`
- Automatic window dragging support
- `TitleBar::title_bar_options()` for WindowOptions

**Usage Pattern:**
```rust
TitleBar::new()
    .child(
        h_flex()
            .gap_1()
            .child(
                Button::new("file-menu")
                    .ghost()
                    .label("File")
                    .dropdown_menu(|menu, _, _| {
                        menu.menu("New", Box::new(NewAction))
                            .menu("Open", Box::new(OpenAction))
                            .separator()
                            .menu("Quit", Box::new(QuitAction))
                    })
            )
    )
```

**Window Setup:**
```rust
let options = WindowOptions {
    titlebar: Some(TitleBar::title_bar_options()),
    ..Default::default()
};
```

---

### 2. Tree Component (File Tree)

**Location:** `vendor/gpui-component/crates/ui/src/tree.rs`

**Import:**
```rust
use gpui_component::tree::{tree, TreeState, TreeItem, TreeEntry};
```

**Key Features:**
- Hierarchical display with expand/collapse
- Keyboard navigation (up/down/left/right arrows)
- Custom item rendering via closure
- Selection state management

**TreeItem Creation:**
```rust
TreeItem::new("src/lib.rs", "lib.rs")  // id, label
    .expanded(true)
    .child(TreeItem::new("src/main.rs", "main.rs"))
    .children(vec![...])
```

**TreeState Management:**
```rust
let tree_state = cx.new(|cx| {
    TreeState::new(cx).items(vec![
        TreeItem::new("src", "src")
            .expanded(true)
            .child(TreeItem::new("src/lib.rs", "lib.rs")),
        TreeItem::new("Cargo.toml", "Cargo.toml"),
    ])
});
```

**Rendering with Custom Items:**
```rust
tree(&tree_state, |ix, entry, selected, window, cx| {
    let item = entry.item();
    let icon = if entry.is_folder() {
        if entry.is_expanded() { IconName::FolderOpen } else { IconName::Folder }
    } else {
        IconName::File
    };

    ListItem::new(ix)
        .selected(selected)
        .pl(px(16.) * entry.depth() as f32 + px(12.))
        .child(
            h_flex()
                .gap_2()
                .child(Icon::new(icon).small())
                .child(item.label.clone())
        )
})
```

**TreeEntry Methods:**
- `entry.item()` → `&TreeItem` - Get the source item
- `entry.depth()` → `usize` - Nesting depth (0 = root)
- `entry.is_folder()` → `bool` - Has children
- `entry.is_expanded()` → `bool` - Currently expanded
- `entry.is_disabled()` → `bool` - Is disabled

**TreeState Methods:**
- `state.items(vec)` - Set initial items (builder pattern)
- `state.set_items(vec, cx)` - Update items
- `state.selected_index()` → `Option<usize>`
- `state.set_selected_index(ix, cx)`
- `state.selected_item()` → `Option<&TreeItem>`
- `state.scroll_to_item(ix, strategy)`

---

### 3. Sidebar Component

**Location:** `vendor/gpui-component/crates/ui/src/sidebar/`

**Import:**
```rust
use gpui_component::sidebar::{
    Sidebar, SidebarHeader, SidebarFooter, SidebarGroup,
    SidebarMenu, SidebarMenuItem, SidebarToggleButton
};
use gpui_component::Side;
```

**Key Features:**
- Collapsible sidebar (255px default, 48px collapsed)
- Header, footer, and content groups
- Automatic scrolling for content

**Basic Structure:**
```rust
Sidebar::new(Side::Left)
    .collapsed(is_collapsed)
    .collapsible(true)
    .header(
        SidebarHeader::new()
            .child("Repository Name")
    )
    .child(
        SidebarGroup::new("Files")
            .child(SidebarMenu::new()
                .child(SidebarMenuItem::new("Changed Files")
                    .icon(IconName::Folder)))
    )
    .footer(
        SidebarFooter::new()
            .child(/* panel toggle buttons */)
    )
```

**Theme Colors:**
- `cx.theme().sidebar` - Background
- `cx.theme().sidebar_foreground` - Text color
- `cx.theme().sidebar_border` - Border color
- `cx.theme().sidebar_accent` - Hover/active background

---

### 4. TabBar Component (Panel Switching)

**Location:** `vendor/gpui-component/crates/ui/src/tab/`

**Import:**
```rust
use gpui_component::tab::{Tab, TabBar};
```

**Key Features:**
- Multiple variants: default, underline, pill, outline, **segmented**
- Icon-only tabs supported
- Size variants (xsmall, small, medium, large)

**Segmented Tabs for Panel Toggle (like reference image):**
```rust
TabBar::new("panel-tabs")
    .segmented()
    .small()
    .selected_index(selected_panel)
    .child(Tab::new().icon(IconName::GitCommit))  // History
    .child(Tab::new().icon(IconName::Folder))     // Files
    .on_click(cx.listener(|this, index, _, cx| {
        this.active_panel = match index {
            0 => PanelKind::History,
            _ => PanelKind::FileTree,
        };
        cx.notify();
    }))
```

---

### 5. PopupMenu / DropdownMenu

**Location:** `vendor/gpui-component/crates/ui/src/menu/`

**Import:**
```rust
use gpui_component::menu::{PopupMenu, PopupMenuItem};
// ContextMenuExt trait for .context_menu()
// Button has .dropdown_menu() method
```

**Dropdown Menu on Button:**
```rust
use gpui::actions;

actions!(changeology, [NewFile, OpenFile, Quit]);

Button::new("file-menu")
    .ghost()
    .label("File")
    .dropdown_menu(|menu, window, cx| {
        menu.menu_with_icon("New File", IconName::FilePlus, Box::new(NewFile))
            .menu_with_icon("Open", IconName::FolderOpen, Box::new(OpenFile))
            .separator()
            .menu("Quit", Box::new(Quit))
    })
```

**Context Menu (right-click):**
```rust
use gpui_component::menu::ContextMenuExt;

div()
    .child("Right click me")
    .context_menu(|menu, window, cx| {
        menu.menu("Copy", Box::new(Copy))
            .menu("Delete", Box::new(Delete))
    })
```

---

### 6. Resizable Panels

**Location:** `vendor/gpui-component/crates/ui/src/resizable/`

**Import:**
```rust
use gpui_component::resizable::{h_resizable, v_resizable, resizable_panel};
```

**Main Layout Structure:**
```rust
h_resizable("main-layout")
    .child(
        resizable_panel()
            .size(px(260.))
            .size_range(px(180.)..px(450.))
            .child(/* Sidebar content */)
    )
    .child(
        resizable_panel()
            .child(/* Main diff view */)
    )
```

---

### 7. List Component (for Commit History)

**Location:** `vendor/gpui-component/crates/ui/src/list/`

**Import:**
```rust
use gpui_component::list::{List, ListState, ListDelegate, ListItem, ListEvent};
use gpui_component::IndexPath;
```

**ListDelegate Pattern:**
```rust
struct CommitListDelegate {
    commits: Vec<Commit>,
    selected_index: Option<IndexPath>,
}

impl ListDelegate for CommitListDelegate {
    type Item = ListItem;

    fn items_count(&self, _section: usize, _cx: &App) -> usize {
        self.commits.len()
    }

    fn render_item(
        &mut self,
        ix: IndexPath,
        _window: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> Option<Self::Item> {
        self.commits.get(ix.row).map(|commit| {
            ListItem::new(ix)
                .child(commit.message.clone())
                .selected(Some(ix) == self.selected_index)
        })
    }

    fn set_selected_index(
        &mut self,
        ix: Option<IndexPath>,
        _window: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) {
        self.selected_index = ix;
        cx.notify();
    }
}
```

---

### 8. Button Component

**Location:** `vendor/gpui-component/crates/ui/src/button/`

**Import:**
```rust
use gpui_component::button::{Button, ButtonGroup};
```

**Key Variants:**
```rust
Button::new("id").primary().label("Primary")
Button::new("id").ghost().label("Ghost")
Button::new("id").outline().label("Outline")
Button::new("id").danger().label("Delete")
```

**Icon Button:**
```rust
Button::new("refresh")
    .ghost()
    .icon(IconName::RotateCcw)
    .small()
    .tooltip("Refresh")
    .on_click(|_, _, _| { /* handler */ })
```

**With Dropdown:**
```rust
Button::new("menu")
    .label("Options")
    .dropdown_menu(|menu, _, _| { ... })
```

---

### 9. Icons

**Location:** `vendor/gpui-component/crates/ui/src/icon.rs`
**Assets:** `vendor/gpui-component/crates/assets/`

**Import:**
```rust
use gpui_component::{Icon, IconName};
```

**Relevant Icons for Git UI:**
- `IconName::Folder`, `IconName::FolderOpen`, `IconName::File`
- `IconName::GitCommit`, `IconName::GitBranch`
- `IconName::ChevronDown`, `IconName::ChevronRight`
- `IconName::Plus`, `IconName::Minus`, `IconName::X`
- `IconName::Eye`, `IconName::EyeOff`
- `IconName::Search`, `IconName::Filter`
- `IconName::RotateCcw` (refresh)

**Usage:**
```rust
Icon::new(IconName::GitCommit)
    .small()  // or .xsmall(), .medium(), .large()
    .text_color(cx.theme().muted_foreground)
```

---

### 10. Root Component (Required!)

**Location:** `vendor/gpui-component/crates/ui/src/root.rs`

**Import:**
```rust
use gpui_component::Root;
```

**Critical:** Must wrap the application's root view:
```rust
cx.open_window(options, |window, cx| {
    let view = cx.new(|cx| MyApp::new(window, cx));
    cx.new(|cx| Root::new(view, window, cx))  // ← Required!
})
```

**Overlay Rendering in your app:**
```rust
impl Render for MyApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .child(/* main content */)
            .children(Root::render_dialog_layer(window, cx))
            .children(Root::render_notification_layer(window, cx))
    }
}
```

---

## Theme Access

**Import:**
```rust
use gpui_component::ActiveTheme;
```

**Common Theme Colors:**
```rust
cx.theme().background        // Main background
cx.theme().foreground        // Main text
cx.theme().border            // Borders
cx.theme().primary           // Primary accent
cx.theme().muted_foreground  // Secondary/muted text
cx.theme().sidebar           // Sidebar background
cx.theme().sidebar_foreground
cx.theme().sidebar_border

// Status colors
cx.theme().green    // Added
cx.theme().yellow   // Modified
cx.theme().red      // Deleted
cx.theme().blue     // Renamed
```

---

## Initialization Pattern

**From `examples/hello_world/`:**
```rust
fn main() {
    let app = Application::new();

    app.run(move |cx| {
        // REQUIRED: Initialize gpui-component
        gpui_component::init(cx);

        cx.spawn(async move |cx| {
            cx.open_window(WindowOptions::default(), |window, cx| {
                let view = cx.new(|cx| MyApp::new(window, cx));
                cx.new(|cx| Root::new(view, window, cx))
            })?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
```

---

## Proposed Layout Hierarchy

```
Root
└── ChangeologyApp
    ├── TitleBar
    │   └── h_flex (menu buttons)
    │       ├── Button "File" with dropdown_menu
    │       └── Button "View" with dropdown_menu
    └── h_resizable "main-layout"
        ├── resizable_panel (260px, 180-450px range)
        │   └── v_flex (sidebar)
        │       ├── Panel header
        │       ├── TabBar (segmented, History/Files icons)
        │       └── Panel content
        │           ├── HistoryPanel (List of commits)
        │           └── FileTreePanel (Tree of changed files)
        └── resizable_panel (flex)
            └── DiffView (content area)
```

---

## Integration with Git Crate

### Building File Tree from StatusList

```rust
use git::{StatusList, StatusKind};

fn build_tree_items(status: &StatusList) -> Vec<TreeItem> {
    status.entries.iter().map(|entry| {
        let filename = entry.path.split('/').last().unwrap_or(&entry.path);
        TreeItem::new(&entry.path, filename)
    }).collect()
}
```

### Status Colors

```rust
fn status_color(kind: StatusKind, cx: &App) -> Hsla {
    match kind {
        StatusKind::Added => cx.theme().green,
        StatusKind::Modified => cx.theme().yellow,
        StatusKind::Deleted => cx.theme().red,
        StatusKind::Renamed => cx.theme().blue,
        StatusKind::Untracked => cx.theme().muted_foreground,
        _ => cx.theme().foreground,
    }
}
```

---

## Cargo.toml Dependencies

Since gpui-component is vendored, use path dependencies:

```toml
[dependencies]
gpui = { path = "../../../vendor/zed/crates/gpui" }
gpui-component = { path = "../../../vendor/gpui-component/crates/ui" }
# Note: You may also need gpui-component-assets for icons
```

---

## Implementation Priority

1. **Phase 1: Basic Shell**
   - Set up with Root wrapper
   - TitleBar with File menu
   - h_resizable layout (sidebar + content placeholder)

2. **Phase 2: File Tree Panel**
   - TreeState with git status integration
   - File icons and indentation
   - Selection handling

3. **Phase 3: Panel Switching**
   - TabBar with segmented style
   - Switch between History/FileTree
   - Placeholder commit list

4. **Phase 4: Integration**
   - Wire file selection to diff view
   - Context menus
   - Keyboard shortcuts