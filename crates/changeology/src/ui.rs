//! Minimal UI primitives to replace gpui-component
//!
//! This module provides basic UI building blocks that work with
//! the local gpui without external dependencies.

use gpui::*;

// ============================================================================
// Layout helpers
// ============================================================================

/// Create a horizontal flex container
pub fn h_flex() -> Div {
    div().flex().flex_row().items_center()
}

/// Create a vertical flex container
pub fn v_flex() -> Div {
    div().flex().flex_col()
}

// ============================================================================
// Theme colors (simple dark theme)
// ============================================================================

pub struct Theme {
    pub background: Rgba,
    pub foreground: Rgba,
    pub muted: Rgba,
    pub muted_foreground: Rgba,
    pub border: Rgba,
    pub sidebar: Rgba,
    pub sidebar_foreground: Rgba,
    pub accent: Rgba,
    pub destructive: Rgba,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: rgb(0x1e1e1e),
            foreground: rgb(0xd4d4d4),
            muted: rgb(0x2d2d2d),
            muted_foreground: rgb(0x808080),
            border: rgb(0x3c3c3c),
            sidebar: rgb(0x252526),
            sidebar_foreground: rgb(0xcccccc),
            accent: rgb(0x0078d4),
            destructive: rgb(0xf14c4c),
        }
    }
}

/// Extension trait to get theme from context
pub trait ActiveTheme {
    fn theme(&self) -> Theme;
}

impl ActiveTheme for App {
    fn theme(&self) -> Theme {
        Theme::default()
    }
}

// ============================================================================
// Icons (simple text-based icons)
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IconName {
    File,
    Folder,
    FolderOpen,
    ChevronRight,
    ChevronDown,
    Inbox,
    GitCommit,
    Copy,
    Check,
    Plus,
    Minus,
    X,
    Menu,
    Settings,
    Refresh,
}

impl IconName {
    pub fn as_str(&self) -> &'static str {
        match self {
            IconName::File => "📄",
            IconName::Folder => "📁",
            IconName::FolderOpen => "📂",
            IconName::ChevronRight => "▶",
            IconName::ChevronDown => "▼",
            IconName::Inbox => "📥",
            IconName::GitCommit => "●",
            IconName::Copy => "📋",
            IconName::Check => "✓",
            IconName::Plus => "+",
            IconName::Minus => "−",
            IconName::X => "✕",
            IconName::Menu => "☰",
            IconName::Settings => "⚙",
            IconName::Refresh => "↻",
        }
    }
}

pub struct Icon {
    name: IconName,
    size: Pixels,
    color: Option<Rgba>,
}

impl Icon {
    pub fn new(name: IconName) -> Self {
        Self {
            name,
            size: px(16.0),
            color: None,
        }
    }

    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }

    pub fn small(mut self) -> Self {
        self.size = px(14.0);
        self
    }

    pub fn text_color(mut self, color: Rgba) -> Self {
        self.color = Some(color);
        self
    }
}

impl RenderOnce for Icon {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let mut el = div()
            .text_size(self.size)
            .child(self.name.as_str());
        
        if let Some(color) = self.color {
            el = el.text_color(color);
        }
        
        el
    }
}

impl IntoElement for Icon {
    type Element = <Self as RenderOnce>::Element;
    
    fn into_element(self) -> Self::Element {
        self.into_any_element().into_element()
    }
}

// ============================================================================
// Button
// ============================================================================

pub struct Button {
    id: ElementId,
    label: Option<SharedString>,
    icon: Option<IconName>,
    variant: ButtonVariant,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

#[derive(Clone, Copy, Default)]
pub enum ButtonVariant {
    #[default]
    Default,
    Ghost,
    Destructive,
}

impl Button {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            icon: None,
            variant: ButtonVariant::Default,
            on_click: None,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn ghost(mut self) -> Self {
        self.variant = ButtonVariant::Ghost;
        self
    }

    pub fn destructive(mut self) -> Self {
        self.variant = ButtonVariant::Destructive;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Button {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        
        let (bg, fg, hover_bg) = match self.variant {
            ButtonVariant::Default => (theme.muted, theme.foreground, theme.border),
            ButtonVariant::Ghost => (Rgba::transparent_black(), theme.foreground, theme.muted),
            ButtonVariant::Destructive => (theme.destructive, rgb(0xffffff), rgb(0xd32f2f)),
        };

        let mut el = div()
            .id(self.id)
            .px_3()
            .py_1()
            .rounded(px(4.0