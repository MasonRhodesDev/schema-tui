use super::OptionCache;
use crate::schema::OptionSource;
use anyhow::{anyhow, Result};
use serde_json::Value;
use std::collections::HashMap;

pub trait OptionProvider: Send + Sync {
    fn get_options(&self) -> Result<Vec<String>>;
}

pub struct OptionResolver {
    cache: OptionCache,
    providers: HashMap<String, Box<dyn OptionProvider>>,
}

impl Default for OptionResolver {
    fn default() -> Self {
        Self::new()
    }
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

    pub async fn resolve(
        &mut self,
        source: &OptionSource,
        values: &HashMap<String, Value>,
    ) -> Result<Vec<String>> {
        match source {
            OptionSource::Static { values } => Ok(values.clone()),

            OptionSource::Script {
                command,
                cache_duration,
                ..
            } => {
                self.resolve_from_script(command, *cache_duration, values)
                    .await
            }

            OptionSource::Function { name } => self.resolve_from_provider(name),

            OptionSource::Provider { provider } => self.resolve_from_provider(provider),

            OptionSource::FileList {
                directory,
                pattern,
                extract,
            } => self.resolve_from_file_list(directory, pattern, extract.as_deref()),
        }
    }

    async fn resolve_from_script(
        &mut self,
        command: &str,
        cache_duration: Option<u64>,
        values: &HashMap<String, Value>,
    ) -> Result<Vec<String>> {
        let substituted_command = Self::substitute_variables(command, values)?;
        let cache_key = format!("{}:{}", command, substituted_command);

        if cache_duration.is_some() {
            if let Some(cached) = self.cache.get(&cache_key) {
                return Ok(cached.clone());
            }
        }

        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(&substituted_command)
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!(
                "Script failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let stdout = String::from_utf8(output.stdout)?;
        let options: Vec<String> = serde_json::from_str(&stdout)?;

        if let Some(duration) = cache_duration {
            self.cache.insert(cache_key, options.clone(), duration);
        }

        Ok(options)
    }

    pub fn resolve_from_provider(&self, name: &str) -> Result<Vec<String>> {
        let provider = self
            .providers
            .get(name)
            .ok_or_else(|| anyhow!("Unknown option provider: {}", name))?;

        provider.get_options()
    }

    pub fn resolve_from_script_sync(
        &self,
        command: &str,
        values: &HashMap<String, Value>,
    ) -> Result<Vec<String>> {
        use std::process::Command;

        let substituted_command = Self::substitute_variables(command, values)?;

        let output = Command::new("sh")
            .arg("-c")
            .arg(&substituted_command)
            .output()?;

        if !output.status.success() {
            return Err(anyhow!(
                "Script failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let stdout = String::from_utf8(output.stdout)?;

        if let Ok(options) = serde_json::from_str::<Vec<String>>(&stdout) {
            return Ok(options);
        }

        Ok(stdout
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect())
    }

    fn substitute_variables(command: &str, values: &HashMap<String, Value>) -> Result<String> {
        use regex::Regex;

        let re = Regex::new(r"\$\{([^}]+)\}").unwrap();
        let mut result = command.to_string();

        for cap in re.captures_iter(command) {
            let full_match = &cap[0];
            let var_name = &cap[1];

            let replacement = values
                .get(var_name)
                .map(|v| match v {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    _ => String::new(),
                })
                .unwrap_or_default();

            result = result.replace(full_match, &replacement);
        }

        Ok(result)
    }

    pub fn resolve_from_file_list(
        &self,
        directory: &str,
        pattern: &str,
        extract: Option<&str>,
    ) -> Result<Vec<String>> {
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
                        .unwrap_or_else(|| path.file_name().unwrap().to_string_lossy().to_string())
                } else {
                    continue;
                }
            } else {
                path.file_name().unwrap().to_string_lossy().to_string()
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn test_substitute_variables() {
        let mut values = HashMap::new();
        values.insert(
            "daemon.language".to_string(),
            Value::String("en".to_string()),
        );
        values.insert("daemon.count".to_string(), Value::Number(42.into()));
        values.insert("daemon.enabled".to_string(), Value::Bool(true));

        let result = OptionResolver::substitute_variables(
            "script.sh ${daemon.language} ${daemon.count}",
            &values,
        )
        .unwrap();
        assert_eq!(result, "script.sh en 42");

        let result =
            OptionResolver::substitute_variables("check ${daemon.enabled}", &values).unwrap();
        assert_eq!(result, "check true");
    }

    #[test]
    fn test_substitute_missing_variable() {
        let values = HashMap::new();

        let result =
            OptionResolver::substitute_variables("script.sh ${missing.var}", &values).unwrap();
        assert_eq!(result, "script.sh ");
    }

    #[test]
    fn test_substitute_no_variables() {
        let values = HashMap::new();

        let result =
            OptionResolver::substitute_variables("script.sh static args", &values).unwrap();
        assert_eq!(result, "script.sh static args");
    }
}
