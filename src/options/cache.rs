use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct OptionCache {
    cache: HashMap<String, CachedOptions>,
}

struct CachedOptions {
    options: Vec<String>,
    timestamp: Instant,
    duration: Duration,
}

impl Default for OptionCache {
    fn default() -> Self {
        Self::new()
    }
}

impl OptionCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&Vec<String>> {
        self.cache.get(key).and_then(|cached| {
            if cached.timestamp.elapsed() < cached.duration {
                Some(&cached.options)
            } else {
                None
            }
        })
    }

    pub fn insert(&mut self, key: String, options: Vec<String>, duration_secs: u64) {
        self.cache.insert(
            key,
            CachedOptions {
                options,
                timestamp: Instant::now(),
                duration: Duration::from_secs(duration_secs),
            },
        );
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }
}
