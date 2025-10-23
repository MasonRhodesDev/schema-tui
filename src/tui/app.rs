use crate::schema::{ConfigSchema, SchemaField, SchemaSection, FieldType, UIWidget};
use crate::options::OptionResolver;
use super::widgets::*;
use super::theme::Theme;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs},
    Frame, Terminal,
};
use serde_json::Value;
use std::collections::HashMap;
use std::io;
use anyhow::Result;

type ChangeHandler = Box<dyn Fn(&str, &Value) + Send>;

pub struct SchemaTUI {
    // Core data
    schema: ConfigSchema,
    values: HashMap<String, Value>,
    config_path: Option<std::path::PathBuf>,
    
    // UI state
    current_section: usize,
    current_field: usize,
    list_state: ListState,
    
    // Active editing
    edit_mode: bool,
    active_field: Option<String>,
    active_widgets: HashMap<String, Box<dyn Widget>>,
    
    // Event system
    change_handlers: Vec<ChangeHandler>,
    
    // Options
    option_resolver: OptionResolver,
    
    // Theme
    theme: Theme,
    
    // Status
    message: Option<String>,
    should_quit: bool,
}

impl SchemaTUI {
    pub fn new(
        schema: ConfigSchema,
        initial_values: HashMap<String, Value>,
        option_resolver: OptionResolver,
        theme: Theme,
        config_path: Option<std::path::PathBuf>,
    ) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        
        // Merge defaults from schema with initial values
        let mut values = initial_values;
        for section in &schema.sections {
            for field in &section.fields {
                let field_key = format!("{}.{}", section.id, field.id);
                
                // Only set default if value not already present
                if !values.contains_key(&field_key) {
                    if let Some(default_value) = Self::get_field_default(&field.field_type) {
                        values.insert(field_key, default_value);
                    }
                }
            }
        }
        
