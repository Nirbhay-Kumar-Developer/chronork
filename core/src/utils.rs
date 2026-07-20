use chrono::{Local, Datelike};
use std::time::{SystemTime, UNIX_EPOCH};

// --- Date, Time, and ID Helpers ---

pub fn get_current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

pub fn generate_id(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    
    format!("{}_{}", prefix, nanos)
}

pub fn get_today_date() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

pub fn get_current_year() -> String {
    Local::now().format("%Y").to_string()
}

// FIX: Handled calendar math within core so the CLI doesn't need to link chrono directly
pub fn get_last_day_of_month(year: i32, month: u32) -> u32 {
    if month == 12 {
        31
    } else {
        chrono::NaiveDate::from_ymd_opt(year, month + 1, 1)
            .and_then(|d| d.pred_opt())
            .map(|d| d.day())
            .unwrap_or(31)
    }
}