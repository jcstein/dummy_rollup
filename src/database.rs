use celestia_rpc::{BlobClient, Client, HeaderClient};
use celestia_types::{nmt::Namespace, Blob, AppVersion};
use serde_json;
use std::collections::HashMap;

use crate::schema::{DatabaseError, DatabaseMetadata, Record};

pub struct DatabaseClient {
    client: Client,
    namespace: Namespace,
    metadata: DatabaseMetadata,
}

// Helper function to get current timestamp for logging
fn log_with_timestamp(message: &str) {
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] {}", timestamp, message);
}

impl DatabaseClient {
    pub async fn new(client: Client, namespace_bytes: Vec<u8>, start_height: Option<u64>) -> Result<Self, DatabaseError> {
        if namespace_bytes.len() != 8 {
            return Err(DatabaseError::InvalidNamespace("Namespace must be exactly 8 bytes".to_string()));
        }

        let namespace = Namespace::new(0, &namespace_bytes)
            .map_err(|e| DatabaseError::InvalidNamespace(e.to_string()))?;

        let mut db_client = Self {
            client,
            namespace,
            metadata: DatabaseMetadata::default(),
        };

        // Load or create metadata
        db_client.metadata = db_client.load_or_create_metadata(start_height).await?;

        Ok(db_client)
    }

    async fn load_or_create_metadata(&self, start_height: Option<u64>) -> Result<DatabaseMetadata, DatabaseError> {
        // Get the latest height from the Celestia client
        let latest_height = self.client.header_local_head()
            .await
            .map_err(|e| DatabaseError::CelestiaError(e.to_string()))?
            .height()
            .value();

        // If a specific start height was provided, check if there's metadata at that height first
        if let Some(height) = start_height {
            log_with_timestamp(&format!("Checking for existing metadata at height {}", height));
            match self.get_blobs_at_height(height).await {
                Ok(blobs) => {
                    for blob in blobs {
                        // Try to parse as metadata
                        if let Ok(metadata) = serde_json::from_slice::<DatabaseMetadata>(&blob.data) {
                            log_with_timestamp(&format!("Found existing metadata at height {}", height));
                            return Ok(metadata);
                        }
                    }
                }
                Err(_) => {
                    log_with_timestamp(&format!("No metadata found at height {}, will create new", height));
                },
            }
            
            // No metadata found at the specified height, create new with the specified start height
            let metadata = DatabaseMetadata {
                start_height: height,
                record_count: 0,
                last_updated: chrono::Utc::now(),
            };

            // Save the new metadata
            log_with_timestamp(&format!("Creating new metadata with start height {}", height));
            self.save_metadata(&metadata).await?;

            return Ok(metadata);
        }

        // Try to find existing metadata by searching in recent blocks
        log_with_timestamp("Searching for existing metadata in recent blocks");
        
        // Search in the most recent 1000 blocks or from height 1 if less than 1000
        let search_start_height = if latest_height > 1000 { latest_height - 1000 } else { 1 };
        
        for height in (search_start_height..=latest_height).rev() {
            match self.get_blobs_at_height(height).await {
                Ok(blobs) => {
                    for blob in blobs {
                        // Try to parse as metadata
                        if let Ok(metadata) = serde_json::from_slice::<DatabaseMetadata>(&blob.data) {
                            log_with_timestamp(&format!("Found existing metadata at height {}", height));
                            return Ok(metadata);
                        }
                    }
                }
                Err(_) => continue, // Error retrieving blobs, try next height
            }
        }

        // No metadata found, create new
        let metadata = DatabaseMetadata {
            start_height: latest_height,
            record_count: 0,
            last_updated: chrono::Utc::now(),
        };

        // Save the new metadata
        log_with_timestamp("No existing metadata found, creating new");
        self.save_metadata(&metadata).await?;

        Ok(metadata)
    }

    async fn save_metadata(&self, metadata: &DatabaseMetadata) -> Result<(), DatabaseError> {
        let metadata_json = serde_json::to_vec(metadata)
            .map_err(|e| DatabaseError::SerializationError(e.to_string()))?;

        let blob = Blob::new(
            self.namespace.clone(),
            metadata_json,
            AppVersion::V2,
        ).map_err(|e| DatabaseError::CelestiaError(e.to_string()))?;

        self.client.blob_submit(&[blob], Default::default())
            .await
            .map_err(|e| DatabaseError::CelestiaError(e.to_string()))?;

        Ok(())
    }

    async fn get_blobs_at_height(&self, height: u64) -> Result<Vec<Blob>, DatabaseError> {
        self.client.blob_get_all(height, &[self.namespace.clone()])
            .await
            .map_err(|e| DatabaseError::CelestiaError(e.to_string()))?
            .ok_or_else(|| DatabaseError::CelestiaError("No blobs found".to_string()))
    }

    pub async fn add_record(&self, record: Record) -> Result<(), DatabaseError> {
        let record_json = serde_json::to_vec(&record)
            .map_err(|e| DatabaseError::SerializationError(e.to_string()))?;

        let blob = Blob::new(
            self.namespace.clone(),
            record_json,
            AppVersion::V2,
        ).map_err(|e| DatabaseError::CelestiaError(e.to_string()))?;

        self.client.blob_submit(&[blob], Default::default())
            .await
            .map_err(|e| DatabaseError::CelestiaError(e.to_string()))?;

        Ok(())
    }

    pub async fn get_record(&self, key: &str) -> Result<Option<Record>, DatabaseError> {
        let latest_height = self.client.header_local_head()
            .await
            .map_err(|e| DatabaseError::CelestiaError(e.to_string()))?
            .height()
            .value();

        let start_height = self.metadata.start_height;
        log_with_timestamp(&format!("Searching for record with key '{}' (start height: {})", key, start_height));
        
        // Search from the start height to the latest height
        for height in (start_height..=latest_height).rev() {
            match self.get_blobs_at_height(height).await {
                Ok(blobs) => {
                    for blob in blobs {
                        // Try to parse as record
                        if let Ok(record) = serde_json::from_slice::<Record>(&blob.data) {
                            if record.key == key {
                                log_with_timestamp(&format!("Found record with key '{}' at height {}", key, height));
                                return Ok(Some(record));
                            }
                        }
                    }
                }
                Err(_) => continue, // Error retrieving blobs, try next height
            }
        }

        log_with_timestamp(&format!("No record found with key '{}'", key));
        Ok(None)
    }

    pub async fn list_records(&self) -> Result<Vec<Record>, DatabaseError> {
        let latest_height = self.client.header_local_head()
            .await
            .map_err(|e| DatabaseError::CelestiaError(e.to_string()))?
            .height()
            .value();

        let start_height = self.metadata.start_height;
        log_with_timestamp(&format!("Listing all records (start height: {})", start_height));
        
        let mut records_map: HashMap<String, Record> = HashMap::new();
        
        // Search from the start height to the latest height
        for height in (start_height..=latest_height).rev() {
            match self.get_blobs_at_height(height).await {
                Ok(blobs) => {
                    for blob in blobs {
                        // Try to parse as record
                        if let Ok(record) = serde_json::from_slice::<Record>(&blob.data) {
                            // Only add if we haven't seen this key before (since we're going backwards)
                            if !records_map.contains_key(&record.key) {
                                records_map.insert(record.key.clone(), record);
                            }
                        }
                    }
                }
                Err(_) => continue, // Error retrieving blobs, try next height
            }
        }

        log_with_timestamp(&format!("Found {} records", records_map.len()));
        Ok(records_map.into_values().collect())
    }
} 