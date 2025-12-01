use crate::internal::models::Story;
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;

const MAX_HISTORY_SIZE: usize = 50;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewedStory {
    pub id: u32,
    pub title: Option<String>,
    pub url: Option<String>,
    pub viewed_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct History {
    #[serde(skip)]
    file_path: PathBuf,
    history: VecDeque<ViewedStory>,
}

impl History {
    /// Create a new History instance with the default storage path
    pub fn new() -> Self {
        let file_path = Self::default_path();
        Self {
            file_path,
            history: VecDeque::with_capacity(MAX_HISTORY_SIZE),
        }
    }

    /// Get the default storage path for history
    fn default_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("gpui-hn-app");

        // Ensure directory exists
        let _ = fs::create_dir_all(&config_dir);

        config_dir.join("history.json")
    }

    /// Load history from disk
    pub fn load() -> Self {
        let file_path = Self::default_path();

        match file_path.exists() {
            true => match fs::read_to_string(&file_path) {
                Ok(content) => match serde_json::from_str::<VecDeque<ViewedStory>>(&content) {
                    Ok(history) => {
                        tracing::info!(
                            "Loaded {} history entries from {}",
                            history.len(),
                            file_path.display()
                        );
                        Self { file_path, history }
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse history: {}", e);
                        Self::new()
                    }
                },
                Err(e) => {
                    tracing::error!("Failed to read history file: {}", e);
                    Self::new()
                }
            },
            false => {
                tracing::info!("No history file found, starting fresh");
                Self::new()
            }
        }
    }

    /// Save history to disk
    pub fn save(&self) {
        match serde_json::to_string_pretty(&self.history) {
            Ok(json) => match fs::write(&self.file_path, json) {
                Ok(_) => {
                    tracing::debug!(
                        "Saved {} history entries to {}",
                        self.history.len(),
                        self.file_path.display()
                    );
                }
                Err(e) => {
                    tracing::error!("Failed to save history: {}", e);
                }
            },
            Err(e) => {
                tracing::error!("Failed to serialize history: {}", e);
            }
        }
    }

    /// Add a story to history
    /// If story already exists, it's moved to the front (most recent)
    pub fn add(&mut self, story: &Story) {
        // Remove existing entry if present
        self.history.retain(|s| s.id != story.id);

        let viewed_story = ViewedStory {
            id: story.id,
            title: story.title.clone(),
            url: story.url.clone(),
            viewed_at: Timestamp::now(),
        };

        // Add to front
        self.history.push_front(viewed_story);

        // Maintain max size
        if self.history.len() > MAX_HISTORY_SIZE {
            self.history.pop_back();
        }

        tracing::debug!("Added story {} to history", story.id);
    }

    /// Get all history entries (already sorted by most recent first)
    pub fn get_all(&self) -> Vec<ViewedStory> {
        self.history.iter().cloned().collect()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.history.clear();
        tracing::info!("Cleared all history");
    }

    /// Get count of history entries
    #[allow(dead_code)]
    pub fn count(&self) -> usize {
        self.history.len()
    }

    /// Format "viewed X ago" string for a timestamp
    pub fn format_viewed_ago(timestamp: Timestamp) -> String {
        let now = Timestamp::now();

        // Calculate duration in seconds
        let duration = now.duration_since(timestamp);
        let seconds = duration.as_secs();

        match seconds {
            s if s < 60 => "just now".to_string(),
            s if s < 3600 => {
                let minutes = s / 60;
                format!("{}m ago", minutes)
            }
            s if s < 86400 => {
                let hours = s / 3600;
                format!("{}h ago", hours)
            }
            s if s < 2592000 => {
                // Less than 30 days
                let days = s / 86400;
                format!("{}d ago", days)
            }
            s => {
                let months = s / 2592000;
                format!("{}mo ago", months)
            }
        }
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_story(id: u32, title: &str) -> Story {
        Story {
            id,
            title: Some(title.to_string()),
            url: Some(format!("https://example.com/{}", id)),
            by: Some("test_user".to_string()),
            score: Some(100),
            time: Some(1234567890),
            descendants: Some(10),
            kids: None,
        }
    }

    #[test]
    fn test_add_story() {
        let mut history = History::new();
        let story = mock_story(1, "Test Story");

        history.add(&story);
        assert_eq!(history.count(), 1);

        let all = history.get_all();
        assert_eq!(all[0].id, 1);
        assert_eq!(all[0].title, Some("Test Story".to_string()));
    }

    #[test]
    fn test_add_duplicate_moves_to_front() {
        let mut history = History::new();

        let story1 = mock_story(1, "First Story");
        let story2 = mock_story(2, "Second Story");

        history.add(&story1);
        std::thread::sleep(std::time::Duration::from_millis(10));
        history.add(&story2);
        std::thread::sleep(std::time::Duration::from_millis(10));
        history.add(&story1); // Add first story again

        let all = history.get_all();
        assert_eq!(all.len(), 2);
        // First story should now be at front
        assert_eq!(all[0].id, 1);
        assert_eq!(all[1].id, 2);
    }

    #[test]
    fn test_max_capacity() {
        let mut history = History::new();

        // Add more than MAX_HISTORY_SIZE stories
        for i in 0..(MAX_HISTORY_SIZE + 10) {
            let story = mock_story(i as u32, &format!("Story {}", i));
            history.add(&story);
        }

        assert_eq!(history.count(), MAX_HISTORY_SIZE);

        // Most recent should be at front
        let all = history.get_all();
        assert_eq!(all[0].id, (MAX_HISTORY_SIZE + 9) as u32);
    }

    #[test]
    fn test_clear() {
        let mut history = History::new();

        let story = mock_story(1, "Test Story");
        history.add(&story);
        assert_eq!(history.count(), 1);

        history.clear();
        assert_eq!(history.count(), 0);
    }

    #[test]
    fn test_save_load() {
        use std::env;

        let temp_dir = env::temp_dir().join("gpui_hn_test_history");
        fs::create_dir_all(&temp_dir).unwrap();
        let test_file = temp_dir.join("test_history.json");

        // Create and save history
        let mut history = History {
            file_path: test_file.clone(),
            history: VecDeque::new(),
        };

        let story = mock_story(1, "Test Story");
        history.add(&story);
        history.save();

        // Load history
        let content = fs::read_to_string(&test_file).unwrap();
        let loaded_history: VecDeque<ViewedStory> = serde_json::from_str(&content).unwrap();

        assert_eq!(loaded_history.len(), 1);
        assert_eq!(loaded_history[0].id, 1);

        // Cleanup
        let _ = fs::remove_file(test_file);
    }

    #[test]
    fn test_format_viewed_ago() {
        let now = Timestamp::now();

        // Test "just now"
        assert_eq!(History::format_viewed_ago(now), "just now");

        // Test minutes ago (5 minutes = 300 seconds)
        let minutes_ago = now
            .checked_sub(jiff::SignedDuration::from_secs(300))
            .unwrap();
        assert_eq!(History::format_viewed_ago(minutes_ago), "5m ago");

        // Test hours ago (2 hours = 7200 seconds)
        let hours_ago = now
            .checked_sub(jiff::SignedDuration::from_secs(7200))
            .unwrap();
        assert_eq!(History::format_viewed_ago(hours_ago), "2h ago");

        // Test days ago (3 days = 259200 seconds)
        let days_ago = now
            .checked_sub(jiff::SignedDuration::from_secs(259200))
            .unwrap();
        assert_eq!(History::format_viewed_ago(days_ago), "3d ago");
    }
}
