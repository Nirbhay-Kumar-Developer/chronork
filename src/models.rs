use serde::{Deserialize, Serialize};

// --- Data Structures ---

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: String,
    pub timestamp: i64,
    pub content: String,
    pub tags: Vec<String>,
}

impl LogEntry {
    /// Built-in integrity check to prevent writing corrupt states
    pub fn is_valid(&self) -> bool {
        !self.id.is_empty() && self.timestamp > 0 && !self.content.is_empty()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub date: String, // Format: YYYY-MM-DD
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct LogCategories {
    pub achievements: Vec<LogEntry>,
    pub mistakes: Vec<LogEntry>,
    pub learnings: Vec<LogEntry>,
    pub ideas: Vec<LogEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DailyLog {
    pub metadata: Metadata,
    pub logs: LogCategories,
}

impl DailyLog {
    /// Helper to validate the entire daily log before serialization
    pub fn is_valid(&self) -> bool {
        if self.metadata.date.is_empty() || self.metadata.updated_at <= 0 {
            return false;
        }

        // Validate all categories
        for entry in &self.logs.achievements {
            if !entry.is_valid() {
                return false;
            }
        }
        for entry in &self.logs.mistakes {
            if !entry.is_valid() {
                return false;
            }
        }
        for entry in &self.logs.learnings {
            if !entry.is_valid() {
                return false;
            }
        }
        for entry in &self.logs.ideas {
            if !entry.is_valid() {
                return false;
            }
        }

        true
    }
}