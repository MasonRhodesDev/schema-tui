use super::{Widget, WidgetResult, WidgetState};
use crate::tui::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use serde_json::Value;

pub struct SearchableDropdown {
    all_options: Vec<String>,
    filtered_options: Vec<String>,
    selected_index: usize,
    search_buffer: String,
    state: WidgetState,
    label: String,
    list_state: ListState,
    current_value: String,
}

impl SearchableDropdown {
    pub fn new(
        label: impl Into<String>,
        options: Vec<String>,
        initial_value: Option<String>,
    ) -> Self {
        let current_value =
            initial_value.unwrap_or_else(|| options.first().cloned().unwrap_or_default());

        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            all_options: options.clone(),
            filtered_options: options,
            selected_index: 0,
            search_buffer: String::new(),
            state: WidgetState::Normal,
            label: label.into(),
            list_state,
            current_value,
        }
    }

    pub fn start_selecting(&mut self) {
        self.state = WidgetState::Editing;
        self.search_buffer.clear();
        self.filtered_options = self.all_options.clone();
        self.selected_index = 0;
        self.list_state.select(Some(0));
    }

    fn update_filter(&mut self) {
        let search_lower = self.search_buffer.to_lowercase();
        self.filtered_options = self
            .all_options
            .iter()
            .filter(|opt| opt.to_lowercase().contains(&search_lower))
            .cloned()
            .collect();

        if self.filtered_options.is_empty() {
            self.selected_index = 0;
        } else {
            self.selected_index = self.selected_index.min(self.filtered_options.len() - 1);
        }
        self.list_state.select(Some(self.selected_index));
    }

    fn select_next(&mut self) {
        if self.filtered_options.is_empty() {
            return;
        }
        self.selected_index = (self.selected_index + 1) % self.filtered_options.len();
        self.list_state.select(Some(self.selected_index));
    }

    fn select_previous(&mut self) {
        if self.filtered_options.is_empty() {
            return;
        }
        self.selected_index = if self.selected_index == 0 {
            self.filtered_options.len() - 1
        } else {
            self.selected_index - 1
        };
        self.list_state.select(Some(self.selected_index));
    }
}

impl Widget for SearchableDropdown {
    fn render(&self, frame: &mut Frame, area: Rect, focused: bool, theme: &Theme) {
        if self.state == WidgetState::Editing {
            self.render_searchable(frame, area, theme);
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
                if !self.filtered_options.is_empty() {
                    self.current_value = self.filtered_options[self.selected_index].clone();
                    self.state = WidgetState::Normal;
                    WidgetResult::Confirmed(self.get_value())
                } else {
                    WidgetResult::Continue
                }
            }
            KeyCode::Esc => {
                self.state = WidgetState::Normal;
                WidgetResult::Cancelled
            }
            KeyCode::Down | KeyCode::Char('j')
                if !key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                self.select_next();
                WidgetResult::Continue
            }
            KeyCode::Up | KeyCode::Char('k')
                if !key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                self.select_previous();
                WidgetResult::Continue
            }
            KeyCode::Char(c) => {
                self.search_buffer.push(c);
                self.update_filter();
                WidgetResult::Continue
            }
            KeyCode::Backspace => {
                self.search_buffer.pop();
                self.update_filter();
                WidgetResult::Continue
            }
            _ => WidgetResult::Continue,
        }
    }

    fn get_value(&self) -> Value {
        Value::String(self.current_value.clone())
    }

    fn set_value(&mut self, value: Value) {
        if let Some(s) = value.as_str() {
            self.current_value = s.to_string();
        }
    }

    fn reset(&mut self) {
        self.state = WidgetState::Normal;
        self.search_buffer.clear();
    }

    fn activate(&mut self) {
        self.state = WidgetState::Editing;
        self.search_buffer.clear();
        self.filtered_options = self.all_options.clone();
        self.selected_index = 0;
        self.list_state.select(Some(0));
    }
}

impl SearchableDropdown {
    fn render_compact(&self, frame: &mut Frame, area: Rect, focused: bool, theme: &Theme) {
        let style = if focused {
            Style::default()
                .fg(theme.focused)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text)
        };

        let content = Line::from(vec![
            Span::styled(
                format!("{}: ", self.label),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(&self.current_value, style),
            Span::raw(" "),
            Span::styled("🔍", Style::default().fg(theme.text_dim)),
        ]);

        let paragraph = Paragraph::new(content);
        frame.render_widget(paragraph, area);
    }

    fn render_searchable(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let frame_size = frame.area();
        let dropdown_height = (self.filtered_options.len() + 2).min(15) as u16;
        let dropdown_width = self
            .filtered_options
            .iter()
            .map(|s| s.len())
            .max()
            .unwrap_or(40)
            .max(self.label.len() + 30)
            .max(60) as u16
            + 4;

        let popup_area = Rect {
            x: area.x,
            y: area.y.saturating_add(1),
            width: dropdown_width.min(frame_size.width.saturating_sub(2)),
            height: dropdown_height.min(frame_size.height.saturating_sub(area.y + 2)),
        };

        use ratatui::style::Color;
        use ratatui::widgets::Clear;
        frame.render_widget(Clear, popup_area);
        let bg = Block::default().style(Style::default().bg(Color::Black).fg(Color::White));
        frame.render_widget(bg, popup_area);

        let items: Vec<ListItem> = self
            .filtered_options
            .iter()
            .map(|opt| {
                ListItem::new(Line::from(opt.as_str()))
                    .style(Style::default().bg(Color::Black).fg(Color::White))
            })
            .collect();

        let title = if self.search_buffer.is_empty() {
            format!("Search {}: (type to filter)", self.label)
        } else {
            format!(
                "Search {}: \"{}\" ({} results)",
                self.label,
                self.search_buffer,
                self.filtered_options.len()
            )
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(
                        Style::default()
                            .fg(theme.popup_border)
                            .add_modifier(Modifier::BOLD),
                    )
                    .style(Style::default().bg(Color::Black).fg(Color::White)),
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
