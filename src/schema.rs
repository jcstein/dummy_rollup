use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use thiserror::Error;

/// Errors that can occur during database operations
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Record not found: {0}")]
    RecordNotFound(String),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Celestia error: {0}")]
    CelestiaError(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
}

/// Result type for database operations
pub type DatabaseResult<T> = Result<T, DatabaseError>;

/// Represents a record in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    /// Unique identifier for the record
    pub id: String,
    /// Timestamp when the record was created
    pub created_at: DateTime<Utc>,
    /// Timestamp of the last update
    pub updated_at: DateTime<Utc>,
    /// The actual data stored in the record
    pub data: Vec<u8>,
}

impl Record {
    /// Creates a new record with the given data
    pub fn new(data: Vec<u8>) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            created_at: now,
            updated_at: now,
            data,
        }
    }

    /// Updates the record's data and updated_at timestamp
    pub fn update(&mut self, data: Vec<u8>) {
        self.data = data;
        self.updated_at = Utc::now();
    }
}

/// Metadata for the database, stored in the first blob
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseMetadata {
    /// Number of records in the database
    pub record_count: u64,
    /// Mapping of record IDs to their blob heights
    pub index: std::collections::HashMap<String, u64>,
    /// Set of deleted record IDs
    pub deleted: std::collections::HashSet<String>,
    /// Last update timestamp
    pub last_updated: DateTime<Utc>,
}

impl DatabaseMetadata {
    /// Creates new empty metadata
    pub fn new() -> Self {
        Self {
            record_count: 0,
            index: std::collections::HashMap::new(),
            deleted: std::collections::HashSet::new(),
            last_updated: Utc::now(),
        }
    }

    /// Adds a record to the metadata
    pub fn add_record(&mut self, record_id: String, height: u64) {
        self.record_count += 1;
        self.index.insert(record_id, height);
        self.last_updated = Utc::now();
    }

    /// Marks a record as deleted
    pub fn delete_record(&mut self, record_id: &str) -> bool {
        if self.index.remove(record_id).is_some() {
            self.record_count -= 1;
            self.deleted.insert(record_id.to_string());
            self.last_updated = Utc::now();
            true
        } else {
            false
        }
    }

    /// Checks if a record exists and is not deleted
    pub fn record_exists(&self, record_id: &str) -> bool {
        self.index.contains_key(record_id) && !self.deleted.contains(record_id)
    }

    /// Gets the height for a record if it exists and is not deleted
    pub fn get_record_height(&self, record_id: &str) -> Option<u64> {
        if self.deleted.contains(record_id) {
            None
        } else {
            self.index.get(record_id).copied()
        }
    }
} 