        Self {
            schema,
            values,
            config_path,
            current_section: 0,
            current_field: 0,
            list_state,
            edit_mode: false,
            active_field: None,
            active_widgets: HashMap::new(),
            change_handlers: Vec::new(),
            option_resolver,
            theme,
            message: None,
            should_quit: false,
        }
    }
    
    fn get_field_default(field_type: &FieldType) -> Option<Value> {
        match field_type {
            FieldType::String { default, .. } | FieldType::Path { default, .. } => {
                default.as_ref().map(|s| Value::String(s.clone()))
            }
            FieldType::Boolean { default } => Some(Value::Bool(*default)),
            FieldType::Number { default, .. } => {
                default.map(|n| Value::Number(n.into()))
            }
            FieldType::Float { default, .. } => {
                default.map(|f| Value::from(f))
            }
            FieldType::Enum { default, .. } => {
                default.as_ref().map(|s| Value::String(s.clone()))
            }
        }
    }
    
    pub fn on_change<F>(&mut self, handler: F)
    where
        F: Fn(&str, &Value) + Send + 'static,
    {
        self.change_handlers.push(Box::new(handler));
    }
    
    pub fn get_value(&self, key: &str) -> Option<&Value> {
        self.values.get(key)
    }
    
    pub fn get_all_values(&self) -> &HashMap<String, Value> {
        &self.values
    }
    
    pub fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        
        let result = self.run_loop(&mut terminal);
        
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;
        
        result
    }
    
    fn run_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        loop {
            terminal.draw(|f| self.render(f))?;
            
            if self.should_quit {
                break;
            }
            
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key(key)?;
                }
            }
        }
        
        Ok(())
    }
    
    fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        if self.edit_mode {
            self.handle_edit_mode(key)?;
        } else {
            self.handle_navigation_mode(key)?;
        }
        
        Ok(())
    }
    
    fn handle_navigation_mode(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => {
                self.next_section();
            }
            KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => {
                self.previous_section();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.next_field();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.previous_field();
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.activate_current_field()?;
            }
            KeyCode::Char('e') => {
                // Check if current field has external editor action
                if let Some(field) = self.get_current_field() {
                    if matches!(field.field_type, crate::schema::FieldType::Path { .. }) {
                        self.execute_external_editor_for_field()?;
                    }
                }
            }
            _ => {}
        }
        
        Ok(())
    }
    
    fn handle_edit_mode(&mut self, key: KeyEvent) -> Result<()> {
        if let Some(field_key) = &self.active_field.clone() {
            if let Some(widget) = self.active_widgets.get_mut(field_key) {
                match widget.handle_key(key) {
                    WidgetResult::Confirmed(value) => {
                        self.fire_change(field_key, value);
                        self.edit_mode = false;
                        self.active_field = None;
                        // Remove widget from cache so it rebuilds with fresh value next time
                        self.active_widgets.remove(field_key);
                        self.message = Some(format!("Saved {}", field_key));
                    }
                    WidgetResult::Cancelled => {
                        self.edit_mode = false;
                        self.active_field = None;
                        self.message = Some("Cancelled".to_string());
                    }
                    WidgetResult::Changed(value) => {
                        // Live update
                        self.fire_change(field_key, value);
                    }
                    WidgetResult::Continue => {}
                }
            }
        }
        
        Ok(())
    }
    
    fn get_visible_sections(&self) -> Vec<(usize, &crate::schema::SchemaSection)> {
        self.schema.sections.iter().enumerate()
            .filter(|(_, section)| {
                if let Some(condition) = &section.visible_when {
                    super::conditions::evaluate_condition(condition, &self.values)
                } else {
                    true
                }
            })
            .collect()
    }
    
    fn next_section(&mut self) {
        let visible = self.get_visible_sections();
        if visible.is_empty() {
            return;
        }
        
        // Find current in visible list and go to next
        let current_pos = visible.iter().position(|(idx, _)| *idx == self.current_section);
        if let Some(pos) = current_pos {
            let next_pos = (pos + 1) % visible.len();
            self.current_section = visible[next_pos].0;
        } else {
            // Current not visible, go to first visible
            self.current_section = visible[0].0;
        }
        
        self.current_field = 0;
        self.list_state.select(Some(0));
    }
    
    fn previous_section(&mut self) {
        let visible = self.get_visible_sections();
        if visible.is_empty() {
            return;
        }
        
        // Find current in visible list and go to previous
        let current_pos = visible.iter().position(|(idx, _)| *idx == self.current_section);
        if let Some(pos) = current_pos {
            let prev_pos = if pos == 0 { visible.len() - 1 } else { pos - 1 };
            self.current_section = visible[prev_pos].0;
        } else {
            // Current not visible, go to first visible
            self.current_section = visible[0].0;
        }
        
        self.current_field = 0;
        self.list_state.select(Some(0));
    }
    
    fn next_field(&mut self) {
        let field_count = self.get_current_section().map(|s| s.fields.len()).unwrap_or(0);
        if field_count > 0 {
            self.current_field = (self.current_field + 1) % field_count;
            self.list_state.select(Some(self.current_field));
        }
    }
    
    fn previous_field(&mut self) {
        let field_count = self.get_current_section().map(|s| s.fields.len()).unwrap_or(0);
        if field_count > 0 {
            self.current_field = if self.current_field == 0 {
                field_count - 1
            } else {
                self.current_field - 1
            };
            self.list_state.select(Some(self.current_field));
        }
    }
    
    fn activate_current_field(&mut self) -> Result<()> {
        let field_key = self.get_current_field_key();
        let widget_type = self.get_current_field().map(|f| f.ui_widget.clone());
        
        if let Some(wt) = widget_type {
            // Create widget if not exists
            if !self.active_widgets.contains_key(&field_key) {
                let field = self.get_current_field().unwrap();
                let widget = self.build_widget_for_field(field)?;
                self.active_widgets.insert(field_key.clone(), widget);
            }
            
            // Activate widget
            if let Some(widget) = self.active_widgets.get_mut(&field_key) {
                // Set current value
                if let Some(value) = self.values.get(&field_key) {
                    widget.set_value(value.clone());
                }
                
                // Activate the widget (transitions to editing state)
                widget.activate();
                
                // Enter edit mode based on widget type
                match &wt {
                    UIWidget::TextInput | UIWidget::NumberInput | UIWidget::Dropdown | UIWidget::DropdownSearchable => {
                        self.edit_mode = true;
                        self.active_field = Some(field_key.clone());
                    }
                    UIWidget::Toggle => {
                        // Toggle activates immediately - get value will be handled below
                    }
                    _ => {}
                }
            }
            
            // Handle toggle fire change outside of the widget borrow
            if matches!(wt, UIWidget::Toggle) {
                if let Some(widget) = self.active_widgets.get(&field_key) {
                    self.fire_change(&field_key, widget.get_value());
                }
            }
        }
        
        Ok(())
    }
    
    fn fire_change(&mut self, key: &str, value: Value) {
        self.values.insert(key.to_string(), value.clone());
        
        // Auto-save to disk if config_path is set
        if let Err(e) = self.save_config() {
            eprintln!("Failed to save config: {}", e);
        }
        
        for handler in &self.change_handlers {
            handler(key, &value);
        }
    }
    
    fn save_config(&self) -> Result<()> {
        if let Some(ref path) = self.config_path {
            use crate::config::{ConfigStore, ConfigSaver};
            
            // Convert flat map ("general.wallpaper") to nested structure
            let mut store = ConfigStore::new();
            for (key, value) in &self.values {
                store.set_nested(key, value.clone());
            }
            
            ConfigSaver::save_toml(&store, &self.schema, path)?;
        }
        Ok(())
    }
    
    fn execute_external_editor_for_field(&mut self) -> Result<()> {
        let field_key = self.get_current_field_key();
        let current_value = self.values.get(&field_key)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        // Get editor from env or use default
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
        
        // Determine file extension based on field type
        let extension = if let Some(field) = self.get_current_field() {
            match &field.field_type {
                crate::schema::FieldType::Path { file_type, .. } => {
                    match file_type {
                        Some(crate::schema::FileTypeFilter::Json) => "json",
                        Some(crate::schema::FileTypeFilter::Image) => "png",
                        _ => "txt",
                    }
                }
                _ => "txt",
            }
        } else {
            "txt"
        };
        
        let action = super::actions::FieldAction::ExternalEditor {
            editor,
            extension: extension.to_string(),
        };
        
        if let Ok(Some(new_value)) = action.execute(&current_value) {
            self.fire_change(&field_key, Value::String(new_value.trim().to_string()));
            self.message = Some(format!("Updated {} from external editor", field_key));
        } else {
            self.message = Some("External editor cancelled or no changes".to_string());
        }
        
        Ok(())
    }
    
    fn get_current_section(&self) -> Option<&SchemaSection> {
        self.schema.sections.get(self.current_section)
    }
    
    fn get_current_field(&self) -> Option<&SchemaField> {
        self.get_current_section()
            .and_then(|s| s.fields.get(self.current_field))
    }
    
    fn get_current_field_key(&self) -> String {
        if let Some(section) = self.get_current_section() {
            if let Some(field) = self.get_current_field() {
                return format!("{}.{}", section.id, field.id);
            }
        }
        String::new()
    }
    
    fn build_widget_for_field(&self, field: &SchemaField) -> Result<Box<dyn Widget>> {
        let field_key = format!("{}.{}", 
            self.schema.sections[self.current_section].id, 
            field.id
        );
        
        let widget: Box<dyn Widget> = match &field.field_type {
            FieldType::String { default, .. } => {
                let initial = self.values.get(&field_key)
                    .and_then(|v| v.as_str())
                    .or(default.as_deref())
                    .unwrap_or("");
                Box::new(TextInput::new(&field.label, initial))
            }
            
            FieldType::Boolean { default } => {
                let initial = self.values.get(&field_key)
                    .and_then(|v| v.as_bool())
                    .unwrap_or(*default);
                Box::new(Toggle::new(&field.label, initial))
            }
            
            FieldType::Number { default, min, max } => {
                let initial = self.values.get(&field_key)
                    .and_then(|v| v.as_i64())
                    .or(*default)
                    .unwrap_or(0);
                Box::new(NumberInput::new(&field.label, initial, *min, *max))
            }
            
            FieldType::Float { default, min, max, step } => {
                let initial = self.values.get(&field_key)
                    .and_then(|v| v.as_f64())
                    .or(*default)
                    .unwrap_or(0.0);
                Box::new(FloatInput::new(&field.label, initial, *min, *max, *step))
            }
            
            FieldType::Enum { options_source, default } => {
                let options = match options_source {
                    crate::schema::OptionSource::Static { values } => values.clone(),
                    crate::schema::OptionSource::Function { name } => {
                        match self.option_resolver.resolve_from_provider(name) {
                            Ok(opts) => opts,
                            Err(_) => vec![]
                        }
                    }
                    crate::schema::OptionSource::Provider { provider } => {
                        match self.option_resolver.resolve_from_provider(provider) {
                            Ok(opts) => opts,
                            Err(_) => vec![]
                        }
                    }
                    crate::schema::OptionSource::Script { command, .. } => {
                        match self.option_resolver.resolve_from_script_sync(command) {
                            Ok(opts) => opts,
                            Err(_) => vec![]
                        }
                    }
                    crate::schema::OptionSource::FileList { directory, pattern, extract } => {
                        match self.option_resolver.resolve_from_file_list(directory, pattern, extract.as_deref()) {
                            Ok(opts) => opts,
                            Err(_) => vec![]
                        }
                    }
                };
                
                let initial = self.values.get(&field_key)
                    .and_then(|v| v.as_str().map(String::from))
                    .or_else(|| default.clone());
                
                // Use searchable dropdown if provider-based (usually many options)
                match &field.ui_widget {
                    UIWidget::DropdownSearchable => {
                        Box::new(SearchableDropdown::new(&field.label, options, initial))
                    }
                    _ => {
                        Box::new(Dropdown::new(&field.label, options, initial))
                    }
                }
            }
            
            FieldType::Path { default, .. } => {
                let initial = self.values.get(&field_key)
                    .and_then(|v| v.as_str())
                    .or(default.as_deref())
                    .unwrap_or("");
                Box::new(TextInput::new(&field.label, initial))
            }
        };
        
        Ok(widget)
    }
    
    fn render(&mut self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(4),
            ])
            .split(frame.area());
        
        self.render_header(frame, chunks[0]);
        self.render_tabs(frame, chunks[1]);
        self.render_content(frame, chunks[2]);
        self.render_footer(frame, chunks[3]);
    }
    
    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let title = self.schema.title.as_deref().unwrap_or("Configuration");
        let header = Paragraph::new(vec![
            Line::from(vec![
                Span::styled(title, Style::default().add_modifier(Modifier::BOLD).fg(self.theme.primary)),
            ]),
            Line::from(vec![
                Span::raw(self.schema.description.as_deref().unwrap_or("")),
            ]),
        ])
        .block(Block::default().borders(Borders::ALL));
        
        frame.render_widget(header, area);
    }
    
    fn render_tabs(&self, frame: &mut Frame, area: Rect) {
        let visible = self.get_visible_sections();
        
        let all_titles: Vec<String> = visible.iter()
            .map(|(_, s)| {
                if let Some(icon) = &s.icon {
                    format!("{} {}", icon, s.title)
                } else {
                    s.title.clone()
                }
            })
            .collect();
        
        if all_titles.is_empty() {
            return;
        }
        
        // Find position of current section in visible list
        let selected = visible.iter().position(|(idx, _)| *idx == self.current_section).unwrap_or(0);
        
        // Calculate available width for tabs (minus borders and padding)
        let available_width = area.width.saturating_sub(4) as usize;
        
        // Calculate tab widths (title + spacing)
        let tab_widths: Vec<usize> = all_titles.iter()
            .map(|t| t.chars().count() + 3)
            .collect();
        
        let total_width: usize = tab_widths.iter().sum();
        
        // If all tabs fit, show them all
        if total_width <= available_width {
            let tabs = Tabs::new(all_titles)
                .block(Block::default().borders(Borders::ALL).title("Sections"))
                .select(selected)
                .style(Style::default().fg(self.theme.text))
                .highlight_style(Style::default().fg(self.theme.primary).add_modifier(Modifier::BOLD));
            
            frame.render_widget(tabs, area);
            return;
        }
        
        // Need to scroll - calculate viewport to center selected tab
        let selected_width = tab_widths[selected];
        
        // Try to center selected tab
        let mut start_idx = 0;
        let mut end_idx = all_titles.len();
        let mut viewport_width = 0;
        
        // Calculate how much space before and after selected tab
        let before_width = tab_widths[..selected].iter().sum::<usize>();
        let after_width = tab_widths[selected + 1..].iter().sum::<usize>();
        
        // If selected + content before/after exceeds available width, trim from edges
        if selected_width + before_width + after_width > available_width {
            // Start from selected and grow outward
            viewport_width = selected_width;
            let mut left = selected;
            let mut right = selected + 1;
            
            // Try to balance left and right, prioritizing centering
            while viewport_width < available_width && (left > 0 || right < all_titles.len()) {
                let can_add_left = left > 0 && viewport_width + tab_widths[left - 1] <= available_width;
                let can_add_right = right < all_titles.len() && viewport_width + tab_widths[right] <= available_width;
                
                // Prefer adding to the side with more content to keep centered
                let left_has_more = left > all_titles.len() - right;
                
                if can_add_left && (left_has_more || !can_add_right) {
                    left -= 1;
                    viewport_width += tab_widths[left];
                } else if can_add_right {
                    viewport_width += tab_widths[right];
                    right += 1;
                } else {
                    break;
                }
            }
            
            start_idx = left;
            end_idx = right;
        }
        
        // Build visible titles
        let visible_titles: Vec<String> = all_titles[start_idx..end_idx].to_vec();
        let adjusted_selected = selected - start_idx;
        
        // Add scroll indicators
        let title = if start_idx > 0 && end_idx < all_titles.len() {
            "Sections ← ··· →"
        } else if start_idx > 0 {
            "Sections ←"
        } else if end_idx < all_titles.len() {
            "Sections →"
        } else {
            "Sections"
        };
        
        let tabs = Tabs::new(visible_titles)
            .block(Block::default().borders(Borders::ALL).title(title.to_string()))
            .select(adjusted_selected)
            .style(Style::default().fg(self.theme.text))
            .highlight_style(Style::default().fg(self.theme.primary).add_modifier(Modifier::BOLD));
        
        frame.render_widget(tabs, area);
    }
    
    fn render_content(&mut self, frame: &mut Frame, area: Rect) {
        let section_data = self.get_current_section().map(|s| (s.id.clone(), s.title.clone(), s.fields.clone()));
        
        if let Some((section_id, section_title, fields)) = section_data {
            let mut items: Vec<ListItem> = Vec::new();
            let mut current_subsection: Option<String> = None;
            let mut field_to_visual_map: Vec<usize> = Vec::new(); // Maps field index to visual list index
            
            for (field_idx, field) in fields.iter().enumerate() {
                // Add subsection header if changed
                if field.subsection.as_ref() != current_subsection.as_ref() {
                    if let Some(ref subsec) = field.subsection {
                        // Add spacing before subsection (except first)
                        if !items.is_empty() {
                            items.push(ListItem::new(Line::from("")));
                        }
                        
                        // Add subsection header
                        let header = Line::from(vec![
                            Span::styled("━━━ ", Style::default().fg(self.theme.text_dim)),
                            Span::styled(subsec, Style::default().fg(self.theme.secondary).add_modifier(Modifier::BOLD)),
                            Span::styled(" ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━", Style::default().fg(self.theme.text_dim)),
                        ]);
                        items.push(ListItem::new(header));
                        current_subsection = Some(subsec.clone());
                    }
                }
                
                // Render field
                let field_key = format!("{}.{}", section_id, field.id);
                let value_display = self.get_value_display(&field_key, field);
                
                let is_focused = field_idx == self.current_field && !self.edit_mode;
                let is_editing = self.edit_mode && self.active_field.as_ref() == Some(&field_key);
                
                let style = if is_editing {
                    Style::default().fg(self.theme.primary).add_modifier(Modifier::BOLD)
                } else if is_focused {
                    Style::default().fg(self.theme.warning)
                } else {
                    Style::default().fg(self.theme.text)
                };
                
                let content = Line::from(vec![
                    Span::styled(format!("{}: ", field.label), Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(value_display, style),
                ]);
                
                items.push(ListItem::new(content));
                
                // Record mapping: field_idx -> visual index (after adding field to list)
                field_to_visual_map.push(items.len() - 1);
            }
            
            // Update list_state to point to the visual index of current field
            let visual_index = field_to_visual_map.get(self.current_field).copied().unwrap_or(0);
            self.list_state.select(Some(visual_index));
            
            let list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!("{} (↑↓ navigate, Enter edit, Space toggle)", section_title))
                )
                .highlight_style(
                    Style::default()
                        .bg(self.theme.highlight_bg)
                        .fg(self.theme.highlight_fg)
                        .add_modifier(Modifier::BOLD)
                )
                .highlight_symbol("» ");
            
            frame.render_stateful_widget(list, area, &mut self.list_state);
            
            // Render active widget if editing
            if self.edit_mode {
                if let Some(field_key) = &self.active_field {
                    if let Some(widget) = self.active_widgets.get(field_key) {
                        // Use the visual index for positioning
                        let visual_pos = field_to_visual_map.get(self.current_field).copied().unwrap_or(0);
                        let widget_y = area.y + 2 + visual_pos as u16;
                        let widget_area = Rect {
                            x: area.x + 1,
                            y: widget_y,
                            width: area.width.saturating_sub(2),
                            height: 3.min(area.height.saturating_sub(widget_y - area.y)),
                        };
                        
                        // Clear the area first for non-dropdown widgets to prevent transparency
                        // Dropdowns handle their own Clear widget and popup rendering
                        if let Some(field) = self.get_current_field() {
                            use crate::schema::UIWidget;
                            match field.ui_widget {
                                UIWidget::Dropdown | UIWidget::DropdownSearchable => {
                                    // Dropdowns render their own popup with Clear
                                }
                                _ => {
                                    // Other widgets need Clear to have opaque background
                                    use ratatui::widgets::Clear;
                                    frame.render_widget(Clear, widget_area);
                                }
                            }
                        }
                        
                        // Render widget at the specific field position
                        widget.render(frame, widget_area, true, &self.theme);
                    }
                }
            }
        }
    }
    
    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let mut help_spans = vec![
            Span::styled("Tab/←→", Style::default().fg(self.theme.primary)),
            Span::raw(" sections  "),
            Span::styled("↑↓", Style::default().fg(self.theme.primary)),
            Span::raw(" fields  "),
            Span::styled("Enter", Style::default().fg(self.theme.primary)),
            Span::raw(" edit  "),
        ];
        
        // Add 'e' for external editor if current field is a path
        if let Some(field) = self.get_current_field() {
            if matches!(field.field_type, crate::schema::FieldType::Path { .. }) && !self.edit_mode {
                help_spans.push(Span::styled("e", Style::default().fg(self.theme.primary)));
                help_spans.push(Span::raw(" $EDITOR  "));
            }
        }
        
        help_spans.push(Span::styled("q", Style::default().fg(self.theme.primary)));
        help_spans.push(Span::raw(" quit"));
        
        let mut lines = vec![Line::from(help_spans)];
        
        if let Some(msg) = &self.message {
            lines.push(Line::from(vec![
                Span::styled("Status: ", Style::default().fg(self.theme.success)),
                Span::raw(msg),
            ]));
        }
        
        // Show field description
        if let Some(field) = self.get_current_field() {
            lines.push(Line::from(vec![
                Span::styled("Help: ", Style::default().fg(self.theme.secondary)),
                Span::styled(&field.description, Style::default().fg(self.theme.text)),
            ]));
        }
        
        let footer = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("Controls"));
        
        frame.render_widget(footer, area);
    }
    
    fn get_value_display(&self, key: &str, field: &SchemaField) -> String {
        if let Some(value) = self.values.get(key) {
            match value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => if *b { "✓ true" } else { "✗ false" }.to_string(),
                _ => value.to_string(),
            }
        } else {
            // Show default
            match &field.field_type {
                FieldType::String { default, .. } => default.clone().unwrap_or_default(),
                FieldType::Number { default, .. } => default.map(|n| n.to_string()).unwrap_or_default(),
                FieldType::Boolean { default } => if *default { "✓ true" } else { "✗ false" }.to_string(),
                FieldType::Enum { default, .. } => default.clone().unwrap_or_default(),
                _ => String::new(),
            }
        }
    }
}
