use super::ConfigSchema;
use anyhow::Result;
use std::path::Path;

pub struct SchemaParser;

impl SchemaParser {
    pub fn from_file(path: impl AsRef<Path>) -> Result<ConfigSchema> {
        let content = std::fs::read_to_string(path)?;
        Self::from_string(&content)
    }

    pub fn from_string(content: &str) -> Result<ConfigSchema> {
        let schema: ConfigSchema = serde_json::from_str(content)?;
        Ok(schema)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_schema() {
        let schema_json = r#"{
            "version": "1.0",
            "title": "Test Config",
            "sections": [
                {
                    "id": "general",
                    "title": "General Settings",
                    "fields": [
                        {
                            "id": "name",
                            "label": "Name",
                            "description": "Your name",
                            "type": "string",
                            "default": "John"
                        }
                    ]
                }
            ]
        }"#;

        let schema = SchemaParser::from_string(schema_json).unwrap();
        assert_eq!(schema.version, "1.0");
        assert_eq!(schema.sections.len(), 1);
        assert_eq!(schema.sections[0].id, "general");
    }
}
