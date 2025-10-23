use serde_json::Value;
use std::collections::HashMap;

/// Evaluates a condition string like "general.use_matugen == true"
pub fn evaluate_condition(condition: &str, values: &HashMap<String, Value>) -> bool {
    let condition = condition.trim();
    
    // Parse: "field_key operator value"
    if let Some((field_part, value_part)) = condition.split_once("==") {
        let field_key = field_part.trim();
        let expected_value = value_part.trim();
        
        if let Some(actual_value) = values.get(field_key) {
            match actual_value {
                Value::Bool(b) => {
                    // Compare boolean
                    if expected_value == "true" {
                        return *b;
                    } else if expected_value == "false" {
                        return !*b;
                    }
                }
                Value::String(s) => {
                    // Compare string (remove quotes from expected)
                    let expected = expected_value.trim_matches('"').trim_matches('\'');
                    return s == expected;
                }
                Value::Number(n) => {
                    // Compare number
                    if let Ok(expected_num) = expected_value.parse::<i64>() {
                        if let Some(actual_num) = n.as_i64() {
                            return actual_num == expected_num;
                        }
                    }
                }
                _ => {}
            }
        }
    } else if let Some((field_part, value_part)) = condition.split_once("!=") {
        let field_key = field_part.trim();
        let expected_value = value_part.trim();
        
        if let Some(actual_value) = values.get(field_key) {
            match actual_value {
                Value::Bool(b) => {
                    if expected_value == "true" {
                        return !*b;
                    } else if expected_value == "false" {
                        return *b;
                    }
                }
                Value::String(s) => {
                    let expected = expected_value.trim_matches('"').trim_matches('\'');
                    return s != expected;
                }
                _ => {}
            }
        }
    }
    
    // Default: always visible if condition can't be evaluated
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_boolean_condition() {
        let mut values = HashMap::new();
        values.insert("general.use_matugen".to_string(), Value::Bool(true));
        
        assert!(evaluate_condition("general.use_matugen == true", &values));
        assert!(!evaluate_condition("general.use_matugen == false", &values));
    }
    
    #[test]
    fn test_string_condition() {
        let mut values = HashMap::new();
        values.insert("general.mode".to_string(), Value::String("dark".to_string()));
        
        assert!(evaluate_condition("general.mode == \"dark\"", &values));
        assert!(!evaluate_condition("general.mode == \"light\"", &values));
    }
}
