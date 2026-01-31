use crate::sync::RwLock;
use crate::types::VectorId;
use crate::{
    Config, Error, QuantizationType, QuantizedConfig, QuantizedVectorDb, Result, VectorDb,
};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

#[derive(Debug, Clone, Serialize)]
pub struct CollectionStats {
    pub vector_count: usize,
    pub memory_usage_bytes: usize,
    pub quantization: String,
    pub dimensions: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct DatabaseStats {
    pub collections: HashMap<String, CollectionStats>,
    pub total_vectors: usize,
    pub total_memory_bytes: usize,
}

/// Enum representing either a standard, quantized, or persistent collection
pub enum Collection {
    Standard(Arc<RwLock<VectorDb>>),
    Quantized(Arc<RwLock<QuantizedVectorDb>>),
    #[cfg(feature = "persistence")]
    Persistent(Arc<RwLock<crate::persistent::PersistentVectorDb>>),
}

impl Collection {
    pub fn insert(&self, id: String, vector: &[f32], metadata: Option<Value>) -> Result<()> {
        match self {
            Collection::Standard(db) => db.write().insert(id, vector, metadata),
            Collection::Quantized(db) => db.write().insert(id, vector, metadata),
            #[cfg(feature = "persistence")]
            Collection::Persistent(db) => db.write().insert(id, vector, metadata),
        }
    }

    pub fn upsert(&self, id: String, vector: &[f32], metadata: Option<Value>) -> Result<()> {
        match self {
            Collection::Standard(db) => db.write().upsert(id, vector, metadata),
            Collection::Quantized(db) => db.write().upsert(id, vector, metadata),
            #[cfg(feature = "persistence")]
            Collection::Persistent(db) => {
                let mut db = db.write();
                let _ = db.delete(id.clone());
                db.insert(id, vector, metadata)
            }
        }
    }

    pub fn upsert_batch(&self, items: Vec<(String, Vec<f32>, Option<Value>)>) -> Result<()> {
        match self {
            Collection::Standard(db) => {
                let items_converted: Vec<(VectorId, Vec<f32>, Option<Value>)> = items
                    .into_iter()
                    .map(|(id, vec, meta)| (VectorId::from(id), vec, meta))
                    .collect();
                db.write().upsert_batch(items_converted)
            }
            Collection::Quantized(db) => {
                let items_converted: Vec<(VectorId, Vec<f32>, Option<Value>)> = items
                    .into_iter()
                    .map(|(id, vec, meta)| (VectorId::from(id), vec, meta))
                    .collect();
                db.write().upsert_batch(items_converted)
            }
            #[cfg(feature = "persistence")]
            Collection::Persistent(db) => {
                let mut db = db.write();
                for (id, vector, metadata) in items {
                    let _ = db.delete(id.clone());
                    db.insert(id, &vector, metadata)?;
                }
                Ok(())
            }
        }
    }

    pub fn delete(&self, id: &str) -> Result<bool> {
        match self {
            Collection::Standard(db) => db.write().delete(id),
            Collection::Quantized(db) => db.write().delete(id),
            #[cfg(feature = "persistence")]
            Collection::Persistent(db) => db.write().delete(id),
        }
    }

    pub fn get(&self, id: &str) -> Result<Option<(Vec<f32>, Option<Value>)>> {
        match self {
            Collection::Standard(db) => db.read().get(id),
            Collection::Quantized(db) => db.read().get(id),
            #[cfg(feature = "persistence")]
            Collection::Persistent(db) => db.read().get(id),
        }
    }

    pub fn search(
        &self,
        query: &[f32],
        k: usize,
        filter: Option<&crate::filter::Filter>,
    ) -> Result<Vec<(VectorId, f32, Option<Value>)>> {
        match self {
            Collection::Standard(db) => db.read().search(query, k, filter),
            Collection::Quantized(db) => db.read().search(query, k, filter),
            #[cfg(feature = "persistence")]
            Collection::Persistent(db) => db.read().search(query, k, filter),
        }
    }

    pub fn list(&self, offset: usize, limit: usize) -> Vec<(VectorId, Option<Value>)> {
        match self {
            Collection::Standard(db) => db.read().list(offset, limit),
            Collection::Quantized(db) => db.read().list(offset, limit),
            #[cfg(feature = "persistence")]
            Collection::Persistent(db) => db.read().list(offset, limit),
        }
    }

    pub fn stats(&self) -> CollectionStats {
        match self {
            Collection::Standard(db) => {
                let db = db.read();
                CollectionStats {
                    vector_count: db.len(),
                    memory_usage_bytes: db.memory_usage(),
                    quantization: "None".to_string(),
                    dimensions: db.config().dimensions,
                }
            }
            Collection::Quantized(db) => {
                let db = db.read();
                CollectionStats {
                    vector_count: db.len(),
                    memory_usage_bytes: db.memory_usage(),
                    quantization: format!("{:?}", db.config().quantization),
                    dimensions: db.config().dimensions,
                }
            }
            #[cfg(feature = "persistence")]
            Collection::Persistent(db) => {
                let db = db.read();
                let disk_usage = if let Ok(metadata) = std::fs::metadata(db.data_dir()) {
                    if metadata.is_dir() {
                        get_dir_size(db.data_dir()).unwrap_or(0)
                    } else {
                        metadata.len()
                    }
                } else {
                    0
                };
                CollectionStats {
                    vector_count: db.len(),
                    memory_usage_bytes: disk_usage as usize,
                    quantization: "None".to_string(),
                    dimensions: db.config().dimensions,
                }
            }
        }
    }
}

#[cfg(feature = "persistence")]
fn get_dir_size(path: impl AsRef<std::path::Path>) -> std::io::Result<u64> {
    let mut size = 0;
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            size += get_dir_size(entry.path())?;
        } else {
            size += metadata.len();
        }
    }
    Ok(size)
}

