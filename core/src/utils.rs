use chrono::Local;
use std::time::{SystemTime, UNIX_EPOCH};

// --- Date, Time, and ID Helpers ---

// Returns exact 64-bit UNIX epoch timestamp (seconds since 1970)
pub fn get_current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

// Generates a simple, robust unique ID based on the timestamp
pub fn generate_id(prefix: &str) -> String {
    format!("{}_{}", prefix, get_current_timestamp())
}

// Returns today's date strictly as "YYYY-MM-DD" for filesystem sorting
pub fn get_today_date() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

pub fn get_current_year() -> String {
    Local::now().format("%Y").to_string()
}