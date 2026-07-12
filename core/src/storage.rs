use std::fs::{self, File};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::ChronorkError;
use crate::models::{DailyLog, LogCategories, Metadata, QueryFilter};

pub struct StorageManager {
    base_path: PathBuf,
}

impl StorageManager {
    /// Initializes the StorageManager with a dynamic path provided by the environment 
    /// (e.g., CLI parsing or Android JNI context).
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    // Constructs the exact file path based on the YYYY-MM-DD date string
    fn resolve_file_path(&self, date_str: &str) -> Result<PathBuf, ChronorkError> {
        // Enforce length and ASCII boundaries to prevent byte-slice panics
        if date_str.len() < 10 || !date_str.is_ascii() {
            return Err(ChronorkError::Validation(
                "Date string must be valid ASCII in YYYY-MM-DD format.".to_string(),
            ));
        }

        let year = &date_str[0..4];
        let month = &date_str[5..7];

        let mut path = self.base_path.clone();
        path.push(year);
        path.push(month);
        path.push(format!("{}.json", date_str));
        
        Ok(path)
    }

    // Applies strict 600 permissions (Owner Read/Write only)
    fn secure_file_permissions(target: &Path) -> Result<(), ChronorkError> {
        let metadata = fs::metadata(target)
            .map_err(|e| ChronorkError::FileSystem(format!("Failed to read metadata for permissions: {}", e)))?;
        
        let mut perms = metadata.permissions();
        perms.set_mode(0o600);
        
        fs::set_permissions(target, perms)
            .map_err(|e| ChronorkError::FileSystem(format!("Failed to set permissions: {}", e)))
    }

    /// Loads a daily log from disk. Returns a new empty log if it doesn't exist.
    pub fn load(&self, date_str: &str) -> Result<DailyLog, ChronorkError> {
        let target_path = self.resolve_file_path(date_str)?;

        if !target_path.exists() {
            return Ok(DailyLog {
                metadata: Metadata {
                    date: date_str.to_string(),
                    updated_at: 0,
                },
                logs: LogCategories::default(),
            });
        }

        let file_content = fs::read_to_string(&target_path)
            .map_err(|e| ChronorkError::FileSystem(format!("Failed to open file for reading {}: {}", target_path.display(), e)))?;

        let log: DailyLog = serde_json::from_str(&file_content)
            .map_err(ChronorkError::Serialization)?;

        Ok(log)
    }

