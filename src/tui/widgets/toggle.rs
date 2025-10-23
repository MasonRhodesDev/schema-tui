use super::{Widget, WidgetResult};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use serde_json::Value;
use crate::tui::theme::Theme;

pub struct Toggle {
    value: bool,
    label: String,
}

impl Toggle {
    pub fn new(label: impl Into<String>, initial_value: bool) -> Self {
        Self {
            value: initial_value,
            label: label.into(),
        }
    }
    
    pub fn toggle(&mut self) {
        self.value = !self.value;
    }
}

impl Widget for Toggle {
    fn render(&self, frame: &mut Frame, area: Rect, focused: bool, theme: &Theme) {
        let style = if focused {
            Style::default().fg(theme.focused).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text)
        };
        
        let indicator = if self.value { "✓" } else { "✗" };
        let indicator_color = if self.value { theme.success } else { theme.error };
        
        let content = Line::from(vec![
            Span::styled(format!("{}: ", self.label), Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(indicator, Style::default().fg(indicator_color).add_modifier(Modifier::BOLD)),
            Span::raw(" "),
            Span::styled(self.value.to_string(), style),
        ]);
        
        let paragraph = Paragraph::new(content);
        frame.render_widget(paragraph, area);
    }
    
    fn handle_key(&mut self, key: KeyEvent) -> WidgetResult {
        match key.code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.toggle();
                WidgetResult::Confirmed(self.get_value())
            }
            _ => WidgetResult::Continue,
        }
    }
    
    fn get_value(&self) -> Value {
        Value::Bool(self.value)
    }
    
    fn set_value(&mut self, value: Value) {
        if let Some(b) = value.as_bool() {
            self.value = b;
        }
    }
    
    fn reset(&mut self) {
        // Toggle doesn't need reset logic
    }
    
    fn activate(&mut self) {
        // Toggle activates immediately - just toggle the value
        self.toggle();
    }
}
