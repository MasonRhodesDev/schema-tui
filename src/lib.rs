pub mod schema;
pub mod options;
pub mod config;
pub mod tui;

// Re-export commonly used types
pub use schema::{ConfigSchema, SchemaField, FieldType, OptionSource, UIWidget};
pub use options::{OptionProvider, OptionResolver};
pub use config::{ConfigStore, ConfigLoader, ConfigSaver};
pub use tui::{Widget, WidgetResult};

// Placeholder for TUI - will implement next
pub struct SchemaTUI {
    schema: ConfigSchema,
    config: ConfigStore,
    options: OptionResolver,
}

impl SchemaTUI {
    pub fn from_files(
        schema_path: impl AsRef<std::path::Path>,
        config_path: impl AsRef<std::path::Path>,
    ) -> anyhow::Result<Self> {
        let schema = schema::SchemaParser::from_file(schema_path)?;
        let config = ConfigLoader::from_toml_file(config_path)?;
        let options = OptionResolver::new();
        
        Ok(Self {
            schema,
            config,
            options,
        })
    }
    
    pub fn builder() -> SchemaTUIBuilder {
        SchemaTUIBuilder::new()
    }
    
    pub fn get_config(&self) -> &ConfigStore {
        &self.config
    }
    
    pub fn register_option_provider(&mut self, name: String, provider: Box<dyn OptionProvider>) {
        self.options.register_provider(name, provider);
    }
    
    pub fn save_to_file(&self, path: impl AsRef<std::path::Path>) -> anyhow::Result<()> {
        ConfigSaver::save_toml(&self.config, &self.schema, path)
    }
    
    // Placeholder for TUI run method
    pub fn run(&mut self) -> anyhow::Result<()> {
        todo!("TUI implementation coming next")
    }
}

pub struct SchemaTUIBuilder {
    schema: Option<ConfigSchema>,
    config: Option<ConfigStore>,
    providers: Vec<(String, Box<dyn OptionProvider>)>,
}

impl SchemaTUIBuilder {
    pub fn new() -> Self {
        Self {
            schema: None,
            config: None,
            providers: Vec::new(),
        }
    }
    
    pub fn schema_file(mut self, path: impl AsRef<std::path::Path>) -> anyhow::Result<Self> {
        self.schema = Some(schema::SchemaParser::from_file(path)?);
        Ok(self)
    }
    
    pub fn config_file(mut self, path: impl AsRef<std::path::Path>) -> anyhow::Result<Self> {
        self.config = Some(ConfigLoader::from_toml_file(path)?);
        Ok(self)
    }
    
    pub fn register_option_provider(
        mut self,
        name: impl Into<String>,
        provider: Box<dyn OptionProvider>,
    ) -> Self {
        self.providers.push((name.into(), provider));
        self
    }
    
    pub fn build(self) -> anyhow::Result<SchemaTUI> {
        let schema = self.schema.ok_or_else(|| anyhow::anyhow!("Schema not provided"))?;
        let config = self.config.ok_or_else(|| anyhow::anyhow!("Config not provided"))?;
        
        let mut options = OptionResolver::new();
        for (name, provider) in self.providers {
            options.register_provider(name, provider);
        }
        
        Ok(SchemaTUI {
            schema,
            config,
            options,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_pattern() {
        // This will be a proper test once we have real schema/config files
        assert!(true);
    }
}
