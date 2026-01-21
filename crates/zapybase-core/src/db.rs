use crate::{Config, Error, Result, VectorDb};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Database manages multiple vector collections
pub struct Database {
    collections: RwLock<HashMap<String, Arc<RwLock<VectorDb>>>>,
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}

impl Database {
    /// Create a new empty database
    pub fn new() -> Self {
        Self {
            collections: RwLock::new(HashMap::new()),
        }
    }

    /// Create a new collection with the given configuration
    pub fn create_collection(&self, name: &str, config: Config) -> Result<()> {
        let mut collections = self.collections.write();
        if collections.contains_key(name) {
            return Err(Error::DuplicateCollection(name.to_string()));
        }
        let db = VectorDb::new(config)?;
        collections.insert(name.to_string(), Arc::new(RwLock::new(db)));
        Ok(())
    }

    /// Delete a collection
    pub fn delete_collection(&self, name: &str) -> Result<()> {
        let mut collections = self.collections.write();
        if collections.remove(name).is_none() {
            return Err(Error::CollectionNotFound(name.to_string()));
        }
        Ok(())
    }

    /// Get a collection by name
    pub fn get_collection(&self, name: &str) -> Result<Arc<RwLock<VectorDb>>> {
        let collections = self.collections.read();
        collections
            .get(name)
            .cloned()
            .ok_or_else(|| Error::CollectionNotFound(name.to_string()))
    }

    /// List all collection names
    pub fn list_collections(&self) -> Vec<String> {
        self.collections.read().keys().cloned().collect()
    }
}
