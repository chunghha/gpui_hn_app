use crate::cache::Cache;
use crate::internal::models::{Comment, Story};
use anyhow::{Context, Result};
use dashmap::DashMap;
use futures::future::{BoxFuture, FutureExt, Shared};
use futures::stream::{self, StreamExt};
use once_cell::sync::OnceCell;
use reqwest::Client;
use serde::de::DeserializeOwned;
use std::sync::Arc;
use std::time::Duration;
use strum_macros::Display;
use tokio::runtime::Handle;
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;

// Global tokio runtime handle for use in GPUI tasks
static TOKIO_HANDLE: OnceCell<Handle> = OnceCell::new();

/// Initialize the global tokio runtime handle
pub fn init_tokio_handle(handle: Handle) {
    let _ = TOKIO_HANDLE.set(handle);
}

/// Get the global tokio runtime handle, lazily creating a runtime if not already initialized.
/// This ensures tests (or other usages) that do not explicitly call `init_tokio_handle` still
/// have a valid tokio handle. The created runtime is leaked to the process lifetime so the
/// handle remains valid for `'static`.
fn tokio_handle() -> &'static Handle {
    if let Some(h) = TOKIO_HANDLE.get() {
        return h;
    }

    // Create a new runtime and leak it to keep it alive for the process lifetime.
    // This provides a fallback for tests that don't initialize a tokio handle explicitly.
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    let handle = rt.handle().clone();
    // Leak the runtime so its handle remains valid for 'static lifetime.
    let _ = Box::leak(Box::new(rt));
    // Try to set the handle; ignore error if another thread set it concurrently.
    let _ = TOKIO_HANDLE.set(handle);
    TOKIO_HANDLE
        .get()
        .expect("Tokio runtime handle not initialized")
}

/// Helper function to make an HTTP GET request in tokio context
/// This should be used by other modules that need to make HTTP requests
pub async fn http_get(url: &str) -> Result<String> {
    let client = Client::new();
    let url = url.to_string();
    let result = tokio_handle()
        .spawn(async move {
            let resp = client.get(&url).send().await?;
            resp.text().await
        })
        .await
        .expect("Tokio task panicked");

    result.map_err(anyhow::Error::new)
}

/// Types of Hacker News story lists we can fetch.
#[derive(Debug, Clone, Copy, PartialEq, Display)]
pub enum StoryListType {
    Best,
    Top,
    New,
    Ask,
    Show,
    Job,
}

impl StoryListType {
    fn as_api_str(&self) -> &str {
        match self {
            Self::Best => "beststories",
            Self::Top => "topstories",
            Self::New => "newstories",
            Self::Ask => "askstories",
            Self::Show => "showstories",
            Self::Job => "jobstories",
        }
    }
}

const HN_API_BASE_URL: &str = "https://hacker-news.firebaseio.com/v0/";

/// Type alias for in-flight request tracking map
type InflightRequestMap =
    Arc<DashMap<String, Shared<BoxFuture<'static, Result<Arc<String>, String>>>>>;

#[cfg(test)]
pub fn hn_item_url(id: u32) -> String {
    format!("{}item/{}.json", HN_API_BASE_URL, id)
}

#[cfg(test)]
pub fn get_story_list_url(list_type: StoryListType) -> String {
    format!("{}{}.json", HN_API_BASE_URL, list_type.as_api_str())
}

/// Network configuration for API requests
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Maximum number of retry attempts (0 = no retries)
    pub max_retries: u32,
    /// Initial retry delay in milliseconds
    pub initial_retry_delay_ms: u64,
    /// Maximum retry delay in milliseconds (caps exponential backoff)
    pub max_retry_delay_ms: u64,
    /// Whether to retry on timeout errors
    pub retry_on_timeout: bool,
    /// Maximum number of concurrent requests
    pub concurrent_requests: usize,
    /// Rate limit in requests per second
    pub rate_limit_per_second: f64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_retry_delay_ms: 500,
            max_retry_delay_ms: 5000,
            retry_on_timeout: true,
            concurrent_requests: 10,
            rate_limit_per_second: 3.0,
        }
    }
}

