use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::jstring;
use std::path::PathBuf;

use chronork_core::models::{DailyLog, QueryFilter};
use chronork_core::storage::StorageManager;

/// Retrieves a log for a specific date and returns it to Java as a serialized JSON string.
#[no_mangle]
pub extern "system" fn Java_com_chronork_getLog<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    app_files_dir: JString<'local>,
    date: JString<'local>,
) -> jstring {
    // 1. Convert Java strings to Rust strings safely
    let base_path_str: String = env.get_string(&app_files_dir).expect("Invalid base path string").into();
    let date_str: String = env.get_string(&date).expect("Invalid date string").into();

    // 2. Initialize the Core StorageManager with the Android sandbox path
    let manager = StorageManager::new(PathBuf::from(base_path_str));

    // 3. Execute logic and handle CoreErrors
    let json_response = match manager.load(&date_str) {
        Ok(log) => {
            // Serialize the Rust struct back to a JSON string for the JVM
            serde_json::to_string(&log).unwrap_or_else(|_| "{\"error\": \"Failed to serialize log\"}".to_string())
        }
        Err(e) => {
            format!("{{\"error\": \"{}\"}}", e)
        }
    };

    // 4. Allocate a new Java string and return its raw pointer
    let output = env.new_string(json_response).expect("Failed to create Java string");
    output.into_raw()
}

/// Consumes a JSON string from Java, deserializes it into a Rust struct, and saves it to disk safely.
#[no_mangle]
pub extern "system" fn Java_com_chronork_storeLog<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    app_files_dir: JString<'local>,
    log_payload: JString<'local>,
) -> jstring {
    // 1. Convert Java strings to Rust strings
    let base_path_str: String = env.get_string(&app_files_dir).expect("Invalid base path string").into();
    let payload_str: String = env.get_string(&log_payload).expect("Invalid payload string").into();

    // 2. Initialize Core
    let manager = StorageManager::new(PathBuf::from(base_path_str));

    // 3. Deserialize JSON payload directly into the Chronork DailyLog struct
    let json_response = match serde_json::from_str::<DailyLog>(&payload_str) {
        Ok(log) => {
            // 4. Save via the atomic StorageManager
            match manager.save(&log) {
                Ok(_) => "{\"status\": \"success\"}".to_string(),
                Err(e) => format!("{{\"status\": \"error\", \"message\": \"{}\"}}", e),
            }
        }
        Err(e) => {
            format!("{{\"status\": \"error\", \"message\": \"JSON Parse Error: {}\"}}", e)
        }
    };

    // 5. Return status back to JVM
    let output = env.new_string(json_response).expect("Failed to create Java string");
    output.into_raw()
}

/// Consumes a JSON string from Java representing a QueryFilter, scans the filesystem,
/// and returns a JSON array of matching DailyLogs.
#[no_mangle]
pub extern "system" fn Java_com_chronork_queryLogs<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    app_files_dir: JString<'local>,
    query_payload: JString<'local>,
) -> jstring {
    // 1. Convert Java strings to Rust strings
    let base_path_str: String = env.get_string(&app_files_dir).expect("Invalid base path string").into();
    let payload_str: String = env.get_string(&query_payload).expect("Invalid query payload string").into();

    // 2. Initialize Core
    let manager = StorageManager::new(PathBuf::from(base_path_str));

    // 3. Deserialize JSON payload into QueryFilter
    let json_response = match serde_json::from_str::<QueryFilter>(&payload_str) {
        Ok(filter) => {
            // 4. Scan the directory range using the core indexing engine
            match manager.scan_range(&filter) {
                Ok(logs) => {
                    // 5. Serialize the resulting Vec<DailyLog> back to a JSON array string
                    serde_json::to_string(&logs).unwrap_or_else(|_| "{\"error\": \"Failed to serialize logs array\"}".to_string())
                }
                Err(e) => format!("{{\"error\": \"{}\"}}", e),
            }
        }
        Err(e) => {
            format!("{{\"error\": \"JSON Parse Error: {}\"}}", e)
        }
    };

    // 6. Return the JSON array (or error object) back to the JVM
    let output = env.new_string(json_response).expect("Failed to create Java string");
    output.into_raw()
}