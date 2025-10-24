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

pub struct FloatInput {
    buffer: String,
    cursor_pos: usize,
    state: WidgetState,
    label: String,
    min: Option<f64>,
    max: Option<f64>,
    step: Option<f64>,
}

impl FloatInput {
    pub fn new(
        label: impl Into<String>,
        initial_value: f64,
        min: Option<f64>,
        max: Option<f64>,
        step: Option<f64>,
    ) -> Self {
        let buffer = initial_value.to_string();
        let cursor_pos = buffer.len();

        Self {
            buffer,
            cursor_pos,
            state: WidgetState::Normal,
            label: label.into(),
            min,
            max,
            step,
        }
    }

    fn insert_char(&mut self, c: char) {
        if c.is_ascii_digit()
            || (c == '-' && self.cursor_pos == 0)
            || (c == '.' && !self.buffer.contains('.'))
        {
            self.buffer.insert(self.cursor_pos, c);
            self.cursor_pos += 1;
        }
    }

    fn delete_char(&mut self) {
        if self.cursor_pos > 0 {
            self.buffer.remove(self.cursor_pos - 1);
            self.cursor_pos -= 1;
        }
    }

    fn validate(&self) -> Option<f64> {
        let num = self.buffer.parse::<f64>().ok()?;

        if let Some(min) = self.min {
            if num < min {
                return None;
            }
        }

        if let Some(max) = self.max {
            if num > max {
                return None;
            }
        }

        Some(num)
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

impl Widget for FloatInput {
    fn render(&self, frame: &mut Frame, area: Rect, focused: bool, theme: &Theme) {
        let is_valid = self.validate().is_some();

        let style = if self.state == WidgetState::Editing {
            if is_valid {
                Style::default()
                    .fg(theme.popup_fg)
                    .bg(theme.popup_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(theme.error)
                    .bg(theme.popup_bg)
                    .add_modifier(Modifier::BOLD)
            }
        } else if focused {
            Style::default().fg(theme.focused)
        } else {
            Style::default().fg(theme.text)
        };

        let text = self.get_display_text();
        let mut spans = vec![
            Span::styled(
                format!("{}: ", self.label),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(text, style),
        ];

        if self.state == WidgetState::Editing && !is_valid {
            spans.push(Span::styled(
                " ✗",
                Style::default().fg(theme.error).bg(theme.popup_bg),
            ));
        }

        let content = Line::from(spans);

        let block = Block::default()
            .borders(if focused { Borders::ALL } else { Borders::NONE })
            .border_style(if self.state == WidgetState::Editing {
                if is_valid {
                    Style::default().fg(theme.editing)
                } else {
                    Style::default().fg(theme.error)
                }
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
                if let Some(num) = self.validate() {
                    self.state = WidgetState::Normal;
                    WidgetResult::Confirmed(Value::from(num))
                } else {
                    WidgetResult::Continue
                }
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
            KeyCode::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
                WidgetResult::Continue
            }
            KeyCode::Right => {
                if self.cursor_pos < self.buffer.len() {
                    self.cursor_pos += 1;
                }
                WidgetResult::Continue
            }
            KeyCode::Up => {
                if let (Some(current), Some(step)) = (self.validate(), self.step) {
                    let new_val = current + step;
                    if self.max.is_none_or(|max| new_val <= max) {
                        self.buffer = new_val.to_string();
                        self.cursor_pos = self.buffer.len();
                        return WidgetResult::Changed(self.get_value());
                    }
                }
                WidgetResult::Continue
            }
            KeyCode::Down => {
                if let (Some(current), Some(step)) = (self.validate(), self.step) {
                    let new_val = current - step;
                    if self.min.is_none_or(|min| new_val >= min) {
                        self.buffer = new_val.to_string();
                        self.cursor_pos = self.buffer.len();
                        return WidgetResult::Changed(self.get_value());
                    }
                }
                WidgetResult::Continue
            }
            _ => WidgetResult::Continue,
        }
    }

    fn get_value(&self) -> Value {
        if let Some(num) = self.validate() {
            Value::from(num)
        } else {
            Value::String(self.buffer.clone())
        }
    }

    fn set_value(&mut self, value: Value) {
        if let Some(n) = value.as_f64() {
            self.buffer = n.to_string();
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
