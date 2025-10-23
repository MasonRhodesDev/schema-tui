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
    pub highlight_bg: Color,
    pub highlight_fg: Color,
    pub focused: Color,
    pub editing: Color,
    pub popup_bg: Color,
    pub popup_fg: Color,
    pub popup_border: Color,
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
            text: Color::Reset,
            text_dim: Color::DarkGray,
            background: Color::Reset,
            border: Color::Gray,
            highlight_bg: Color::Reset,
            highlight_fg: Color::Cyan,
            focused: Color::Yellow,
            editing: Color::Cyan,
            popup_bg: Color::Reset,
            popup_fg: Color::Reset,
            popup_border: Color::Cyan,
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
            highlight_bg: Color::DarkGray,
            highlight_fg: Color::White,
            focused: Color::Yellow,
            editing: Color::Cyan,
            popup_bg: Color::Black,
            popup_fg: Color::White,
            popup_border: Color::Cyan,
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
            highlight_bg: Color::Gray,
            highlight_fg: Color::Black,
            focused: Color::Blue,
            editing: Color::Cyan,
            popup_bg: Color::White,
            popup_fg: Color::Black,
            popup_border: Color::Blue,
        }
    }
}
