use super::app::SchemaTUI;
use super::theme::Theme;
use crate::schema::{ConfigSchema, SchemaParser};
use crate::config::ConfigLoader;
use crate::options::{OptionProvider, OptionResolver};
use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

pub struct SchemaTUIBuilder {
    schema: Option<ConfigSchema>,
    initial_values: Option<HashMap<String, Value>>,
    option_providers: Vec<(String, Box<dyn OptionProvider>)>,
    theme: Theme,
    config_path: Option<std::path::PathBuf>,
}

impl SchemaTUIBuilder {
    pub fn new() -> Self {
        Self {
            schema: None,
            initial_values: None,
            option_providers: Vec::new(),
            theme: Theme::default(),
            config_path: None,
        }
    }
    
    pub fn schema(mut self, schema: ConfigSchema) -> Self {
        self.schema = Some(schema);
        self
    }
    
    pub fn schema_file(mut self, path: impl AsRef<Path>) -> Result<Self> {
        let schema = SchemaParser::from_file(path)?;
        self.schema = Some(schema);
        Ok(self)
    }
    
    pub fn initial_values(mut self, values: HashMap<String, Value>) -> Self {
        self.initial_values = Some(values);
        self
    }
    
    pub fn config_file(mut self, path: impl AsRef<Path>) -> Result<Self> {
        let path_buf = path.as_ref().to_path_buf();
        // Load without expanding env vars so TUI can display literal $VAR values
        let config = ConfigLoader::from_toml_file_with_expansion(&path_buf, false)?;
        // Flatten nested structure to dot-notation keys for TUI
        self.initial_values = Some(config.as_flat_map());
        self.config_path = Some(path_buf);
        Ok(self)
    }
    
    pub fn register_option_provider(
        mut self,
        name: impl Into<String>,
        provider: Box<dyn OptionProvider>,
    ) -> Self {
        self.option_providers.push((name.into(), provider));
        self
    }
    
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }
    
    pub fn build(self) -> Result<SchemaTUI> {
        let schema = self.schema
            .ok_or_else(|| anyhow::anyhow!("Schema not provided"))?;
        
        let initial_values = self.initial_values.unwrap_or_default();
        
        let mut option_resolver = OptionResolver::new();
        for (name, provider) in self.option_providers {
            option_resolver.register_provider(name, provider);
        }
        
        Ok(SchemaTUI::new(
            schema,
            initial_values,
            option_resolver,
            self.theme,
            self.config_path,
        ))
    }
}

impl Default for SchemaTUIBuilder {
    fn default() -> Self {
        Self::new()
    }
}
