//! Learnings store - cross-session insights (max 20).

use ferrum_core::error::{FerrumError, Result};
use std::fs;
use std::path::Path;

const MAX_LEARNINGS: usize = 20;

/// Persistent learnings store
pub struct LearningsStore {
    path: std::path::PathBuf,
    learnings: Vec<String>,
}

impl LearningsStore {
    pub fn load(path: &Path) -> Result<Self> {
        let learnings = if path.exists() {
            let content = fs::read_to_string(path)
                .map_err(|e| FerrumError::SessionError(e.to_string()))?;
            Self::parse_learnings(&content)
        } else {
            vec![]
        };
        Ok(Self {
            path: path.to_path_buf(),
            learnings,
        })
    }

    pub fn new_in_memory() -> Self {
        Self {
            path: std::path::PathBuf::new(),
            learnings: vec![],
        }
    }

    fn parse_learnings(content: &str) -> Vec<String> {
        content.lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                if trimmed.starts_with("- ") {
                    Some(trimmed[2..].to_string())
                } else if trimmed.starts_with("* ") {
                    Some(trimmed[2..].to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn add(&mut self, learning: String) {
        if self.learnings.len() >= MAX_LEARNINGS {
            self.learnings.remove(0); // Remove oldest
        }
        self.learnings.push(learning);
    }

    pub fn get_active(&self) -> &[String] {
        &self.learnings
    }

    pub fn count(&self) -> usize {
        self.learnings.len()
    }

    pub fn persist(&self) -> Result<()> {
        if self.path.as_os_str().is_empty() {
            return Ok(()); // In-memory
        }
        let mut content = String::from("# Agent Learnings\n\n");
        for learning in &self.learnings {
            content.push_str(&format!("- {}\n", learning));
        }
        fs::write(&self.path, content)
            .map_err(|e| FerrumError::SessionError(e.to_string()))
    }

    pub fn clear(&mut self) {
        self.learnings.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_learnings_add_and_max() {
        let mut store = LearningsStore::new_in_memory();
        for i in 0..25 {
            store.add(format!("Learning {}", i));
        }
        assert_eq!(store.count(), MAX_LEARNINGS);
        // Should have kept the last 20
        assert_eq!(store.get_active()[0], "Learning 5");
    }

    #[test]
    fn test_parse_learnings() {
        let content = "# Learnings\n\n- First insight\n- Second insight\nNot a learning\n";
        let parsed = LearningsStore::parse_learnings(content);
        assert_eq!(parsed.len(), 2);
    }
}
