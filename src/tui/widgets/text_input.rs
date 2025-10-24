use super::{Widget, WidgetResult, WidgetState};
use crate::tui::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use serde_json::Value;

pub struct TextInput {
    buffer: String,
    cursor_pos: usize,
    state: WidgetState,
    label: String,
}

impl TextInput {
    pub fn new(label: impl Into<String>, initial_value: impl Into<String>) -> Self {
        let buffer = initial_value.into();
        let cursor_pos = buffer.len();

        Self {
            buffer,
            cursor_pos,
            state: WidgetState::Normal,
            label: label.into(),
        }
    }

    fn insert_char(&mut self, c: char) {
        self.buffer.insert(self.cursor_pos, c);
        self.cursor_pos += 1;
    }

    fn delete_char(&mut self) {
        if self.cursor_pos > 0 {
            self.buffer.remove(self.cursor_pos - 1);
            self.cursor_pos -= 1;
        }
    }

    fn delete_forward(&mut self) {
        if self.cursor_pos < self.buffer.len() {
            self.buffer.remove(self.cursor_pos);
        }
    }

    fn move_cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }
    }

    fn move_cursor_right(&mut self) {
        if self.cursor_pos < self.buffer.len() {
            self.cursor_pos += 1;
        }
    }

    fn get_display_text(&self) -> String {
        if self.state == WidgetState::Editing {
            let mut display = self.buffer.clone();
            display.insert(self.cursor_pos, '█');
            display
        } else {
            self.buffer.clone()
        }
    }
}

impl Widget for TextInput {
    fn render(&self, frame: &mut Frame, area: Rect, focused: bool, theme: &Theme) {
        let style = if self.state == WidgetState::Editing {
            Style::default()
                .fg(theme.popup_fg)
                .bg(theme.popup_bg)
                .add_modifier(Modifier::BOLD)
        } else if focused {
            Style::default().fg(theme.focused)
        } else {
            Style::default().fg(theme.text)
        };

        let text = self.get_display_text();
        let content = Line::from(vec![
            Span::styled(
                format!("{}: ", self.label),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(text, style),
        ]);

        let block = Block::default()
            .borders(if focused { Borders::ALL } else { Borders::NONE })
            .border_style(if self.state == WidgetState::Editing {
                Style::default().fg(theme.editing)
            } else {
                Style::default()
            })
            .style(if self.state == WidgetState::Editing {
                Style::default().bg(theme.popup_bg)
            } else {
                Style::default()
            });

        let paragraph = Paragraph::new(content).block(block);
        frame.render_widget(paragraph, area);
    }

    fn handle_key(&mut self, key: KeyEvent) -> WidgetResult {
        if self.state != WidgetState::Editing {
            return WidgetResult::Continue;
        }

        match key.code {
            KeyCode::Enter => {
                self.state = WidgetState::Normal;
                WidgetResult::Confirmed(self.get_value())
            }
            KeyCode::Esc => {
                self.state = WidgetState::Normal;
                WidgetResult::Cancelled
            }
            KeyCode::Char(c) => {
                self.insert_char(c);
                WidgetResult::Changed(self.get_value())
            }
            KeyCode::Backspace => {
                self.delete_char();
                WidgetResult::Changed(self.get_value())
            }
            KeyCode::Delete => {
                self.delete_forward();
                WidgetResult::Changed(self.get_value())
            }
            KeyCode::Left => {
                self.move_cursor_left();
                WidgetResult::Continue
            }
            KeyCode::Right => {
                self.move_cursor_right();
                WidgetResult::Continue
            }
            KeyCode::Home => {
                self.cursor_pos = 0;
                WidgetResult::Continue
            }
            KeyCode::End => {
                self.cursor_pos = self.buffer.len();
                WidgetResult::Continue
            }
            _ => WidgetResult::Continue,
        }
    }

    fn get_value(&self) -> Value {
        Value::String(self.buffer.clone())
    }

    fn set_value(&mut self, value: Value) {
        if let Some(s) = value.as_str() {
            self.buffer = s.to_string();
            self.cursor_pos = self.buffer.len();
        }
    }

    fn reset(&mut self) {
        self.state = WidgetState::Normal;
        self.cursor_pos = self.buffer.len();
    }

    fn activate(&mut self) {
        self.state = WidgetState::Editing;
        self.cursor_pos = self.buffer.len();
    }
}