/// HTTP API service for fetching Hacker News data with caching, rate limiting,
/// request deduplication, and exponential backoff retries.
///
/// This service uses async `reqwest::Client` with in-memory caching (TTL-based),
/// semaphore-based rate limiting, and request deduplication to prevent duplicate
/// in-flight requests.
#[derive(Clone)]
pub struct ApiService {
    client: Client,
    story_ids_cache: Cache<Vec<u32>>,
    story_cache: Cache<Story>,
    comment_cache: Cache<Comment>,
    rate_limiter: Arc<Semaphore>,
    network_config: NetworkConfig,
    base_url: Option<String>,
    // Request deduplication map
    inflight_requests: InflightRequestMap,
    enable_metrics: bool,
}

impl ApiService {
    /// Create a new `ApiService` with default configuration.
    pub fn new() -> Self {
        Self::with_config(NetworkConfig::default())
    }

    /// Create a new `ApiService` with custom network configuration.
    pub fn with_config(network_config: NetworkConfig) -> Self {
        let permits = network_config.rate_limit_per_second.ceil() as usize;
        let rate_limiter = Arc::new(Semaphore::new(permits));
        let client = Client::new();

        Self {
            client,
            story_ids_cache: Cache::new(300), // 5 min TTL for story lists
            story_cache: Cache::new(300),     // 5 min TTL for stories
            comment_cache: Cache::new(300),   // 5 min TTL for comments
            rate_limiter,
            network_config,
            base_url: None,
            inflight_requests: Arc::new(DashMap::new()),
            enable_metrics: false,
        }
    }

    /// Create a service with a custom base URL (for testing).
    #[allow(dead_code)]
    pub fn with_base_url(base_url: String) -> Self {
        let network_config = NetworkConfig::default();
        let permits = network_config.rate_limit_per_second.ceil() as usize;
        let client = Client::new();

        Self {
            client,
            story_ids_cache: Cache::new(300),
            story_cache: Cache::new(300),
            comment_cache: Cache::new(300),
            rate_limiter: Arc::new(Semaphore::new(permits)),
            network_config,
            base_url: Some(base_url),
            inflight_requests: Arc::new(DashMap::new()),
            enable_metrics: false,
        }
    }

    /// Enable performance metrics logging.
    #[allow(dead_code)]
    pub fn with_metrics(mut self, enable: bool) -> Self {
        self.enable_metrics = enable;
        self
    }

    fn get_base_url(&self) -> &str {
        self.base_url.as_deref().unwrap_or(HN_API_BASE_URL)
    }

    /// Fetch raw text from URL with retries and exponential backoff.
    #[tracing::instrument(skip(self), fields(url = %url))]
    async fn fetch_raw(&self, url: String) -> Result<Arc<String>> {
        let start = std::time::Instant::now();
        let mut attempt = 0;
        let mut delay = self.network_config.initial_retry_delay_ms;

        loop {
            attempt += 1;

            // Acquire rate limiting permit
            let _permit = self
                .rate_limiter
                .acquire()
                .await
                .expect("Semaphore should never be closed");

            // Enter tokio runtime context for reqwest
            let client = self.client.clone();
            let url_clone = url.clone();
            let resp_result = tokio_handle()
                .spawn(async move { client.get(&url_clone).send().await })
                .await
                .expect("Tokio task panicked");

            match resp_result {
                Ok(resp) => {
                    let text = resp
                        .text()
                        .await
                        .with_context(|| format!("failed to get response text from {}", url))?;

                    if self.enable_metrics {
                        tracing::debug!(
                            elapsed = ?start.elapsed(),
                            url = %url,
                            attempt = attempt,
                            "GET successful"
                        );
                    }
                    return Ok(Arc::new(text));
                }
                Err(e) => {
                    let is_timeout = e.is_timeout();
                    let is_connect = e.is_connect();
                    let should_retry =
                        (is_timeout && self.network_config.retry_on_timeout) || is_connect;

                    if !should_retry || attempt > self.network_config.max_retries {
                        if self.enable_metrics {
                            tracing::debug!(
                                elapsed = ?start.elapsed(),
                                url = %url,
                                attempt = attempt,
                                error = %e,
                                "GET failed (final)"
                            );
                        }
                        return Err(anyhow::Error::new(e))
                            .with_context(|| format!("failed to send GET request to {}", url));
                    }

                    tracing::warn!(
                        "Request to {} failed (attempt {}/{}): {}. Retrying in {}ms...",
                        url,
                        attempt,
                        self.network_config.max_retries + 1,
                        e,
                        delay
                    );

                    tokio::time::sleep(Duration::from_millis(delay)).await;
                    delay = (delay * 2).min(self.network_config.max_retry_delay_ms);
                }
            }
        }
    }

