use serde::{Deserialize, Serialize};

// --- Data Structures ---

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: String,
    pub timestamp: i64,
    pub content: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryFilter {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
}

impl LogEntry {
    /// Built-in integrity check to prevent writing corrupt states
    pub fn is_valid(&self) -> bool {
        // 1. Core strings cannot be empty
        if self.id.is_empty() || self.content.is_empty() {
            return false;
        }

        // 2. Enforce a realistic Unix epoch second window (Year 2000 to Year 2100)
        if self.timestamp < 946684800 || self.timestamp > 4102444800 {
            return false;
        }

        // 3. Strict schema validation for the unique ID: `{prefix}_{digits}`
        let id_bytes = self.id.as_bytes();
        if let Some(underscore_pos) = id_bytes.iter().position(|&b| b == b'_') {
            // The separator cannot be the first or last character
            if underscore_pos == 0 || underscore_pos == id_bytes.len() - 1 {
                return false;
            }
            // Verify all trailing characters after the prefix separator are numeric ticks
            id_bytes[underscore_pos + 1..].iter().all(|b| b.is_ascii_digit())
        } else {
            false
        }
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
        // 1. Validate exact YYYY-MM-DD schema string constraints to secure database layouts
        if self.metadata.date.len() != 10 {
            return false;
        }
        
        let date_bytes = self.metadata.date.as_bytes();
        let is_valid_date_format = date_bytes[0..4].iter().all(|c| c.is_ascii_digit())
            && date_bytes[4] == b'-'
            && date_bytes[5..7].iter().all(|c| c.is_ascii_digit())
            && date_bytes[7] == b'-'
            && date_bytes[8..10].iter().all(|c| c.is_ascii_digit());

        if !is_valid_date_format {
            return false;
        }

        // 2. Enforce a realistic Unix epoch second window for the modification marker (Year 2000 to Year 2100)
        if self.metadata.updated_at < 946684800 || self.metadata.updated_at > 4102444800 {
            return false;
        }

        // 3. Deep validation tracking across all category records
        self.logs.achievements.iter().all(|entry| entry.is_valid())
            && self.logs.mistakes.iter().all(|entry| entry.is_valid())
            && self.logs.learnings.iter().all(|entry| entry.is_valid())
            && self.logs.ideas.iter().all(|entry| entry.is_valid())
    }
}