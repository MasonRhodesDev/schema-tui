use serde_json::Value;
use std::collections::HashMap;

pub struct ConfigStore {
    values: HashMap<String, Value>,
}

impl ConfigStore {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }
    
    pub fn from_map(values: HashMap<String, Value>) -> Self {
        Self { values }
    }
    
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.values.get(key)
    }
    
    pub fn set(&mut self, key: String, value: Value) {
        self.values.insert(key, value);
    }
    
    pub fn get_nested(&self, path: &str) -> Option<&Value> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = self.values.get(parts[0])?;
        
        for part in &parts[1..] {
            current = current.get(part)?;
        }
        
        Some(current)
    }
    
    pub fn set_nested(&mut self, path: &str, value: Value) {
        let parts: Vec<&str> = path.split('.').collect();
        
        if parts.len() == 1 {
            self.values.insert(path.to_string(), value);
            return;
        }
        
        let mut current = self.values
            .entry(parts[0].to_string())
            .or_insert_with(|| Value::Object(serde_json::Map::new()));
        
        for part in &parts[1..parts.len() - 1] {
            let obj = current.as_object_mut().unwrap();
            current = obj
                .entry(part.to_string())
                .or_insert_with(|| Value::Object(serde_json::Map::new()));
        }
        
        if let Value::Object(obj) = current {
            obj.insert(parts.last().unwrap().to_string(), value);
        }
    }
    
    pub fn as_map(&self) -> &HashMap<String, Value> {
        &self.values
    }
}
