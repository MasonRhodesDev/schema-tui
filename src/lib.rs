pub mod schema;
pub mod options;
pub mod config;
pub mod tui;

// Re-export commonly used types
pub use schema::{ConfigSchema, SchemaField, FieldType, OptionSource, UIWidget, SchemaParser};
pub use options::{OptionProvider, OptionResolver};
pub use config::{ConfigStore, ConfigLoader, ConfigSaver};
pub use tui::{Widget, WidgetResult, SchemaTUI, SchemaTUIBuilder, Theme};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_pattern() {
        // This will be a proper test once we have real schema/config files
        assert!(true);
    }
}