pub struct Database {
    collections: RwLock<HashMap<String, Collection>>,
    #[cfg(feature = "persistence")]
    path: Option<std::path::PathBuf>,
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}

impl Database {
    pub fn new() -> Self {
        Self {
            collections: RwLock::new(HashMap::new()),
            #[cfg(feature = "persistence")]
            path: None,
        }
    }

    #[cfg(feature = "persistence")]
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        std::fs::create_dir_all(&path)?;

        let db = Self {
            collections: RwLock::new(HashMap::new()),
            path: Some(path.clone()),
        };

        info!("Opening SurgeDB at {:?}", path);

        for entry in std::fs::read_dir(&path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let name = entry.file_name().to_string_lossy().into_owned();
                let meta_path = entry.path().join("metadata.json");
                if meta_path.exists() {
                    debug!("Recovering collection: {}", name);
                    let meta_str = std::fs::read_to_string(meta_path)?;
                    let config: Config =
                        serde_json::from_str(&meta_str).map_err(|e| Error::Serialization {
                            message: e.to_string(),
                        })?;
                    let p_config = crate::persistent::PersistentConfig {
                        dimensions: config.dimensions,
                        distance_metric: config.distance_metric,
                        hnsw: config.hnsw.clone(),
                        ..Default::default()
                    };
                    let p_db = crate::persistent::PersistentVectorDb::open(entry.path(), p_config)?;
                    info!("Collection {} recovered with {} vectors", name, p_db.len());
                    db.collections
                        .write()
                        .insert(name, Collection::Persistent(Arc::new(RwLock::new(p_db))));
                }
            }
        }
        Ok(db)
    }

    pub fn create_collection(&self, name: &str, config: Config) -> Result<()> {
        let mut collections = self.collections.write();
        if collections.contains_key(name) {
            return Err(Error::DuplicateCollection(name.to_string()));
        }

        #[cfg(feature = "persistence")]
        let collection = if let Some(base_path) = &self.path {
            let col_path = base_path.join(name);
            std::fs::create_dir_all(&col_path)?;
            let meta_path = col_path.join("metadata.json");
            let meta_json = serde_json::to_string(&config).map_err(|e| Error::Serialization {
                message: e.to_string(),
            })?;
            std::fs::write(meta_path, meta_json)?;
            let p_config = crate::persistent::PersistentConfig {
                dimensions: config.dimensions,
                distance_metric: config.distance_metric,
                hnsw: config.hnsw,
                ..Default::default()
            };
            let p_db = crate::persistent::PersistentVectorDb::open(col_path, p_config)?;
            Collection::Persistent(Arc::new(RwLock::new(p_db)))
        } else {
            Self::create_in_memory_collection(config)?
        };

        #[cfg(not(feature = "persistence"))]
        let collection = Self::create_in_memory_collection(config)?;

        collections.insert(name.to_string(), collection);
        Ok(())
    }

    fn create_in_memory_collection(config: Config) -> Result<Collection> {
        if config.quantization == QuantizationType::None {
            let db = VectorDb::new(config)?;
            Ok(Collection::Standard(Arc::new(RwLock::new(db))))
        } else {
            let q_config = QuantizedConfig {
                dimensions: config.dimensions,
                distance_metric: config.distance_metric,
                hnsw: config.hnsw,
                quantization: config.quantization,
                keep_originals: false,
                rerank_multiplier: 3,
            };
            let db = QuantizedVectorDb::new(q_config)?;
            Ok(Collection::Quantized(Arc::new(RwLock::new(db))))
        }
    }

    pub fn delete_collection(&self, name: &str) -> Result<()> {
        let mut collections = self.collections.write();
        if collections.remove(name).is_some() {
            #[cfg(feature = "persistence")]
            if let Some(base_path) = &self.path {
                let col_path = base_path.join(name);
                if col_path.exists() {
                    let _ = std::fs::remove_dir_all(col_path);
                }
            }
            Ok(())
        } else {
            Err(Error::CollectionNotFound(name.to_string()))
        }
    }

    pub fn get_collection(&self, name: &str) -> Result<Collection> {
        let collections = self.collections.read();
        collections
            .get(name)
            .cloned()
            .ok_or_else(|| Error::CollectionNotFound(name.to_string()))
    }

    pub fn list_collections(&self) -> Vec<String> {
        self.collections.read().keys().cloned().collect()
    }

    pub fn get_stats(&self) -> DatabaseStats {
        let collections = self.collections.read();
        let mut stats_map = HashMap::new();
        let mut total_vectors = 0;
        let mut total_memory = 0;
        for (name, collection) in collections.iter() {
            let stats = collection.stats();
            total_vectors += stats.vector_count;
            total_memory += stats.memory_usage_bytes;
            stats_map.insert(name.clone(), stats);
        }
        DatabaseStats {
            collections: stats_map,
            total_vectors,
            total_memory_bytes: total_memory,
        }
    }
}

impl Clone for Collection {
    fn clone(&self) -> Self {
        match self {
            Collection::Standard(db) => Collection::Standard(db.clone()),
            Collection::Quantized(db) => Collection::Quantized(db.clone()),
            #[cfg(feature = "persistence")]
            Collection::Persistent(db) => Collection::Persistent(db.clone()),
        }
    }
}
