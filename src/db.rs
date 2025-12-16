// cdb64 database operations module
// Uses cdb64 library for persistent storage

use cdb64::{Cdb, CdbWriter, CdbHash, Error};
use std::collections::HashMap;
use std::fs::{File, create_dir_all};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

pub struct Cdb64Store {
    db_path: PathBuf,
    // In-memory cache for writes before finalizing
    write_cache: Arc<RwLock<HashMap<String, String>>>,
    // Read-only CDB instance
    cdb: Arc<RwLock<Option<Cdb<File, CdbHash>>>>,
}

impl Cdb64Store {
    pub fn new() -> Self {
        // Use ./data directory for persistent storage
        let data_dir = PathBuf::from("./data");
        
        // Create data directory if it doesn't exist
        if let Err(e) = create_dir_all(&data_dir) {
            tracing::warn!("Failed to create data directory: {}", e);
        }
        
        let db_path = data_dir.join("short-rust.cdb");
        
        // Initialize empty cache
        let write_cache = Arc::new(RwLock::new(HashMap::new()));
        
        // Try to open existing CDB file, or create empty
        let cdb = if db_path.exists() {
            match Cdb::<File, CdbHash>::open(&db_path) {
                Ok(cdb_instance) => Arc::new(RwLock::new(Some(cdb_instance))),
                Err(_) => Arc::new(RwLock::new(None)),
            }
        } else {
            Arc::new(RwLock::new(None))
        };
        
        Self {
            db_path,
            write_cache,
            cdb,
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        // First check write cache
        {
            let cache = self.write_cache.read().unwrap();
            if let Some(value) = cache.get(key) {
                return Some(value.clone());
            }
        }
        
        // Then check CDB file
        let cdb_guard = self.cdb.read().unwrap();
        if let Some(ref cdb) = *cdb_guard {
            if let Ok(Some(value_bytes)) = cdb.get(key.as_bytes()) {
                if let Ok(value) = String::from_utf8(value_bytes) {
                    return Some(value);
                }
            }
        }
        
        None
    }

    pub fn set(&self, key: String, value: String) {
        // Add to write cache
        {
            let mut cache = self.write_cache.write().unwrap();
            cache.insert(key.clone(), value.clone());
        }
        
        // Flush to CDB file periodically or on demand
        // For now, we'll flush immediately for simplicity
        if let Err(e) = self.flush() {
            tracing::warn!("Failed to flush to CDB: {}", e);
        }
    }

    pub fn exists(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    fn flush(&self) -> Result<(), Error> {
        // Read all data from cache
        let cache_data = {
            let cache = self.write_cache.read().unwrap();
            cache.clone()
        };
        
        // Read existing CDB data if any
        let mut all_data = HashMap::new();
        
        // Merge existing CDB data
        {
            let cdb_guard = self.cdb.read().unwrap();
            if let Some(ref cdb) = *cdb_guard {
                for result in cdb.iter() {
                    if let Ok((key_bytes, value_bytes)) = result {
                        if let (Ok(key), Ok(value)) = (
                            String::from_utf8(key_bytes),
                            String::from_utf8(value_bytes),
                        ) {
                            all_data.insert(key, value);
                        }
                    }
                }
            }
        }
        
        // Merge cache data (cache takes precedence)
        for (k, v) in cache_data {
            all_data.insert(k, v);
        }
        
        // Create a temporary file path for atomic write
        let temp_path = self.db_path.with_extension("tmp");
        
        // Write all data to temporary CDB file
        {
            let mut writer = CdbWriter::<File, CdbHash>::create(&temp_path)?;
            for (key, value) in &all_data {
                writer.put(key.as_bytes(), value.as_bytes())?;
            }
            writer.finalize()?;
        }
        
        // Atomically replace old file with new one
        std::fs::rename(&temp_path, &self.db_path)
            .map_err(|e| Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to rename temp file: {}", e),
            )))?;
        
        // Reload CDB
        let new_cdb = Cdb::<File, CdbHash>::open(&self.db_path)?;
        {
            let mut cdb_guard = self.cdb.write().unwrap();
            *cdb_guard = Some(new_cdb);
        }
        
        // Clear cache after successful flush
        {
            let mut cache = self.write_cache.write().unwrap();
            cache.clear();
        }
        
        Ok(())
    }
}

impl Default for Cdb64Store {
    fn default() -> Self {
        Self::new()
    }
}
