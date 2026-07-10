use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::models::{DailyLog, LogCategories, Metadata};

pub struct StorageManager;

impl StorageManager {
    // Resolves the root directory natively in Termux
    fn get_base_path() -> String {
        match env::var("PREFIX") {
            Ok(prefix) => format!("{}/var/lib/chronork/logs/", prefix),
            Err(_) => "/var/lib/chronork/logs/".to_string(),
        }
    }

    // Constructs the exact file path based on the YYYY-MM-DD date string
    fn resolve_file_path(date_str: &str) -> Result<PathBuf, String> {
        if date_str.len() < 10 {
            return Err("Date string must be in YYYY-MM-DD format.".to_string());
        }

        let year = &date_str[0..4];
        let month = &date_str[5..7];

        let mut path = PathBuf::from(Self::get_base_path());
        path.push(year);
        path.push(month);
        path.push(format!("{}.json", date_str));
        
        Ok(path)
    }

    // Applies strict 600 permissions (Owner Read/Write only)
    fn secure_file_permissions(target: &Path) -> Result<(), String> {
        let metadata = fs::metadata(target)
            .map_err(|e| format!("Failed to read metadata for permissions: {}", e))?;
        
        let mut perms = metadata.permissions();
        perms.set_mode(0o600);
        
        fs::set_permissions(target, perms)
            .map_err(|e| format!("Failed to set permissions: {}", e))
    }

    /// Loads a daily log from disk. Returns a new empty log if it doesn't exist.
    pub fn load(date_str: &str) -> Result<DailyLog, String> {
        let target_path = Self::resolve_file_path(date_str)?;

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
            .map_err(|e| format!("Failed to open file for reading {}: {}", target_path.display(), e))?;

        let log: DailyLog = serde_json::from_str(&file_content)
            .map_err(|e| format!("JSON Parse Error in {}: {}", target_path.display(), e))?;

        Ok(log)
    }

    /// Safely serializes and atomically writes a daily log to disk.
    pub fn save(log: &DailyLog) -> Result<(), String> {
        // 1. Data Integrity Check before touching the filesystem
        if !log.is_valid() {
            return Err("DailyLog validation failed. Aborting write to prevent data corruption.".to_string());
        }

        let target_path = Self::resolve_file_path(&log.metadata.date)?;
        let tmp_path_str = format!("{}.tmp", target_path.display());
        let tmp_path = Path::new(&tmp_path_str);

        // 2. Ensure parent directories exist
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directories: {}", e))?;
        }

        // 3. Write to a temporary file
        // A block is used here to ensure the File handle goes out of scope and closes automatically.
        {
            let mut tmp_file = File::create(tmp_path)
                .map_err(|e| format!("Failed to create temporary write file {}: {}", tmp_path.display(), e))?;

            // 4 spaces for readability, matching j.dump(4)
            let json_string = serde_json::to_string_pretty(log)
                .map_err(|e| format!("JSON Serialization Error: {}", e))?;

            tmp_file.write_all(json_string.as_bytes())
                .map_err(|e| format!("Failed to write to temporary file: {}", e))?;

            // 4. Force OS to flush buffer to physical disk
            tmp_file.sync_all()
                .map_err(|e| format!("Failed to force flush to disk: {}", e))?;
        } // File closes here securely.

        // 5. Secure the temp file before making it live
        Self::secure_file_permissions(tmp_path)?;

        // 6. Atomic swap: Overwrite the old file seamlessly
        // If power fails exactly here, the old file remains perfectly intact.
        fs::rename(tmp_path, target_path)
            .map_err(|e| format!("Atomic swap failed: {}", e))?;

        Ok(())
    }
}