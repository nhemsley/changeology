//! Theme definitions for diff display
//!
//! This module provides color definitions for rendering diffs.
//! Colors are designed to work well on both light and dark backgrounds.

use gpui::{hsla, Hsla};

/// Colors for diff display
#[derive(Debug, Clone)]
pub struct DiffTheme {
    /// Background color for the editor area
    pub editor_background: Hsla,

    /// Background color for added lines (green tint)
    pub added_line_background: Hsla,

    /// Background color for deleted lines (red tint)
    pub deleted_line_background: Hsla,

    /// Background color for modified lines (yellow/orange tint)
    pub modified_line_background: Hsla,

    /// Default text color
    pub text: Hsla,

    /// Muted text color (for less important elements)
    pub text_muted: Hsla,

    /// Border color
    pub border: Hsla,
}

impl DiffTheme {
    /// Create a dark theme (default)
    pub fn dark() -> Self {
        Self {
            // Dark editor background
            editor_background: hsla(0.0, 0.0, 0.12, 1.0), // #1e1e1e equivalent

            // Green with low opacity for added lines
            added_line_background: hsla(120.0 / 360.0, 0.4, 0.25, 0.20),

            // Red with low opacity for deleted lines
            deleted_line_background: hsla(0.0 / 360.0, 0.5, 0.30, 0.20),

            // Yellow/orange with low opacity for modified lines
            modified_line_background: hsla(45.0 / 360.0, 0.5, 0.30, 0.20),

            // Light text for dark background
            text: hsla(0.0, 0.0, 0.85, 1.0), // #d9d9d9 equivalent

            // Dimmer text
            text_muted: hsla(0.0, 0.0, 0.5, 1.0), // #808080 equivalent

            // Subtle border
            border: hsla(0.0, 0.0, 0.25, 1.0),
        }
    }

    /// Create a light theme
    pub fn light() -> Self {
        Self {
            // Light editor background
            editor_background: hsla(0.0, 0.0, 0.98, 1.0), // #fafafa equivalent

            // Green with slightly higher opacity for light mode
            added_line_background: hsla(120.0 / 360.0, 0.5, 0.45, 0.18),

            // Red with slightly higher opacity for light mode
            deleted_line_background: hsla(0.0 / 360.0, 0.6, 0.50, 0.18),

            // Yellow/orange for light mode
            modified_line_background: hsla(45.0 / 360.0, 0.6, 0.50, 0.18),

            // Dark text for light background
            text: hsla(0.0, 0.0, 0.15, 1.0), // #262626 equivalent

            // Dimmer text
            text_muted: hsla(0.0, 0.0, 0.45, 1.0), // #737373 equivalent

            // Subtle border
            border: hsla(0.0, 0.0, 0.80, 1.0),
        }
    }
}

impl Default for DiffTheme {
    fn default() -> Self {
        Self::dark()
    }
}

/// Solid colors for diff indicators (gutter bars, etc.)
/// These are more saturated than the line backgrounds
pub struct DiffIndicatorColors;

impl DiffIndicatorColors {
    /// Solid green for added indicators
    pub fn added() -> Hsla {
        hsla(120.0 / 360.0, 0.55, 0.45, 1.0)
    }

    /// Solid red for deleted indicators
    pub fn deleted() -> Hsla {
        hsla(0.0 / 360.0, 0.65, 0.50, 1.0)
    }

    /// Solid yellow/orange for modified indicators
    pub fn modified() -> Hsla {
        hsla(45.0 / 360.0, 0.70, 0.50, 1.0)
    }
}
