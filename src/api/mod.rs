use crate::internal::models::{Comment, Story};
use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::de::DeserializeOwned;
use strum_macros::Display;

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

pub fn hn_item_url(id: u32) -> String {
    format!("{}item/{}.json", HN_API_BASE_URL, id)
}

pub fn get_story_list_url(list_type: StoryListType) -> String {
    format!("{}{}.json", HN_API_BASE_URL, list_type.as_api_str())
}

/// HTTP API service for fetching Hacker News data.
///
/// This service uses `reqwest::blocking::Client` and returns `anyhow::Result` with
/// contextualized errors to preserve diagnostic information instead of erasing it
/// into plain strings.
#[derive(Clone)]
pub struct ApiService {
    client: Client,
}

impl ApiService {
    /// Create a new `ApiService` with a default reqwest blocking client.
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Generic helper to GET a URL and deserialize the JSON body into `T`.
    fn get_json<T>(&self, url: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let resp = self
            .client
            .get(url)
            .send()
            .with_context(|| format!("failed to send GET request to {}", url))?;

        resp.json::<T>()
            .with_context(|| format!("failed to parse JSON response from {}", url))
    }

    /// Fetch a list of story IDs for the given list type (e.g., top, new).
    pub fn fetch_story_ids(&self, list_type: StoryListType) -> Result<Vec<u32>> {
        let url = get_story_list_url(list_type);
        self.get_json(&url)
            .with_context(|| format!("fetch_story_ids failed for list {:?}", list_type))
    }

    /// Fetch a single story item by id.
    pub fn fetch_story_content(&self, id: u32) -> Result<Story> {
        let url = hn_item_url(id);
        self.get_json(&url)
            .with_context(|| format!("fetch_story_content failed for id {}", id))
    }

    /// Fetch a single comment item by id.
    pub fn fetch_comment_content(&self, id: u32) -> Result<Comment> {
        let url = hn_item_url(id);
        self.get_json(&url)
            .with_context(|| format!("fetch_comment_content failed for id {}", id))
    }
}

impl Default for ApiService {
    fn default() -> Self {
        Self::new()
    }
}