    /// Generic helper to GET a URL and deserialize the JSON body into `T`.
    /// Uses request deduplication to prevent duplicate in-flight requests.
    #[tracing::instrument(skip(self), fields(url = %url))]
    async fn get_json<T>(&self, url: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        // Request deduplication logic
        let future = {
            if let Some(future) = self.inflight_requests.get(url) {
                if self.enable_metrics {
                    tracing::debug!(url = %url, "Deduplicated request joined");
                }
                future.clone()
            } else {
                let url_owned = url.to_string();
                let self_clone = self.clone();

                let future = async move {
                    self_clone
                        .fetch_raw(url_owned)
                        .await
                        .map_err(|e| e.to_string())
                }
                .boxed()
                .shared();

                self.inflight_requests
                    .insert(url.to_string(), future.clone());
                future
            }
        };

        // Wait for the request to complete
        let result = future.await;

        // Clean up the map entry after request completes
        self.inflight_requests.remove(url);

        match result {
            Ok(body) => {
                let parsed = serde_json::from_str::<T>(&body)
                    .with_context(|| format!("failed to parse JSON response from {}", url))?;
                Ok(parsed)
            }
            Err(e_str) => Err(anyhow::anyhow!(e_str)),
        }
    }

    /// Fetch a list of story IDs for the given list type (e.g., top, new).
    /// Uses cache with 5 min TTL. Supports cancellation.
    #[tracing::instrument(skip(self, token), fields(list_type = ?list_type))]
    pub async fn fetch_story_ids(
        &self,
        list_type: StoryListType,
        token: Option<CancellationToken>,
    ) -> Result<Vec<u32>> {
        let start = std::time::Instant::now();
        let cache_key = format!("story_ids_{}", list_type.as_api_str());

        // Check cache first
        if let Some(cached) = self.story_ids_cache.get(&cache_key) {
            return Ok(cached);
        }

        // Check cancellation before request
        if let Some(token) = &token
            && token.is_cancelled()
        {
            return Err(anyhow::anyhow!("Request cancelled"));
        }

        // Cache miss - fetch from API
        let url = format!("{}{}.json", self.get_base_url(), list_type.as_api_str());

        let result: Vec<u32> = if let Some(token) = token {
            tokio::select! {
                res = self.get_json(&url) => {
                    res.with_context(|| format!("fetch_story_ids failed for list {:?}", list_type))?
                }
                _ = token.cancelled() => {
                    return Err(anyhow::anyhow!("Request cancelled"));
                }
            }
        } else {
            self.get_json(&url)
                .await
                .with_context(|| format!("fetch_story_ids failed for list {:?}", list_type))?
        };

        // Store in cache
        self.story_ids_cache.insert(cache_key, result.clone());

        if self.enable_metrics {
            tracing::debug!(elapsed = ?start.elapsed(), count = result.len(), "Fetched story IDs");
        }

        Ok(result)
    }

