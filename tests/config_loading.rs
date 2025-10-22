use schema_tui::config::{ConfigLoader, ConfigStore, expand_env_vars};
use std::env;
use tempfile::NamedTempFile;
use std::io::Write;

#[test]
fn test_load_toml_config() {
    let toml_content = r#"
[general]
name = "Test User"
age = 30
enabled = true

[paths]
home = "~/Documents"
    "#;

    let config = ConfigLoader::from_toml_string(toml_content).unwrap();
    
    let name = config.get_nested("general.name").unwrap();
    assert_eq!(name.as_str().unwrap(), "Test User");
    
    let age = config.get_nested("general.age").unwrap();
    assert_eq!(age.as_i64().unwrap(), 30);
}

#[test]
fn test_env_var_expansion_tilde() {
    let input = "~/config.toml";
    let result = expand_env_vars(input);
    
    // Should not start with tilde after expansion
    assert!(!result.starts_with("~"));
    assert!(result.contains("config.toml"));
}

#[test]
fn test_env_var_expansion_dollar() {
    env::set_var("TEST_VAR_123", "test_value");
    
    let input = "$TEST_VAR_123/file.txt";
    let result = expand_env_vars(input);
    
    assert_eq!(result, "test_value/file.txt");
}

#[test]
fn test_env_var_expansion_braces() {
    env::set_var("TEST_VAR_456", "another_value");
    
    let input = "${TEST_VAR_456}/config";
    let result = expand_env_vars(input);
    
    assert_eq!(result, "another_value/config");
}

#[test]
fn test_config_store_nested_access() {
    let mut store = ConfigStore::new();
    
    store.set_nested("section.field", serde_json::json!("value"));
    
    let retrieved = store.get_nested("section.field").unwrap();
    assert_eq!(retrieved.as_str().unwrap(), "value");
}

#[test]
fn test_config_from_file() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "[test]").unwrap();
    writeln!(temp_file, "key = \"value\"").unwrap();
    
    let config = ConfigLoader::from_toml_file(temp_file.path()).unwrap();
    let value = config.get_nested("test.key").unwrap();
    
    assert_eq!(value.as_str().unwrap(), "value");
}
