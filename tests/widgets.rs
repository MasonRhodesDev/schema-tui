use schema_tui::tui::{TextInput, Toggle, NumberInput, Widget, WidgetResult};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde_json::Value;

fn key_event(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

#[test]
fn test_text_input_basic() {
    let mut input = TextInput::new("Name", "John");
    
    assert_eq!(input.get_value(), Value::String("John".to_string()));
    
    input.set_value(Value::String("Jane".to_string()));
    assert_eq!(input.get_value(), Value::String("Jane".to_string()));
}

#[test]
fn test_text_input_editing() {
    let mut input = TextInput::new("Test", "");
    input.start_editing();
    
    let result = input.handle_key(key_event(KeyCode::Char('H')));
    assert!(matches!(result, WidgetResult::Changed(_)));
    
    input.handle_key(key_event(KeyCode::Char('i')));
    
    let result = input.handle_key(key_event(KeyCode::Enter));
    match result {
        WidgetResult::Confirmed(val) => {
            assert_eq!(val.as_str().unwrap(), "Hi");
        }
        _ => panic!("Expected confirmed result"),
    }
}

#[test]
fn test_text_input_cursor_movement() {
    let mut input = TextInput::new("Test", "Hello");
    input.start_editing();
    
    // Move left
    input.handle_key(key_event(KeyCode::Left));
    input.handle_key(key_event(KeyCode::Left));
    
    // Insert character
    input.handle_key(key_event(KeyCode::Char('X')));
    
    let value = input.get_value();
    assert_eq!(value.as_str().unwrap(), "HelXlo");
}

#[test]
fn test_toggle_basic() {
    let mut toggle = Toggle::new("Feature", false);
    
    assert_eq!(toggle.get_value(), Value::Bool(false));
    
    toggle.handle_key(key_event(KeyCode::Enter));
    assert_eq!(toggle.get_value(), Value::Bool(true));
    
    toggle.handle_key(key_event(KeyCode::Char(' ')));
    assert_eq!(toggle.get_value(), Value::Bool(false));
}

#[test]
fn test_number_input_validation() {
    let mut input = NumberInput::new("Age", 25, Some(0), Some(150));
    input.start_editing();
    
    // Clear and try invalid negative
    input.handle_key(key_event(KeyCode::Backspace));
    input.handle_key(key_event(KeyCode::Backspace));
    input.handle_key(key_event(KeyCode::Char('-')));
    input.handle_key(key_event(KeyCode::Char('5')));
    
    // Should not confirm invalid value
    let result = input.handle_key(key_event(KeyCode::Enter));
    assert!(matches!(result, WidgetResult::Continue));
}

#[test]
fn test_number_input_valid() {
    let mut input = NumberInput::new("Count", 0, Some(0), Some(100));
    input.start_editing();
    
    input.handle_key(key_event(KeyCode::Char('4')));
    input.handle_key(key_event(KeyCode::Char('2')));
    
    let result = input.handle_key(key_event(KeyCode::Enter));
    match result {
        WidgetResult::Confirmed(val) => {
            assert_eq!(val.as_i64().unwrap(), 42);
        }
        _ => panic!("Expected confirmed result"),
    }
}

#[test]
fn test_number_input_only_digits() {
    let mut input = NumberInput::new("Test", 0, None, None);
    input.start_editing();
    
    // Try to insert letter - should be rejected
    input.handle_key(key_event(KeyCode::Char('a')));
    // Value should still be valid (0)
    assert_eq!(input.get_value().as_i64().unwrap(), 0);
    
    // Insert digit - cursor at end, so becomes 05 which parses to 5
    input.handle_key(key_event(KeyCode::Char('5')));
    assert_eq!(input.get_value().as_i64().unwrap(), 5);
}