    /// Fetch a single story item by id.
    /// Uses cache with 5 min TTL. Falls back to stale cache on network errors.
    #[tracing::instrument(skip(self), fields(id = %id))]
    pub async fn fetch_story_content(&self, id: u32) -> Result<Story> {
        let cache_key = format!("story_{}", id);

        // Check cache first
        if let Some(cached) = self.story_cache.get(&cache_key) {
            tracing::trace!("Cache hit for story {}", id);
            return Ok(cached);
        }

        if self.enable_metrics {
            tracing::trace!("Cache miss for story {}", id);
        }

        let start = std::time::Instant::now();

        // If not in cache or expired, fetch from API
        let url = format!("{}item/{}.json", self.get_base_url(), id);

        let story = match self.get_json::<Story>(&url).await {
            Ok(s) => s,
            Err(e) => {
                // Try to get stale data from cache as fallback
                if let Some(stale_story) = self.story_cache.get_stale(&cache_key) {
                    tracing::warn!("Using stale cache for story {}: {}", id, e);
                    return Ok(stale_story);
                }
                return Err(e.context(format!("fetch_story_content failed for id {}", id)));
            }
        };

        self.story_cache.insert(cache_key, story.clone());

        if self.enable_metrics {
            tracing::debug!(elapsed = ?start.elapsed(), "Fetched and cached story content");
        }

        Ok(story)
    }

    /// Fetch a single comment item by id.
    /// Uses cache with 5 min TTL. Falls back to stale cache on network errors.
    #[tracing::instrument(skip(self), fields(id = %id))]
    pub async fn fetch_comment_content(&self, id: u32) -> Result<Comment> {
        let cache_key = format!("comment_{}", id);

        // Check cache first
        if let Some(cached) = self.comment_cache.get(&cache_key) {
            tracing::trace!("Cache hit for comment {}", id);
            return Ok(cached);
        }

        if self.enable_metrics {
            tracing::trace!("Cache miss for comment {}", id);
        }

        let start = std::time::Instant::now();

        // If not in cache or expired, fetch from API
        let url = format!("{}item/{}.json", self.get_base_url(), id);

        let comment = match self.get_json::<Comment>(&url).await {
            Ok(c) => c,
            Err(e) => {
                // Try to get stale data from cache as fallback
                if let Some(stale_comment) = self.comment_cache.get_stale(&cache_key) {
                    tracing::warn!("Using stale cache for comment {}: {}", id, e);
                    return Ok(stale_comment);
                }
                return Err(e.context(format!("fetch_comment_content failed for id {}", id)));
            }
        };

        self.comment_cache.insert(cache_key, comment.clone());

        if self.enable_metrics {
            tracing::debug!(elapsed = ?start.elapsed(), "Fetched and cached comment content");
        }

        Ok(comment)
    }

    /// Fetch multiple stories concurrently.
    /// Uses `buffer_unordered` to limit concurrency based on network config.
    /// Supports cancellation via token.
    #[tracing::instrument(skip(self, ids, token), fields(count = ids.len()))]
    pub async fn fetch_stories_concurrent(
        &self,
        ids: Vec<u32>,
        token: Option<CancellationToken>,
    ) -> Vec<Story> {
        let start = std::time::Instant::now();

        // Check cancellation before starting
        if let Some(token) = &token
            && token.is_cancelled()
        {
            tracing::warn!("Request cancelled before starting story fetch");
            return Vec::new();
        }

        let limit = self.network_config.concurrent_requests;

        let results: Vec<Result<Story>> = stream::iter(ids.into_iter())
            .map(|id| {
                let api = self.clone();
                let token = token.clone();
                async move {
                    if let Some(token) = &token
                        && token.is_cancelled()
                    {
                        return Err(anyhow::anyhow!("Request cancelled"));
                    }
                    api.fetch_story_content(id).await
                }
            })
            .buffer_unordered(limit)
            .collect()
            .await;

        let stories: Vec<Story> = results.into_iter().filter_map(|r| r.ok()).collect();

        if self.enable_metrics {
            tracing::debug!(
                elapsed = ?start.elapsed(),
                successful = stories.len(),
                "Fetched stories concurrently"
            );
        }

        stories
    }

