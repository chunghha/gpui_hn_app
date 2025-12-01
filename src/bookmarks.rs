use crate::internal::models::Story;
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkedStory {
    pub id: u32,
    pub title: Option<String>,
    pub url: Option<String>,
    pub bookmarked_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmarks {
    #[serde(skip)]
    file_path: PathBuf,
    bookmarks: HashMap<u32, BookmarkedStory>,
}

impl Bookmarks {
    /// Create a new Bookmarks instance with the default storage path
    pub fn new() -> Self {
        let file_path = Self::default_path();
        Self {
            file_path,
            bookmarks: HashMap::new(),
        }
    }

    /// Get the default storage path for bookmarks
    fn default_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("gpui-hn-app");

        // Ensure directory exists
        let _ = fs::create_dir_all(&config_dir);

        config_dir.join("bookmarks.json")
    }

    /// Load bookmarks from disk
    pub fn load() -> Self {
        let file_path = Self::default_path();

        match file_path.exists() {
            true => match fs::read_to_string(&file_path) {
                Ok(content) => {
                    match serde_json::from_str::<HashMap<u32, BookmarkedStory>>(&content) {
                        Ok(bookmarks) => {
                            tracing::info!(
                                "Loaded {} bookmarks from {}",
                                bookmarks.len(),
                                file_path.display()
                            );
                            Self {
                                file_path,
                                bookmarks,
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to parse bookmarks: {}", e);
                            Self::new()
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to read bookmarks file: {}", e);
                    Self::new()
                }
            },
            false => {
                tracing::info!("No bookmarks file found, starting fresh");
                Self::new()
            }
        }
    }

    /// Save bookmarks to disk
    pub fn save(&self) {
        match serde_json::to_string_pretty(&self.bookmarks) {
            Ok(json) => match fs::write(&self.file_path, json) {
                Ok(_) => {
                    tracing::debug!(
                        "Saved {} bookmarks to {}",
                        self.bookmarks.len(),
                        self.file_path.display()
                    );
                }
                Err(e) => {
                    tracing::error!("Failed to save bookmarks: {}", e);
                }
            },
            Err(e) => {
                tracing::error!("Failed to serialize bookmarks: {}", e);
            }
        }
    }

    /// Toggle bookmark for a story (add if not bookmarked, remove if already bookmarked)
    pub fn toggle(&mut self, story: &Story) -> bool {
        match self.bookmarks.contains_key(&story.id) {
            true => {
                self.bookmarks.remove(&story.id);
                tracing::info!("Removed bookmark for story {}", story.id);
                false
            }
            false => {
                let bookmarked_story = BookmarkedStory {
                    id: story.id,
                    title: story.title.clone(),
                    url: story.url.clone(),
                    bookmarked_at: Timestamp::now(),
                };
                self.bookmarks.insert(story.id, bookmarked_story);
                tracing::info!("Added bookmark for story {}", story.id);
                true
            }
        }
    }

    /// Check if a story is bookmarked
    pub fn is_bookmarked(&self, story_id: u32) -> bool {
        self.bookmarks.contains_key(&story_id)
    }

    /// Get all bookmarks sorted by timestamp (newest first)
    pub fn get_all(&self) -> Vec<BookmarkedStory> {
        let mut bookmarks: Vec<BookmarkedStory> = self.bookmarks.values().cloned().collect();
        bookmarks.sort_by(|a, b| b.bookmarked_at.cmp(&a.bookmarked_at));
        bookmarks
    }

    /// Get count of bookmarks
    #[allow(dead_code)]
    pub fn count(&self) -> usize {
        self.bookmarks.len()
    }

    /// Export bookmarks to a JSON file
    #[allow(dead_code)]
    pub fn export(&self, path: &PathBuf) -> Result<(), String> {
        match serde_json::to_string_pretty(&self.bookmarks) {
            Ok(json) => {
                fs::write(path, json).map_err(|e| format!("Failed to write export file: {}", e))
            }
            Err(e) => Err(format!("Failed to serialize bookmarks: {}", e)),
        }
    }

    /// Import bookmarks from a JSON file
    #[allow(dead_code)]
    pub fn import(&mut self, path: &PathBuf) -> Result<usize, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read import file: {}", e))?;

        let imported: HashMap<u32, BookmarkedStory> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse import file: {}", e))?;

        let count = imported.len();
        self.bookmarks.extend(imported);
        Ok(count)
    }
}

impl Default for Bookmarks {
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
    fn test_toggle_bookmark() {
        let mut bookmarks = Bookmarks::new();
        let story = mock_story(1, "Test Story");

        // Add bookmark
        assert!(bookmarks.toggle(&story));
        assert!(bookmarks.is_bookmarked(1));
        assert_eq!(bookmarks.count(), 1);

        // Remove bookmark
        assert!(!bookmarks.toggle(&story));
        assert!(!bookmarks.is_bookmarked(1));
        assert_eq!(bookmarks.count(), 0);
    }

    #[test]
    fn test_is_bookmarked() {
        let mut bookmarks = Bookmarks::new();
        let story = mock_story(1, "Test Story");

        assert!(!bookmarks.is_bookmarked(1));
        bookmarks.toggle(&story);
        assert!(bookmarks.is_bookmarked(1));
    }

    #[test]
    fn test_get_all_sorted() {
        let mut bookmarks = Bookmarks::new();

        // Add bookmarks with slight delay to ensure different timestamps
        let story1 = mock_story(1, "First Story");
        bookmarks.toggle(&story1);

        std::thread::sleep(std::time::Duration::from_millis(10));

        let story2 = mock_story(2, "Second Story");
        bookmarks.toggle(&story2);

        let all = bookmarks.get_all();
        assert_eq!(all.len(), 2);
        // Newest first
        assert_eq!(all[0].id, 2);
        assert_eq!(all[1].id, 1);
    }

    #[test]
    fn test_save_load() {
        use std::env;

        let temp_dir = env::temp_dir().join("gpui_hn_test_bookmarks");
        fs::create_dir_all(&temp_dir).unwrap();
        let test_file = temp_dir.join("test_bookmarks.json");

        // Create and save bookmarks
        let mut bookmarks = Bookmarks {
            file_path: test_file.clone(),
            bookmarks: HashMap::new(),
        };

        let story = mock_story(1, "Test Story");
        bookmarks.toggle(&story);
        bookmarks.save();

        // Load bookmarks
        let _loaded = Bookmarks {
            file_path: test_file.clone(),
            bookmarks: HashMap::new(),
        };

        let content = fs::read_to_string(&test_file).unwrap();
        let loaded_bookmarks: HashMap<u32, BookmarkedStory> =
            serde_json::from_str(&content).unwrap();

        assert_eq!(loaded_bookmarks.len(), 1);
        assert!(loaded_bookmarks.contains_key(&1));

        // Cleanup
        let _ = fs::remove_file(test_file);
    }
}
