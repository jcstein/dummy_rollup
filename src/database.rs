use celestia_rpc::{BlobClient, Client, HeaderClient};
use celestia_types::{nmt::Namespace, Blob, AppVersion};
use serde_json;
use std::collections::HashMap;

use crate::schema::{DatabaseError, DatabaseMetadata, Record};

pub struct DatabaseClient {
    client: Client,
    namespace: Namespace,
    metadata: Option<DatabaseMetadata>,
    search_limit: Option<u64>,
}

// Helper function to get current timestamp for logging
fn log_with_timestamp(message: &str) {
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] {}", timestamp, message);
}

impl DatabaseClient {
    pub async fn new(
        client: Client, 
        namespace_bytes: Vec<u8>, 
        _start_height: Option<u64>, // Kept for API compatibility but ignored
        search_limit: Option<u64>
    ) -> Result<Self, DatabaseError> {
        if namespace_bytes.len() != 10 {
            return Err(DatabaseError::InvalidNamespace("Namespace must be exactly 10 bytes".to_string()));
        }

        let namespace = Namespace::new(0, &namespace_bytes)
            .map_err(|e| DatabaseError::InvalidNamespace(e.to_string()))?;

        let mut db_client = Self {
            client,
            namespace,
            metadata: None,
            search_limit,
        };

        // Try to discover existing database within search_limit blocks
        if let Some(metadata) = db_client.discover_database().await? {
            log_with_timestamp(&format!("Found existing database at height {}", metadata.start_height));
            db_client.metadata = Some(metadata);
        } else {
            // Create new database at current height
            let latest_height = db_client.client.header_local_head()
                .await
                .map_err(|e| DatabaseError::CelestiaError(e.to_string()))?
                .height()
                .value();
            
            let metadata = DatabaseMetadata {
                start_height: latest_height,
                record_count: 0,
                last_updated: chrono::Utc::now(),
            };
            
            log_with_timestamp(&format!("Creating new database (estimating height {})", latest_height));
            
            // Save metadata and get the actual inclusion height
            let actual_height = db_client.save_metadata(&metadata).await?;
            
            // Update metadata with actual height
            let updated_metadata = DatabaseMetadata {
                start_height: actual_height,
                record_count: 0,
                last_updated: chrono::Utc::now(),
            };
            
            log_with_timestamp(&format!("Database created at height {}", actual_height));
            db_client.metadata = Some(updated_metadata);
        }

        Ok(db_client)
    }

    async fn discover_database(&self) -> Result<Option<DatabaseMetadata>, DatabaseError> {
        let latest_height = self.client.header_local_head()
            .await
            .map_err(|e| DatabaseError::CelestiaError(e.to_string()))?
            .height()
            .value();
        
        // Determine search range based on search_limit
        let start_height = if let Some(limit) = self.search_limit {
            if latest_height > limit {
                latest_height - limit
            } else {
                1
            }
        } else {
            // If no search_limit, just check the most recent 100 blocks
            if latest_height > 100 {
                latest_height - 100
            } else {
                1
            }
        };
        
        log_with_timestamp(&format!("Searching for existing database (blocks {}..{})", start_height, latest_height));
        
        // Search for metadata
        for height in (start_height..=latest_height).rev() {
            match self.get_blobs_at_height(height).await {
                Ok(blobs) => {
                    for blob in blobs {
                        // Try to parse as metadata
                        if let Ok(metadata) = serde_json::from_slice::<DatabaseMetadata>(&blob.data) {
                            return Ok(Some(metadata));
                        }
                    }
                }
                Err(_) => continue,
            }
        }
        
        log_with_timestamp("No existing database found within search range");
        Ok(None)
    }
    
