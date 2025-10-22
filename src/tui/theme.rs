use ratatui::style::Color;

/// Theme configuration for the TUI
/// Uses terminal theme colors by default to respect user's terminal configuration
#[derive(Debug, Clone)]
pub struct Theme {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub success: Color,
    pub error: Color,
    pub warning: Color,
    pub text: Color,
    pub text_dim: Color,
    pub background: Color,
    pub border: Color,
}

impl Default for Theme {
    /// Default theme using terminal's ANSI colors
    /// This respects the user's terminal color scheme
    fn default() -> Self {
        Self::terminal()
    }
}

impl Theme {
    /// Use terminal's ANSI colors (respects user's terminal theme)
    pub fn terminal() -> Self {
        Self {
            primary: Color::Cyan,
            secondary: Color::Blue,
            accent: Color::Magenta,
            success: Color::Green,
            error: Color::Red,
            warning: Color::Yellow,
            text: Color::Reset,        // Use terminal's default foreground
            text_dim: Color::DarkGray,
            background: Color::Reset,   // Use terminal's default background
            border: Color::Gray,
        }
    }
    
    /// Dark theme with explicit colors
    pub fn dark() -> Self {
        Self {
            primary: Color::Cyan,
            secondary: Color::Blue,
            accent: Color::Magenta,
            success: Color::Green,
            error: Color::Red,
            warning: Color::Yellow,
            text: Color::White,
            text_dim: Color::DarkGray,
            background: Color::Black,
            border: Color::Gray,
        }
    }
    
    /// Light theme with explicit colors
    pub fn light() -> Self {
        Self {
            primary: Color::Blue,
            secondary: Color::Cyan,
            accent: Color::Magenta,
            success: Color::Green,
            error: Color::Red,
            warning: Color::Yellow,
            text: Color::Black,
            text_dim: Color::Gray,
            background: Color::White,
            border: Color::DarkGray,
        }
    }
}