    /// Fetch multiple comments concurrently.
    /// Uses `buffer_unordered` to limit concurrency based on network config.
    /// Supports cancellation via token.
    #[tracing::instrument(skip(self, ids, token), fields(count = ids.len()))]
    pub async fn fetch_comments_concurrent(
        &self,
        ids: Vec<u32>,
        token: Option<CancellationToken>,
    ) -> Vec<Comment> {
        let start = std::time::Instant::now();

        // Check cancellation before starting
        if let Some(token) = &token
            && token.is_cancelled()
        {
            tracing::warn!("Request cancelled before starting comment fetch");
            return Vec::new();
        }

        let limit = self.network_config.concurrent_requests;

        let results: Vec<Result<Comment>> = stream::iter(ids.into_iter())
            .map(|id| {
                let api = self.clone();
                let token = token.clone();
                async move {
                    if let Some(token) = &token
                        && token.is_cancelled()
                    {
                        return Err(anyhow::anyhow!("Request cancelled"));
                    }
                    api.fetch_comment_content(id).await
                }
            })
            .buffer_unordered(limit)
            .collect()
            .await;

        let comments: Vec<Comment> = results.into_iter().filter_map(|r| r.ok()).collect();

        if self.enable_metrics {
            tracing::debug!(
                elapsed = ?start.elapsed(),
                successful = comments.len(),
                "Fetched comments concurrently"
            );
        }

        comments
    }
}