    async fn save_metadata(&self, metadata: &DatabaseMetadata) -> Result<u64, DatabaseError> {
        let metadata_json = serde_json::to_vec(metadata)
            .map_err(|e| DatabaseError::SerializationError(e.to_string()))?;

        let blob = Blob::new(
            self.namespace.clone(),
            metadata_json,
            AppVersion::V2,
        ).map_err(|e| DatabaseError::CelestiaError(e.to_string()))?;

        let response = self.client.blob_submit(&[blob], Default::default())
            .await
            .map_err(|e| DatabaseError::CelestiaError(e.to_string()))?;
            
        // Debug the response content
        log_with_timestamp(&format!("Submit response: {:?}", response));
        
        // Try to access height using different approaches
        let inclusion_height = if let Ok(serde_value) = serde_json::to_value(&response) {
            if let Some(height_value) = serde_value.get("height") {
                if let Some(height_num) = height_value.as_u64() {
                    Some(height_num)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };
        
        // Get the current height as fallback
        let current_height = self.client.header_local_head()
            .await
            .map_err(|e| DatabaseError::CelestiaError(e.to_string()))?
            .height()
            .value();
            
        // Use the inclusion height if available, otherwise use current height
        let final_height = inclusion_height.unwrap_or(current_height);
        log_with_timestamp(&format!("Using height: {}", final_height));
        
        Ok(final_height)
    }

    async fn get_blobs_at_height(&self, height: u64) -> Result<Vec<Blob>, DatabaseError> {
        self.client.blob_get_all(height, &[self.namespace.clone()])
            .await
            .map_err(|e| DatabaseError::CelestiaError(e.to_string()))?
            .ok_or_else(|| DatabaseError::CelestiaError("No blobs found".to_string()))
    }

    pub async fn add_record(&mut self, record: Record) -> Result<(), DatabaseError> {
        let mut blobs = Vec::new();

        // Prepare record blob
        let record_json = serde_json::to_vec(&record)
            .map_err(|e| DatabaseError::SerializationError(e.to_string()))?;
        let record_blob = Blob::new(
            self.namespace.clone(),
            record_json,
            AppVersion::V2,
        ).map_err(|e| DatabaseError::CelestiaError(e.to_string()))?;
        blobs.push(record_blob);

        // Prepare metadata blob if needed
        if let Some(metadata) = &self.metadata {
            let mut updated_metadata = metadata.clone();
            updated_metadata.record_count += 1;
            updated_metadata.last_updated = chrono::Utc::now();
            
            let metadata_json = serde_json::to_vec(&updated_metadata)
                .map_err(|e| DatabaseError::SerializationError(e.to_string()))?;
            let metadata_blob = Blob::new(
                self.namespace.clone(),
                metadata_json,
                AppVersion::V2,
            ).map_err(|e| DatabaseError::CelestiaError(e.to_string()))?;
            blobs.push(metadata_blob);

            // Update in-memory metadata
            self.metadata = Some(updated_metadata);
        }

        // Submit both blobs in a single transaction
        self.client.blob_submit(&blobs, Default::default())
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
        
        // Get start height from metadata if available
        let start_height = if let Some(metadata) = &self.metadata {
            metadata.start_height
        } else {
            1 // Fallback to beginning if no metadata (shouldn't happen)
        };
        
        log_with_timestamp(&format!(
            "Searching for record with key '{}' (database start: {}, current height: {})", 
            key, start_height, latest_height
        ));
        
        // Search from start height to the latest height
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
        log_with_timestamp("Listing all records");
        
        let latest_height = self.client.header_local_head()
            .await
            .map_err(|e| DatabaseError::CelestiaError(e.to_string()))?
            .height()
            .value();
        
        // Get start height from metadata if available
        let start_height = if let Some(metadata) = &self.metadata {
            metadata.start_height
        } else {
            1 // Fallback to beginning if no metadata (shouldn't happen)
        };
        
        log_with_timestamp(&format!(
            "Listing all records (database start: {}, current height: {})", 
            start_height, latest_height
        ));
        
        let mut records_map: HashMap<String, Record> = HashMap::new();
        
        // Search from start height to the latest height
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