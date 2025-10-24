use super::{ConfigSchema, FieldType};
use anyhow::{anyhow, Result};
use serde_json::Value;

pub struct SchemaValidator;

impl SchemaValidator {
    pub fn validate_schema(schema: &ConfigSchema) -> Result<()> {
        if schema.sections.is_empty() {
            return Err(anyhow!("Schema must have at least one section"));
        }

        for section in &schema.sections {
            if section.fields.is_empty() {
                return Err(anyhow!("Section '{}' has no fields", section.id));
            }
        }

        Ok(())
    }

    pub fn validate_value(field_type: &FieldType, value: &Value) -> Result<()> {
        match field_type {
            FieldType::String { max_length, .. } => {
                let s = value
                    .as_str()
                    .ok_or_else(|| anyhow!("Value must be a string"))?;

                if let Some(max) = max_length {
                    if s.len() > *max {
                        return Err(anyhow!("String exceeds max length of {}", max));
                    }
                }
            }

            FieldType::Number { min, max, .. } => {
                let n = value
                    .as_i64()
                    .ok_or_else(|| anyhow!("Value must be a number"))?;

                if let Some(min_val) = min {
                    if n < *min_val {
                        return Err(anyhow!("Number is below minimum of {}", min_val));
                    }
                }

                if let Some(max_val) = max {
                    if n > *max_val {
                        return Err(anyhow!("Number exceeds maximum of {}", max_val));
                    }
                }
            }

            FieldType::Float { min, max, .. } => {
                let f = value
                    .as_f64()
                    .ok_or_else(|| anyhow!("Value must be a number"))?;

                if let Some(min_val) = min {
                    if f < *min_val {
                        return Err(anyhow!("Number is below minimum of {}", min_val));
                    }
                }

                if let Some(max_val) = max {
                    if f > *max_val {
                        return Err(anyhow!("Number exceeds maximum of {}", max_val));
                    }
                }
            }

            FieldType::Boolean { .. } => {
                if !value.is_boolean() {
                    return Err(anyhow!("Value must be a boolean"));
                }
            }

            FieldType::Enum { .. } => {
                if !value.is_string() {
                    return Err(anyhow!("Enum value must be a string"));
                }
            }

            FieldType::Path { must_exist, .. } => {
                let path_str = value
                    .as_str()
                    .ok_or_else(|| anyhow!("Path must be a string"))?;

                if *must_exist {
                    let path = std::path::Path::new(path_str);
                    if !path.exists() {
                        return Err(anyhow!("Path does not exist: {}", path_str));
                    }
                }
            }
        }

        Ok(())
    }
}
