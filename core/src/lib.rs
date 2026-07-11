//! Chronork Core Library
//!
//! This crate contains the environment-agnostic domain logic, data models,
//! and secure storage mechanisms for Chronork.

pub mod models;
pub mod storage;
pub mod utils;

use std::fmt;
use std::io;

/// A unified error type for all core operations.
/// This allows the CLI and JNI wrappers to handle failures natively.
#[derive(Debug)]
pub enum ChronorkError {
    Io(io::Error),
    Serialization(serde_json::Error),
    Validation(String),
    FileSystem(String),
}

impl fmt::Display for ChronorkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChronorkError::Io(e) => write!(f, "I/O Error: {}", e),
            ChronorkError::Serialization(e) => write!(f, "JSON Error: {}", e),
            ChronorkError::Validation(msg) => write!(f, "Validation Failed: {}", msg),
            ChronorkError::FileSystem(msg) => write!(f, "Storage Error: {}", msg),
        }
    }
}

impl std::error::Error for ChronorkError {}

// Implement standard From conversions to make using `?` in your storage.rs seamless
impl From<io::Error> for ChronorkError {
    fn from(error: io::Error) -> Self {
        ChronorkError::Io(error)
    }
}

impl From<serde_json::Error> for ChronorkError {
    fn from(error: serde_json::Error) -> Self {
        ChronorkError::Serialization(error)
    }
}