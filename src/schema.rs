use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur during database operations
#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Celestia error: {0}")]
    CelestiaError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Invalid namespace: {0}")]
    InvalidNamespace(String),
}

/// Represents a record in the database
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Record {
    /// The actual data stored in the record
    pub key: String,
    pub value: String,
    /// Timestamp when the record was created
    pub created_at: DateTime<Utc>,
    /// Timestamp of the last update
    pub updated_at: Option<DateTime<Utc>>,
    /// Unique identifier for the record
    pub id: String,
}

impl Record {
    /// Creates a new record with the given data
    pub fn new(key: String, value: String) -> Self {
        Self {
            key,
            value,
            created_at: Utc::now(),
            updated_at: None,
            id: Uuid::new_v4().to_string(),
        }
    }
}

/// Metadata for the database, stored in the first blob
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseMetadata {
    /// Number of records in the database
    pub record_count: u64,
    /// Last update timestamp
    pub last_updated: DateTime<Utc>,
    /// Start height of the database
    pub start_height: u64,
}

impl Default for DatabaseMetadata {
    fn default() -> Self {
        Self {
            record_count: 0,
            last_updated: Utc::now(),
            start_height: 1,
        }
    }
} 