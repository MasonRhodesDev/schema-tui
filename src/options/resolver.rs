use super::OptionCache;
use crate::schema::OptionSource;
use anyhow::{anyhow, Result};
use std::collections::HashMap;

pub trait OptionProvider: Send + Sync {
    fn get_options(&self) -> Result<Vec<String>>;
}

pub struct OptionResolver {
    cache: OptionCache,
    providers: HashMap<String, Box<dyn OptionProvider>>,
}

impl OptionResolver {
    pub fn new() -> Self {
        Self {
            cache: OptionCache::new(),
            providers: HashMap::new(),
        }
    }
    
    pub fn register_provider(&mut self, name: String, provider: Box<dyn OptionProvider>) {
        self.providers.insert(name, provider);
    }
    
    pub async fn resolve(&mut self, source: &OptionSource) -> Result<Vec<String>> {
        match source {
            OptionSource::Static { values } => Ok(values.clone()),
            
            OptionSource::Script { command, cache_duration } => {
                self.resolve_from_script(command, *cache_duration).await
            }
            
            OptionSource::Function { name } => {
                self.resolve_from_provider(name)
            }
            
            OptionSource::Provider { provider } => {
                self.resolve_from_provider(provider)
            }
            
            OptionSource::FileList { directory, pattern, extract } => {
                self.resolve_from_file_list(directory, pattern, extract.as_deref())
            }
        }
    }
    
    async fn resolve_from_script(&mut self, command: &str, cache_duration: Option<u64>) -> Result<Vec<String>> {
        // Check cache first
        if cache_duration.is_some() {
            if let Some(cached) = self.cache.get(command) {
                return Ok(cached.clone());
            }
        }
        
        // Execute script
        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .await?;
        
        if !output.status.success() {
            return Err(anyhow!("Script failed: {}", String::from_utf8_lossy(&output.stderr)));
        }
        
        let stdout = String::from_utf8(output.stdout)?;
        let options: Vec<String> = serde_json::from_str(&stdout)?;
        
        // Update cache
        if let Some(duration) = cache_duration {
            self.cache.insert(command.to_string(), options.clone(), duration);
        }
        
        Ok(options)
    }
    
    pub fn resolve_from_provider(&self, name: &str) -> Result<Vec<String>> {
        let provider = self.providers.get(name)
            .ok_or_else(|| anyhow!("Unknown option provider: {}", name))?;
        
        provider.get_options()
    }
    
    pub fn resolve_from_script_sync(&self, command: &str) -> Result<Vec<String>> {
        use std::process::Command;
        
        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow!("Script failed: {}", String::from_utf8_lossy(&output.stderr)));
        }
        
        let stdout = String::from_utf8(output.stdout)?;
        
        // Try to parse as JSON array first
        if let Ok(options) = serde_json::from_str::<Vec<String>>(&stdout) {
            return Ok(options);
        }
        
        // Fallback: split by newlines
        Ok(stdout.lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect())
    }
    
    pub fn resolve_from_file_list(&self, directory: &str, pattern: &str, extract: Option<&str>) -> Result<Vec<String>> {
        let dir = expand_path(directory);
        let glob_pattern = format!("{}/{}", dir, pattern);
        
        let mut results = Vec::new();
        for entry in glob::glob(&glob_pattern)? {
            let path = entry?;
            let display_name = if let Some(regex_str) = extract {
                let re = regex::Regex::new(regex_str)?;
                if let Some(caps) = re.captures(&path.to_string_lossy()) {
                    caps.get(1)
                        .map(|m| m.as_str().to_string())
                        .unwrap_or_else(|| {
                            path.file_name()
                                .unwrap()
                                .to_string_lossy()
                                .to_string()
                        })
                } else {
                    continue;
                }
            } else {
                path.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            };
            results.push(display_name);
        }
        
        Ok(results)
    }
}

fn expand_path(path: &str) -> String {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return path.replacen("~", &home.display().to_string(), 1);
        }
    }
    path.to_string()
}
