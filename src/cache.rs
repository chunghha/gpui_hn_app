use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

#[derive(Clone)]
struct CacheEntry<T> {
    value: T,
    expires_at: Instant,
}

/// Thread-safe in-memory cache with TTL support
#[derive(Clone)]
pub struct Cache<T: Clone> {
    store: Arc<RwLock<HashMap<String, CacheEntry<T>>>>,
    ttl: Duration,
}

impl<T: Clone> Cache<T> {
    /// Create a new cache with the specified TTL in seconds
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    /// Get a value from the cache if it exists and hasn't expired
    pub fn get(&self, key: &str) -> Option<T> {
        let store = self.store.read().ok()?;
        let entry = store.get(key)?;

        if entry.expires_at > Instant::now() {
            Some(entry.value.clone())
        } else {
            // Entry has expired, we'll clean it up later
            None
        }
    }

    /// Insert a value into the cache with TTL
    pub fn insert(&self, key: String, value: T) {
        let entry = CacheEntry {
            value,
            expires_at: Instant::now() + self.ttl,
        };

        if let Ok(mut store) = self.store.write() {
            store.insert(key, entry);
        }
    }

    /// Remove expired entries from the cache
    #[allow(dead_code)]
    pub fn cleanup(&self) {
        if let Ok(mut store) = self.store.write() {
            let now = Instant::now();
            store.retain(|_, entry| entry.expires_at > now);
        }
    }

    /// Clear all entries from the cache
    #[allow(dead_code)]
    pub fn clear(&self) {
        if let Ok(mut store) = self.store.write() {
            store.clear();
        }
    }

    /// Get the number of entries in the cache (including expired)
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.store.read().map(|s| s.len()).unwrap_or(0)
    }

    /// Check if the cache is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_cache_insert_and_get() {
        let cache = Cache::new(300); // 5 minutes
        cache.insert("key1".to_string(), "value1".to_string());

        assert_eq!(cache.get("key1"), Some("value1".to_string()));
        assert_eq!(cache.get("key2"), None);
    }

    #[test]
    fn test_cache_expiration() {
        let cache = Cache::new(1); // 1 second TTL
        cache.insert("key1".to_string(), "value1".to_string());

        // Should exist immediately
        assert_eq!(cache.get("key1"), Some("value1".to_string()));

        // Wait for expiration
        thread::sleep(Duration::from_secs(2));

        // Should be expired
        assert_eq!(cache.get("key1"), None);
    }

    #[test]
    fn test_cache_cleanup() {
        let cache = Cache::new(1); // 1 second TTL
        cache.insert("key1".to_string(), "value1".to_string());
        cache.insert("key2".to_string(), "value2".to_string());

        assert_eq!(cache.len(), 2);

        // Wait for expiration
        thread::sleep(Duration::from_secs(2));

        // Cleanup expired entries
        cache.cleanup();

        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_clear() {
        let cache = Cache::new(300);
        cache.insert("key1".to_string(), "value1".to_string());
        cache.insert("key2".to_string(), "value2".to_string());

        assert_eq!(cache.len(), 2);

        cache.clear();

        assert_eq!(cache.len(), 0);
    }
}
