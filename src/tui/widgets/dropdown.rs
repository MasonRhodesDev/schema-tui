use super::{Widget, WidgetResult, WidgetState};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use serde_json::Value;
use crate::tui::theme::Theme;

pub struct Dropdown {
    options: Vec<String>,
    selected_index: usize,
    state: WidgetState,
    label: String,
    list_state: ListState,
}

impl Dropdown {
    pub fn new(label: impl Into<String>, options: Vec<String>, initial_value: Option<String>) -> Self {
        let selected_index = if let Some(val) = initial_value {
            options.iter().position(|o| o == &val).unwrap_or(0)
        } else {
            0
        };
        
        let mut list_state = ListState::default();
        list_state.select(Some(selected_index));
        
        Self {
            options,
            selected_index,
            state: WidgetState::Normal,
            label: label.into(),
            list_state,
        }
    }
    
    pub fn start_selecting(&mut self) {
        self.state = WidgetState::Editing;
        self.list_state.select(Some(self.selected_index));
    }
    
    fn select_next(&mut self) {
        if self.options.is_empty() {
            return;
        }
        self.selected_index = (self.selected_index + 1) % self.options.len();
        self.list_state.select(Some(self.selected_index));
    }
    
    fn select_previous(&mut self) {
        if self.options.is_empty() {
            return;
        }
        self.selected_index = if self.selected_index == 0 {
            self.options.len() - 1
        } else {
            self.selected_index - 1
        };
        self.list_state.select(Some(self.selected_index));
    }
    
    fn get_current_value(&self) -> String {
        self.options.get(self.selected_index)
            .cloned()
            .unwrap_or_default()
    }
}

impl Widget for Dropdown {
    fn render(&self, frame: &mut Frame, area: Rect, focused: bool, theme: &Theme) {
        if self.state == WidgetState::Editing {
            self.render_dropdown(frame, area, theme);
        } else {
            self.render_compact(frame, area, focused, theme);
        }
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
            KeyCode::Down | KeyCode::Char('j') => {
                self.select_next();
                WidgetResult::Continue
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.select_previous();
                WidgetResult::Continue
            }
            _ => WidgetResult::Continue,
        }
    }
    
    fn get_value(&self) -> Value {
        Value::String(self.get_current_value())
    }
    
    fn set_value(&mut self, value: Value) {
        if let Some(s) = value.as_str() {
            if let Some(idx) = self.options.iter().position(|o| o == s) {
                self.selected_index = idx;
                self.list_state.select(Some(idx));
            }
        }
    }
    
    fn reset(&mut self) {
        self.state = WidgetState::Normal;
    }
    
    fn activate(&mut self) {
        self.state = WidgetState::Editing;
        self.list_state.select(Some(self.selected_index));
    }
}

impl Dropdown {
    fn render_compact(&self, frame: &mut Frame, area: Rect, focused: bool, theme: &Theme) {
        let style = if focused {
            Style::default().fg(theme.focused).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text)
        };
        
        let current = self.get_current_value();
        let content = Line::from(vec![
            Span::styled(format!("{}: ", self.label), Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(current, style),
            Span::raw(" "),
            Span::styled("▼", Style::default().fg(theme.text_dim)),
        ]);
        
        let paragraph = Paragraph::new(content);
        frame.render_widget(paragraph, area);
    }
    
    fn render_dropdown(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let frame_size = frame.area();
        let dropdown_height = (self.options.len() + 2).min(15) as u16;
        let dropdown_width = self.options.iter()
            .map(|s| s.len())
            .max()
            .unwrap_or(20)
            .max(self.label.len() + 10) as u16 + 4;
        
        let popup_area = Rect {
            x: area.x,
            y: area.y.saturating_add(1),
            width: dropdown_width.min(frame_size.width.saturating_sub(2)),
            height: dropdown_height.min(frame_size.height.saturating_sub(area.y + 2)),
        };
        
        use ratatui::widgets::Clear;
        use ratatui::style::Color;
        frame.render_widget(Clear, popup_area);
        let bg = Block::default().style(Style::default().bg(Color::Black).fg(Color::White));
        frame.render_widget(bg, popup_area);
        
        let items: Vec<ListItem> = self.options
            .iter()
            .map(|opt| {
                ListItem::new(Line::from(opt.as_str()))
                    .style(Style::default().bg(Color::Black).fg(Color::White))
            })
            .collect();
        
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Select {} (↑↓ navigate, Enter confirm, Esc cancel)", self.label))
                    .border_style(Style::default().fg(theme.popup_border).add_modifier(Modifier::BOLD))
                    .style(Style::default().bg(Color::Black).fg(Color::White))
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().bg(Color::Black).fg(Color::White))
            .highlight_symbol("» ");
        
        frame.render_stateful_widget(list, popup_area, &mut self.list_state.clone());
    }
}
