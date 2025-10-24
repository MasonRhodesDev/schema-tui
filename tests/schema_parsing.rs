use schema_tui::schema::{FieldType, OptionSource, SchemaParser};

#[test]
fn test_parse_basic_schema() {
    let schema_json = r#"{
        "version": "1.0",
        "title": "Test Schema",
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
                        "default": "John Doe"
                    },
                    {
                        "id": "age",
                        "label": "Age",
                        "description": "Your age",
                        "type": "number",
                        "default": 25,
                        "min": 0,
                        "max": 150
                    },
                    {
                        "id": "enabled",
                        "label": "Enabled",
                        "description": "Enable feature",
                        "type": "boolean",
                        "default": true
                    }
                ]
            }
        ]
    }"#;

    let schema = SchemaParser::from_string(schema_json).unwrap();

    assert_eq!(schema.version, "1.0");
    assert_eq!(schema.title.unwrap(), "Test Schema");
    assert_eq!(schema.sections.len(), 1);
    assert_eq!(schema.sections[0].id, "general");
    assert_eq!(schema.sections[0].fields.len(), 3);
}

#[test]
fn test_parse_enum_field() {
    let schema_json = r#"{
        "version": "1.0",
        "sections": [
            {
                "id": "settings",
                "title": "Settings",
                "fields": [
                    {
                        "id": "mode",
                        "label": "Mode",
                        "description": "Application mode",
                        "type": "enum",
                        "options_source": {
                            "type": "static",
                            "values": ["Light", "Dark", "Auto"]
                        },
                        "default": "Dark"
                    }
                ]
            }
        ]
    }"#;

    let schema = SchemaParser::from_string(schema_json).unwrap();
    let field = &schema.sections[0].fields[0];

    match &field.field_type {
        FieldType::Enum {
            options_source,
            default,
        } => {
            match options_source {
                OptionSource::Static { values } => {
                    assert_eq!(values.len(), 3);
                    assert_eq!(values[0], "Light");
                }
                _ => panic!("Expected static option source"),
            }
            assert_eq!(default.as_ref().unwrap(), "Dark");
        }
        _ => panic!("Expected enum field type"),
    }
}

#[test]
fn test_parse_path_field() {
    let schema_json = r#"{
        "version": "1.0",
        "sections": [
            {
                "id": "paths",
                "title": "Paths",
                "fields": [
                    {
                        "id": "config_file",
                        "label": "Config File",
                        "description": "Path to config",
                        "type": "path",
                        "default": "~/config.toml",
                        "file_type": "any",
                        "must_exist": false
                    }
                ]
            }
        ]
    }"#;

    let schema = SchemaParser::from_string(schema_json).unwrap();
    let field = &schema.sections[0].fields[0];

    match &field.field_type {
        FieldType::Path {
            default,
            must_exist,
            ..
        } => {
            assert_eq!(default.as_ref().unwrap(), "~/config.toml");
            assert_eq!(*must_exist, false);
        }
        _ => panic!("Expected path field type"),
    }
}

#[test]
fn test_invalid_schema() {
    let invalid_json = r#"{
        "version": "1.0"
    }"#;

    let result = SchemaParser::from_string(invalid_json);
    assert!(result.is_err());
}
