use crate::tui::theme::Theme;
use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};
use serde_json::Value;

/// Result of handling a key event in a widget
#[derive(Debug, Clone)]
pub enum WidgetResult {
    /// Continue editing, no value change
    Continue,
    /// Value was changed, here's the new value
    Changed(Value),
    /// User confirmed the edit (Enter pressed)
    Confirmed(Value),
    /// User cancelled the edit (Esc pressed)
    Cancelled,
}

/// Base trait for all interactive widgets
pub trait Widget {
    /// Render the widget to the frame
    fn render(&self, frame: &mut Frame, area: Rect, focused: bool, theme: &Theme);

    /// Handle a key event, returning the result
    fn handle_key(&mut self, key: KeyEvent) -> WidgetResult;

    /// Get the current value
    fn get_value(&self) -> Value;

    /// Set the value (used when loading from config)
    fn set_value(&mut self, value: Value);

    /// Reset to initial state
    fn reset(&mut self);

    /// Activate the widget for editing (transition to Editing state)
    fn activate(&mut self);
}

/// Widget state for tracking focus and editing
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WidgetState {
    Normal,
    Focused,
    Editing,
}
