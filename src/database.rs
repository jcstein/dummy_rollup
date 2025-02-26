use crate::schema::{DatabaseError, DatabaseMetadata, DatabaseResult, Record};
use async_trait::async_trait;
use celestia_rpc::{BlobClient, Client, HeaderClient, TxConfig};
use celestia_types::{nmt::Namespace, Blob, AppVersion};
use serde_json;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Trait defining the database operations
#[async_trait]
pub trait Database {
    /// Creates a new record in the database
    async fn create_record(&self, data: Vec<u8>) -> DatabaseResult<String>;
    
    /// Reads a record from the database
    async fn read_record(&self, id: &str) -> DatabaseResult<Record>;
    
    /// Updates an existing record
    async fn update_record(&self, id: &str, data: Vec<u8>) -> DatabaseResult<()>;
    
    /// Deletes a record from the database
    async fn delete_record(&self, id: &str) -> DatabaseResult<()>;
    
    /// Lists all records in the database
    async fn list_records(&self) -> DatabaseResult<Vec<Record>>;
}

/// The main database client
pub struct DatabaseClient {
    client: Client,
    namespace: Namespace,
    metadata: Arc<Mutex<DatabaseMetadata>>,
}

impl DatabaseClient {
    /// Creates a new database client
    pub async fn new(client: Client, namespace: Namespace) -> DatabaseResult<Self> {
        let metadata = Self::load_or_create_metadata(&client, &namespace).await?;
        Ok(Self {
            client,
            namespace,
            metadata: Arc::new(Mutex::new(metadata)),
        })
    }

    /// Loads existing metadata or creates new if none exists
    async fn load_or_create_metadata(client: &Client, namespace: &Namespace) -> DatabaseResult<DatabaseMetadata> {
        // Try to get the latest height
        let header = client
            .header_local_head()
            .await
            .map_err(|e| DatabaseError::CelestiaError(e.to_string()))?;
        
        let height = header.height().value();
        
        // Search for metadata blob in recent blocks
        for h in (height.saturating_sub(100)..=height).rev() {
            if let Ok(blobs) = client.blob_get_all(h, &[namespace.clone()]).await {
                if let Some(blobs) = blobs {
                    for blob in blobs {
                        if let Ok(metadata) = serde_json::from_slice::<DatabaseMetadata>(&blob.data) {
                            return Ok(metadata);
                        }
                    }
                }
            }
        }
        
        // No metadata found, create new
        Ok(DatabaseMetadata::new())
    }

    /// Saves the current metadata to a blob
    async fn save_metadata(&self) -> DatabaseResult<u64> {
        let metadata = self.metadata.lock().await;
        let data = serde_json::to_vec(&*metadata).map_err(DatabaseError::SerializationError)?;
        
        let blob = Blob::new(self.namespace.clone(), data, AppVersion::V2)
            .map_err(|e| DatabaseError::DatabaseError(e.to_string()))?;
        
        self.client
            .blob_submit(&[blob], TxConfig::default())
            .await
            .map_err(|e| DatabaseError::CelestiaError(e.to_string()))
    }

    /// Retrieves a blob at a specific height
    async fn get_blob(&self, height: u64) -> DatabaseResult<Option<Blob>> {
        let blobs = self
            .client
            .blob_get_all(height, &[self.namespace.clone()])
            .await
            .map_err(|e| DatabaseError::CelestiaError(e.to_string()))?;
        
        Ok(blobs.and_then(|mut b| b.pop()))
    }
}

#[async_trait]
impl Database for DatabaseClient {
    async fn create_record(&self, data: Vec<u8>) -> DatabaseResult<String> {
        let record = Record::new(data);
        let record_id = record.id.clone();
        
        // Create and submit blob
        let blob = Blob::new(
            self.namespace.clone(),
            serde_json::to_vec(&record).map_err(DatabaseError::SerializationError)?,
            AppVersion::V2,
        )
        .map_err(|e| DatabaseError::DatabaseError(e.to_string()))?;
        
        let height = self
            .client
            .blob_submit(&[blob], TxConfig::default())
            .await
            .map_err(|e| DatabaseError::CelestiaError(e.to_string()))?;
        
        // Update metadata
        {
            let mut metadata = self.metadata.lock().await;
            metadata.add_record(record_id.clone(), height);
        }
        
        // Save updated metadata
        self.save_metadata().await?;
        
        Ok(record_id)
    }

    async fn read_record(&self, id: &str) -> DatabaseResult<Record> {
        let metadata = self.metadata.lock().await;
        
        let height = metadata
            .get_record_height(id)
            .ok_or_else(|| DatabaseError::RecordNotFound(id.to_string()))?;
        
        drop(metadata); // Release lock before async operation
        
        let blob = self
            .get_blob(height)
            .await?
            .ok_or_else(|| DatabaseError::RecordNotFound(id.to_string()))?;
        
        serde_json::from_slice(&blob.data).map_err(DatabaseError::SerializationError)
    }

    async fn update_record(&self, id: &str, data: Vec<u8>) -> DatabaseResult<()> {
        // First read the existing record
        let mut record = self.read_record(id).await?;
        record.update(data);
        
        // Create and submit new blob
        let blob = Blob::new(
            self.namespace.clone(),
            serde_json::to_vec(&record).map_err(DatabaseError::SerializationError)?,
            AppVersion::V2,
        )
        .map_err(|e| DatabaseError::DatabaseError(e.to_string()))?;
        
        let height = self
            .client
            .blob_submit(&[blob], TxConfig::default())
            .await
            .map_err(|e| DatabaseError::CelestiaError(e.to_string()))?;
        
        // Update metadata
        {
            let mut metadata = self.metadata.lock().await;
            metadata.add_record(id.to_string(), height);
        }
        
        // Save updated metadata
        self.save_metadata().await?;
        
        Ok(())
    }

    async fn delete_record(&self, id: &str) -> DatabaseResult<()> {
        let mut metadata = self.metadata.lock().await;
        
        if !metadata.record_exists(id) {
            return Err(DatabaseError::RecordNotFound(id.to_string()));
        }
        
        metadata.delete_record(id);
        drop(metadata);
        
        // Save updated metadata
        self.save_metadata().await?;
        
        Ok(())
    }

    async fn list_records(&self) -> DatabaseResult<Vec<Record>> {
        let metadata = self.metadata.lock().await;
        let mut records = Vec::new();
        
        for (id, height) in metadata.index.iter() {
            if !metadata.deleted.contains(id) {
                if let Ok(Some(blob)) = self.get_blob(*height).await {
                    if let Ok(record) = serde_json::from_slice(&blob.data) {
                        records.push(record);
                    }
                }
            }
        }
        
        Ok(records)
    }
} 