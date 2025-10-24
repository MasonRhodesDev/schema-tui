use super::{expand_env_vars, ConfigStore};
use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

pub struct ConfigLoader;

impl ConfigLoader {
    pub fn from_toml_file(path: impl AsRef<Path>) -> Result<ConfigStore> {
        Self::from_toml_file_with_expansion(path, true)
    }

    pub fn from_toml_file_with_expansion(
        path: impl AsRef<Path>,
        expand: bool,
    ) -> Result<ConfigStore> {
        let content = std::fs::read_to_string(path)?;
        Self::from_toml_string_with_expansion(&content, expand)
    }

    pub fn from_toml_string(content: &str) -> Result<ConfigStore> {
        Self::from_toml_string_with_expansion(content, true)
    }

    pub fn from_toml_string_with_expansion(content: &str, expand: bool) -> Result<ConfigStore> {
        let toml_value: toml::Value = toml::from_str(content)?;
        let json_value = Self::toml_to_json(toml_value);

        let values = if let Value::Object(map) = json_value {
            map.into_iter().collect()
        } else {
            HashMap::new()
        };

        let mut store = ConfigStore::from_map(values);
        if expand {
            Self::expand_all_env_vars(&mut store);
        }

        Ok(store)
    }

    fn toml_to_json(toml_value: toml::Value) -> Value {
        match toml_value {
            toml::Value::String(s) => Value::String(s),
            toml::Value::Integer(i) => Value::Number(i.into()),
            toml::Value::Float(f) => Value::Number(serde_json::Number::from_f64(f).unwrap()),
            toml::Value::Boolean(b) => Value::Bool(b),
            toml::Value::Datetime(dt) => Value::String(dt.to_string()),
            toml::Value::Array(arr) => {
                Value::Array(arr.into_iter().map(Self::toml_to_json).collect())
            }
            toml::Value::Table(table) => {
                let map = table
                    .into_iter()
                    .map(|(k, v)| (k, Self::toml_to_json(v)))
                    .collect();
                Value::Object(map)
            }
        }
    }

    fn expand_all_env_vars(store: &mut ConfigStore) {
        let keys: Vec<String> = store.as_map().keys().cloned().collect();

        for key in keys {
            if let Some(value) = store.get(&key).cloned() {
                let expanded = Self::expand_value(&value);
                store.set(key, expanded);
            }
        }
    }

    fn expand_value(value: &Value) -> Value {
        match value {
            Value::String(s) => Value::String(expand_env_vars(s)),
            Value::Object(map) => {
                let expanded_map: serde_json::Map<String, Value> = map
                    .iter()
                    .map(|(k, v)| (k.clone(), Self::expand_value(v)))
                    .collect();
                Value::Object(expanded_map)
            }
            Value::Array(arr) => Value::Array(arr.iter().map(Self::expand_value).collect()),
            _ => value.clone(),
        }
    }
}