    /// Safely serializes and atomically writes a daily log to disk.
    pub fn save(&self, log: &DailyLog) -> Result<(), ChronorkError> {
        // 1. Data Integrity Check before touching the filesystem
        if !log.is_valid() {
            return Err(ChronorkError::Validation(
                "DailyLog validation failed. Aborting write to prevent data corruption.".to_string(),
            ));
        }

        let target_path = self.resolve_file_path(&log.metadata.date)?;
        let tmp_path_str = format!("{}.tmp", target_path.display());
        let tmp_path = Path::new(&tmp_path_str);

        // 2. Ensure parent directories exist
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| ChronorkError::FileSystem(format!("Failed to create directories: {}", e)))?;
        }

        // 3. Write to a temporary file
        {
            let mut tmp_file = File::create(tmp_path)
                .map_err(|e| ChronorkError::FileSystem(format!("Failed to create temporary write file {}: {}", tmp_path.display(), e)))?;

            let json_string = serde_json::to_string_pretty(log)
                .map_err(ChronorkError::Serialization)?;

            tmp_file.write_all(json_string.as_bytes())
                .map_err(|e| ChronorkError::FileSystem(format!("Failed to write to temporary file: {}", e)))?;

            // 4. Force OS to flush buffer to physical disk
            tmp_file.sync_all()
                .map_err(|e| ChronorkError::FileSystem(format!("Failed to force flush to disk: {}", e)))?;
        } 

        // 5. Secure the temp file before making it live
        Self::secure_file_permissions(tmp_path)?;

        // 6. Atomic swap: Overwrite the old file seamlessly
        fs::rename(tmp_path, target_path)
            .map_err(|e| ChronorkError::FileSystem(format!("Atomic swap failed: {}", e)))?;

        Ok(())
    }

    /// Recursively scans directories to retrieve logs within a date range, applying central filters.
    pub fn scan_range(&self, filter: &QueryFilter) -> Result<Vec<DailyLog>, ChronorkError> {
        let mut results = Vec::new();

        // Normalize filters for case-insensitive matching
        let target_cats: Vec<String> = filter.categories.iter().map(|c| c.to_lowercase()).collect();
        let target_tags: Vec<String> = filter.tags.iter().map(|t| t.to_lowercase()).collect();

        let year_dirs = match fs::read_dir(&self.base_path) {
            Ok(dirs) => dirs,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(results),
            Err(e) => return Err(ChronorkError::FileSystem(format!("Failed to read base path: {}", e))),
        };

        // Traverse Years (Pruning irrelevant directories to optimize I/O)
        for year_entry in year_dirs.flatten() {
            let year_path = year_entry.path();
            if !year_path.is_dir() { continue; }
            
            let year_str = year_entry.file_name().to_string_lossy().to_string();

            // Lexicographical bounds checking for YYYY
            if let Some(start) = &filter.start_date {
                if let Some(sy) = start.get(0..4) { if year_str.as_str() < sy { continue; } }
            }
            if let Some(end) = &filter.end_date {
                if let Some(ey) = end.get(0..4) { if year_str.as_str() > ey { continue; } }
            }

            let month_dirs = match fs::read_dir(&year_path) {
                Ok(dirs) => dirs,
                Err(_) => continue,
            };

            // Traverse Months (Pruning irrelevant directories)
            for month_entry in month_dirs.flatten() {
                let month_path = month_entry.path();
                if !month_path.is_dir() { continue; }
                
                let month_str = month_entry.file_name().to_string_lossy().to_string();
                let ym_str = format!("{}-{}", year_str, month_str);

                // Lexicographical bounds checking for YYYY-MM
                if let Some(start) = &filter.start_date {
                    if let Some(sym) = start.get(0..7) { if ym_str.as_str() < sym { continue; } }
                }
                if let Some(end) = &filter.end_date {
                    if let Some(eym) = end.get(0..7) { if ym_str.as_str() > eym { continue; } }
                }

                let file_entries = match fs::read_dir(&month_path) {
                    Ok(files) => files,
                    Err(_) => continue,
                };

                // Traverse Files (Exact Date Check & Filtering)
                for file_entry in file_entries.flatten() {
                    let file_path = file_entry.path();
                    if !file_path.is_file() || file_path.extension().unwrap_or_default() != "json" {
                        continue;
                    }

                    let date_str = file_path.file_stem().unwrap_or_default().to_string_lossy().to_string();

                    // Lexicographical bounds checking for YYYY-MM-DD
                    if let Some(start) = &filter.start_date { if date_str < *start { continue; } }
                    if let Some(end) = &filter.end_date { if date_str > *end { continue; } }

                    // Load file and apply node filtering
                    if let Ok(mut log) = self.load(&date_str) {
                        if log.metadata.updated_at == 0 { continue; }

                        // 1. Category Filtering
                        if !target_cats.is_empty() {
                            if !target_cats.contains(&"achievements".to_string()) { log.logs.achievements.clear(); }
                            if !target_cats.contains(&"learnings".to_string()) { log.logs.learnings.clear(); }
                            if !target_cats.contains(&"mistakes".to_string()) { log.logs.mistakes.clear(); }
                            if !target_cats.contains(&"ideas".to_string()) { log.logs.ideas.clear(); }
                        }

                        // 2. Tag Filtering (In-place retention)
                        if !target_tags.is_empty() {
                            let tag_match = |entry: &crate::models::LogEntry| -> bool {
                                entry.tags.iter().any(|t| target_tags.contains(&t.to_lowercase()))
                            };
                            
                            log.logs.achievements.retain(tag_match);
                            log.logs.learnings.retain(tag_match);
                            log.logs.mistakes.retain(tag_match);
                            log.logs.ideas.retain(tag_match);
                        }

                        // 3. Drop empty days
                        if !log.logs.achievements.is_empty() 
                            || !log.logs.learnings.is_empty() 
                            || !log.logs.mistakes.is_empty() 
                            || !log.logs.ideas.is_empty() {
                            results.push(log);
                        }
                    }
                }
            }
        }

        // Enforce strict chronological order before returning to environment wrappers
        results.sort_by(|a, b| a.metadata.date.cmp(&b.metadata.date));
        
        Ok(results)
    }
}