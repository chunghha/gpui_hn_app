use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;

const MAX_HISTORY_SIZE: usize = 20;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHistory {
    history: VecDeque<String>,
    #[serde(skip)]
    file_path: PathBuf,
}

impl SearchHistory {
    pub fn new(config_dir: PathBuf) -> Self {
        let file_path = config_dir.join("search_history.json");
        let mut history = Self {
            history: VecDeque::new(),
            file_path: file_path.clone(),
        };
        history.load();
        history
    }

    pub fn load(&mut self) {
        if let Ok(content) = fs::read_to_string(&self.file_path)
            && let Ok(loaded) = serde_json::from_str::<VecDeque<String>>(&content)
        {
            self.history = loaded;
        }
    }

    pub fn save(&self) {
        if let Some(parent) = self.file_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(content) = serde_json::to_string_pretty(&self.history) {
            let _ = fs::write(&self.file_path, content);
        }
    }

    pub fn add(&mut self, query: String) {
        if query.trim().is_empty() {
            return;
        }

        // Remove if already exists to move it to the front
        if let Some(pos) = self.history.iter().position(|x| x == &query) {
            self.history.remove(pos);
        }

        self.history.push_front(query);

        if self.history.len() > MAX_HISTORY_SIZE {
            self.history.pop_back();
        }

        self.save();
    }

    pub fn get_all(&self) -> Vec<String> {
        self.history.iter().cloned().collect()
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.history.clear();
        self.save();
    }
}
