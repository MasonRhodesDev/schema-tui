pub mod config;
pub mod options;
pub mod schema;
pub mod tui;

// Re-export commonly used types
pub use config::{ConfigLoader, ConfigSaver, ConfigStore};
pub use options::{OptionProvider, OptionResolver};
pub use schema::{ConfigSchema, FieldType, OptionSource, SchemaField, SchemaParser, UIWidget};
pub use tui::{SchemaTUI, SchemaTUIBuilder, Theme, Widget, WidgetResult};

#[cfg(test)]
mod tests {

    #[test]
    fn test_builder_pattern() {
        // This will be a proper test once we have real schema/config files
        assert!(true);
    }
}
