// Phase 4: Redis Cache Module
// In-memory and distributed caching with TTL support

use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, SystemTime};

/// Cache entry with TTL
#[derive(Clone, Debug)]
struct CacheEntry {
    value: Value,
    expires_at: SystemTime,
}

impl CacheEntry {
    fn is_expired(&self) -> bool {
        SystemTime::now() > self.expires_at
    }
}

/// In-memory cache (Phase 4 - memory tier)
pub struct Cache {
    entries: Arc<RwLock<HashMap<String, CacheEntry>>>,
}

impl Cache {
    /// Create a new cache
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set a cached value with TTL
    pub async fn set(&self, key: String, value: Value, ttl_secs: u64) {
        let expires_at = SystemTime::now() + Duration::from_secs(ttl_secs);
        let entry = CacheEntry {
            value,
            expires_at,
        };
        let mut entries = self.entries.write().await;
        entries.insert(key, entry);
    }

    /// Get a cached value (returns None if expired)
    pub async fn get(&self, key: &str) -> Option<Value> {
        let mut entries = self.entries.write().await;
        
        if let Some(entry) = entries.get(key) {
            if entry.is_expired() {
                entries.remove(key);
                return None;
            }
            return Some(entry.value.clone());
        }
        None
    }

    /// Delete a cached value
    pub async fn delete(&self, key: &str) {
        let mut entries = self.entries.write().await;
        entries.remove(key);
    }

    /// Clear all expired entries (cleanup)
    pub async fn cleanup(&self) {
        let mut entries = self.entries.write().await;
        entries.retain(|_, entry| !entry.is_expired());
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        let entries = self.entries.read().await;
        CacheStats {
            total_entries: entries.len(),
            expired_entries: entries.values().filter(|e| e.is_expired()).count(),
        }
    }
}

/// Cache statistics
#[derive(Debug, serde::Serialize)]
pub struct CacheStats {
    pub total_entries: usize,
    pub expired_entries: usize,
}

/// Cache key builder
pub struct CacheKey;

impl CacheKey {
    pub fn user(user_id: &str) -> String {
        format!("user:{}", user_id)
    }

    pub fn endpoint(path: &str) -> String {
        format!("endpoint:{}", path)
    }

    pub fn data(resource_type: &str, id: &str) -> String {
        format!("data:{}:{}", resource_type, id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_set_and_get() {
        let cache = Cache::new();
        let value = serde_json::json!({"test": "data"});
        cache.set("test_key".to_string(), value.clone(), 60).await;
        
        let retrieved = cache.get("test_key").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), value);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let cache = Cache::new();
        let value = serde_json::json!({"test": "data"});
        cache.set("test_key".to_string(), value, 0).await; // 0 second TTL
        
        tokio::time::sleep(Duration::from_millis(100)).await;
        let retrieved = cache.get("test_key").await;
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_cache_delete() {
        let cache = Cache::new();
        let value = serde_json::json!({"test": "data"});
        cache.set("test_key".to_string(), value, 60).await;
        
        cache.delete("test_key").await;
        let retrieved = cache.get("test_key").await;
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_cache_key_builder() {
        assert_eq!(CacheKey::user("123"), "user:123");
        assert_eq!(CacheKey::endpoint("/api/users"), "endpoint:/api/users");
        assert_eq!(CacheKey::data("user", "123"), "data:user:123");
    }
}