impl Default for ApiService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_story_list_type_as_api_str() {
        assert_eq!(StoryListType::Best.as_api_str(), "beststories");
        assert_eq!(StoryListType::Top.as_api_str(), "topstories");
        assert_eq!(StoryListType::New.as_api_str(), "newstories");
        assert_eq!(StoryListType::Ask.as_api_str(), "askstories");
        assert_eq!(StoryListType::Show.as_api_str(), "showstories");
        assert_eq!(StoryListType::Job.as_api_str(), "jobstories");
    }

    #[test]
    fn test_hn_item_url() {
        assert_eq!(
            hn_item_url(12345),
            "https://hacker-news.firebaseio.com/v0/item/12345.json"
        );
    }

    #[test]
    fn test_get_story_list_url() {
        assert_eq!(
            get_story_list_url(StoryListType::Top),
            "https://hacker-news.firebaseio.com/v0/topstories.json"
        );
        assert_eq!(
            get_story_list_url(StoryListType::Best),
            "https://hacker-news.firebaseio.com/v0/beststories.json"
        );
    }

    #[tokio::test]
    async fn test_fetch_story_ids_success() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/topstories.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("[1, 2, 3, 4, 5]")
            .create();

        let service = ApiService::with_base_url(format!("{}/", server.url()));
        let result = service.fetch_story_ids(StoryListType::Top, None).await;

        mock.assert();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![1, 2, 3, 4, 5]);
    }

    #[tokio::test]
    async fn test_fetch_story_ids_network_error() {
        // Use a URL that will fail to connect
        let service = ApiService::with_base_url("http://localhost:1/".to_string());
        let result = service.fetch_story_ids(StoryListType::Top, None).await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("fetch_story_ids failed"));
    }

    #[tokio::test]
    async fn test_fetch_story_ids_with_cancellation() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/topstories.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("[1, 2, 3, 4, 5]")
            .create();

        let service = ApiService::with_base_url(format!("{}/", server.url()));
        let token = CancellationToken::new();
        token.cancel(); // Cancel immediately

        let result = service
            .fetch_story_ids(StoryListType::Top, Some(token))
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cancelled"));
    }

    #[tokio::test]
    async fn test_fetch_story_content_success() {
        let mut server = mockito::Server::new_async().await;
        let story_json = r#"{
            "by": "testuser",
            "descendants": 10,
            "id": 12345,
            "kids": [1, 2, 3],
            "score": 100,
            "time": 1234567890,
            "title": "Test Story",
            "type": "story",
            "url": "https://example.com"
        }"#;

        let mock = server
            .mock("GET", "/item/12345.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(story_json)
            .create();

        let service = ApiService::with_base_url(format!("{}/", server.url()));
        let result = service.fetch_story_content(12345).await;

        mock.assert();
        assert!(result.is_ok());
        let story = result.unwrap();
        assert_eq!(story.id, 12345);
        assert_eq!(story.title, Some("Test Story".to_string()));
        assert_eq!(story.by, Some("testuser".to_string()));
    }

    #[tokio::test]
    async fn test_fetch_story_content_invalid_json() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/item/12345.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("invalid json")
            .create();

        let service = ApiService::with_base_url(format!("{}/", server.url()));
        let result = service.fetch_story_content(12345).await;

        mock.assert();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fetch_comment_content_success() {
        let mut server = mockito::Server::new_async().await;
        let comment_json = r#"{
            "by": "commenter",
            "id": 67890,
            "kids": [10, 11],
            "parent": 12345,
            "text": "This is a comment",
            "time": 1234567890,
            "type": "comment"
        }"#;

        let mock = server
            .mock("GET", "/item/67890.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(comment_json)
            .create();

        let service = ApiService::with_base_url(format!("{}/", server.url()));
        let result = service.fetch_comment_content(67890).await;

        mock.assert();
        assert!(result.is_ok());
        let comment = result.unwrap();
        assert_eq!(comment.id, 67890);
        assert_eq!(comment.by, Some("commenter".to_string()));
        assert_eq!(comment.text, Some("This is a comment".to_string()));
    }

    #[tokio::test]
    async fn test_fetch_comment_content_http_error() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/item/99999.json")
            .with_status(404)
            .create();

        let service = ApiService::with_base_url(format!("{}/", server.url()));
        let result = service.fetch_comment_content(99999).await;

        mock.assert();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_request_deduplication() {
        let mut server = mockito::Server::new_async().await;
        let story_json = r#"{
            "by": "testuser",
            "descendants": 10,
            "id": 12345,
            "kids": [1, 2, 3],
            "score": 100,
            "time": 1234567890,
            "title": "Test Story",
            "type": "story",
            "url": "https://example.com"
        }"#;

        // Mock should only be called once due to deduplication
        let mock = server
            .mock("GET", "/item/12345.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(story_json)
            .expect(1) // Should only be called once
            .create();

        let service = ApiService::with_base_url(format!("{}/", server.url()));

        // Make multiple concurrent requests for the same story
        let handles: Vec<_> = (0..5)
            .map(|_| {
                let service_clone = service.clone();
                tokio::spawn(async move { service_clone.fetch_story_content(12345).await })
            })
            .collect();

        // Wait for all requests to complete
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }

        mock.assert();
    }

    #[tokio::test]
    async fn test_api_service_default() {
        let service = ApiService::default();
        // Just verify we can create a default instance
        assert!(service.get_base_url() == HN_API_BASE_URL);
    }

    #[tokio::test]
    async fn test_network_config_custom() {
        let config = NetworkConfig {
            max_retries: 5,
            initial_retry_delay_ms: 1000,
            max_retry_delay_ms: 10000,
            retry_on_timeout: false,
            concurrent_requests: 20,
            rate_limit_per_second: 5.0,
        };

        let service = ApiService::with_config(config);
        assert_eq!(service.network_config.max_retries, 5);
        assert_eq!(service.network_config.concurrent_requests, 20);
    }
}
