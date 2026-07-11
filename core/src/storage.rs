use std::fs::{self, File};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::ChronorkError;
use crate::models::{DailyLog, LogCategories, Metadata};

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
        // A block is used here to ensure the File handle goes out of scope and closes automatically.
        {
            let mut tmp_file = File::create(tmp_path)
                .map_err(|e| ChronorkError::FileSystem(format!("Failed to create temporary write file {}: {}", tmp_path.display(), e)))?;

            // 4 spaces for readability
            let json_string = serde_json::to_string_pretty(log)
                .map_err(ChronorkError::Serialization)?;

            tmp_file.write_all(json_string.as_bytes())
                .map_err(|e| ChronorkError::FileSystem(format!("Failed to write to temporary file: {}", e)))?;

            // 4. Force OS to flush buffer to physical disk
            tmp_file.sync_all()
                .map_err(|e| ChronorkError::FileSystem(format!("Failed to force flush to disk: {}", e)))?;
        } // File closes here securely.

        // 5. Secure the temp file before making it live
        Self::secure_file_permissions(tmp_path)?;

        // 6. Atomic swap: Overwrite the old file seamlessly
        // If power fails exactly here, the old file remains perfectly intact.
        fs::rename(tmp_path, target_path)
            .map_err(|e| ChronorkError::FileSystem(format!("Atomic swap failed: {}", e)))?;

        Ok(())
    }
}