use super::ConfigStore;
use crate::schema::{ConfigSchema, FieldType};
use anyhow::Result;
use serde_json::Value;
use std::path::Path;

pub struct ConfigSaver;

impl ConfigSaver {
    pub fn save_toml(
        store: &ConfigStore,
        schema: &ConfigSchema,
        path: impl AsRef<Path>,
    ) -> Result<()> {
        let content = Self::generate_toml_with_comments(store, schema)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    fn generate_toml_with_comments(store: &ConfigStore, schema: &ConfigSchema) -> Result<String> {
        let mut output = String::new();

        // Header
        if let Some(title) = &schema.title {
            output.push_str(&format!("# {}\n", title));
        }
        if let Some(desc) = &schema.description {
            output.push_str(&format!("# {}\n", desc));
        }
        output.push_str("# This file is auto-generated but safe to edit manually\n\n");

        for section in &schema.sections {
            output.push_str(&format!("[{}]\n", section.id));

            if let Some(desc) = &section.description {
                output.push_str(&format!("# {}\n", desc));
            }

            for field in &section.fields {
                output.push_str(&format!("# {}\n", field.description));

                let field_key = format!("{}.{}", section.id, field.id);
                let value = if let Some(v) = store.get_nested(&field_key) {
                    Some(v.clone())
                } else {
                    Self::get_default_value(&field.field_type)
                };

                if let Some(val) = value {
                    let value_str = Self::format_value(&val);
                    output.push_str(&format!("{} = {}\n\n", field.id, value_str));
                } else {
                    // Write empty string for missing values
                    output.push_str(&format!("{} = \"\"\n\n", field.id));
                }
            }

            output.push('\n');
        }

        Ok(output)
    }

    fn get_default_value(field_type: &FieldType) -> Option<Value> {
        match field_type {
            FieldType::String { default, .. } => default.as_ref().map(|s| Value::String(s.clone())),
            FieldType::Number { default, .. } => {
                default.as_ref().map(|n| Value::Number((*n).into()))
            }
            FieldType::Float { default, .. } => default
                .as_ref()
                .and_then(|f| serde_json::Number::from_f64(*f).map(Value::Number)),
            FieldType::Boolean { default } => Some(Value::Bool(*default)),
            FieldType::Enum { default, .. } => default.as_ref().map(|s| Value::String(s.clone())),
            FieldType::Path { default, .. } => default.as_ref().map(|s| Value::String(s.clone())),
        }
    }

    fn format_value(value: &Value) -> String {
        match value {
            Value::String(s) => format!("\"{}\"", s.replace('"', "\\\"")),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Array(arr) => {
                let items: Vec<String> = arr.iter().map(Self::format_value).collect();
                format!("[{}]", items.join(", "))
            }
            Value::Object(_) => "{}".to_string(),
            Value::Null => "null".to_string(),
        }
    }
}
